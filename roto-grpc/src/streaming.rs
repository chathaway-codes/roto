//! Zero-copy `#![no_std]` streaming abstractions (`RecvStream` and `SendStream`) for
//! server-streaming, client-streaming, and bidirectional-streaming gRPC RPCs.

use core::fmt;
use core::iter::FusedIterator;

use roto_runtime::{BackwardWriter, DecodeError, MessageView};

use crate::framing::{prepend_grpc_header, take_writer, GrpcFrameIter, DEFAULT_MAX_MESSAGE_SIZE};
use crate::status::GrpcStatus;

/// Zero-copy receiver yielding decoded protobuf message views from a gRPC stream
/// until `END_STREAM`.
pub trait RecvStream<V> {
    /// Decodes and returns the next message from the stream, or `Ok(None)` when the stream
    /// has reached `END_STREAM`.
    fn next(&mut self) -> Result<Option<V>, GrpcStatus<'static>>;

    /// Alias for [`RecvStream::next`].
    #[inline]
    fn recv(&mut self) -> Result<Option<V>, GrpcStatus<'static>> {
        self.next()
    }
}

impl<R: RecvStream<V> + ?Sized, V> RecvStream<V> for &mut R {
    #[inline]
    fn next(&mut self) -> Result<Option<V>, GrpcStatus<'static>> {
        (**self).next()
    }
}

/// Zero-allocation sender encoding protobuf messages backwards via `B` (`<Output>Builder<'b>`)
/// and framing each message with a 5-byte gRPC Length-Prefixed-Message (LPM) header.
pub trait SendStream<B> {
    /// Encodes a single outbound message using `f` and appends its LPM frame to the stream.
    fn send<F>(&mut self, f: F) -> Result<(), GrpcStatus<'static>>
    where
        F: FnOnce(&mut B);
}

impl<S: SendStream<B> + ?Sized, B> SendStream<B> for &mut S {
    #[inline]
    fn send<F>(&mut self, f: F) -> Result<(), GrpcStatus<'static>>
    where
        F: FnOnce(&mut B),
    {
        (**self).send(f)
    }
}

/// Zero-copy [`RecvStream`] decoding consecutive gRPC Length-Prefixed-Message (LPM) frames
/// from a borrowed wire slice `&'a [u8]`.
#[must_use]
#[derive(Clone, Copy)]
pub struct LpmRecvStream<'a, V> {
    iter: GrpcFrameIter<'a>,
    decode: fn(&'a [u8]) -> Result<V, DecodeError>,
}

impl<V> fmt::Debug for LpmRecvStream<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LpmRecvStream")
            .field("iter", &self.iter)
            .finish()
    }
}

impl<'a, V> LpmRecvStream<'a, V> {
    /// Creates a new [`LpmRecvStream`] over `buf` using `decode` (`<Input>View::from_bytes`)
    /// and [`DEFAULT_MAX_MESSAGE_SIZE`].
    #[inline]
    pub const fn new(buf: &'a [u8], decode: fn(&'a [u8]) -> Result<V, DecodeError>) -> Self {
        Self::with_max_message_size(buf, DEFAULT_MAX_MESSAGE_SIZE, decode)
    }

    /// Creates a new [`LpmRecvStream`] over `buf` using `decode` and a custom `max_message_size`.
    #[inline]
    pub const fn with_max_message_size(
        buf: &'a [u8],
        max_message_size: usize,
        decode: fn(&'a [u8]) -> Result<V, DecodeError>,
    ) -> Self {
        Self {
            iter: GrpcFrameIter::new(buf, max_message_size).with_require_complete(true),
            decode,
        }
    }

    /// Creates a new [`LpmRecvStream`] with [`DEFAULT_MAX_MESSAGE_SIZE`] for any [`MessageView`].
    #[inline]
    pub const fn from_view_slice(buf: &'a [u8]) -> Self
    where
        V: MessageView<'a>,
    {
        Self::new(buf, V::from_bytes)
    }

    /// Returns the unconsumed trailing byte slice.
    #[inline]
    pub const fn remainder(&self) -> &'a [u8] {
        self.iter.remainder()
    }

    /// Returns `true` if no unconsumed bytes remain in the stream.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }

    /// Decodes and returns the next message from the stream, or `Ok(None)` at `END_STREAM`.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<V>, GrpcStatus<'static>> {
        match self.iter.next() {
            None => Ok(None),
            Some(Err(status)) => Err(status),
            Some(Ok(frame)) => {
                if frame.compressed {
                    return Err(GrpcStatus::internal("message compression not supported"));
                }
                (self.decode)(frame.payload)
                    .map(Some)
                    .map_err(|_| GrpcStatus::internal("invalid protobuf message payload"))
            }
        }
    }

    /// Alias for [`LpmRecvStream::next`].
    #[inline]
    pub fn recv(&mut self) -> Result<Option<V>, GrpcStatus<'static>> {
        self.next()
    }
}

impl<'a, V> RecvStream<V> for LpmRecvStream<'a, V> {
    #[inline]
    fn next(&mut self) -> Result<Option<V>, GrpcStatus<'static>> {
        LpmRecvStream::next(self)
    }
}

impl<'a, V> Iterator for LpmRecvStream<'a, V> {
    type Item = Result<V, GrpcStatus<'static>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        LpmRecvStream::next(self).transpose()
    }
}

impl<V> FusedIterator for LpmRecvStream<'_, V> {}

/// Zero-allocation [`SendStream`] that encodes each message backwards into a [`BackwardWriter`],
/// prepends its 5-byte gRPC LPM header in-place, and rotates each completed frame to the end
/// of the written region so consecutive messages appear in forward wire order.
#[must_use]
pub struct WriterSendStream<'w, 'b, B> {
    writer: &'w mut BackwardWriter<'b>,
    from_writer: fn(BackwardWriter<'b>) -> B,
    into_writer: fn(B) -> BackwardWriter<'b>,
    messages_sent: usize,
}

impl<B> fmt::Debug for WriterSendStream<'_, '_, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriterSendStream")
            .field("writer", &self.writer)
            .field("messages_sent", &self.messages_sent)
            .finish()
    }
}

impl<'w, 'b, B> WriterSendStream<'w, 'b, B> {
    /// Wraps `writer` with the generated builder constructor (`<Output>Builder::from_writer`)
    /// and extractor (`|b| b.writer`).
    #[inline]
    pub fn new(
        writer: &'w mut BackwardWriter<'b>,
        from_writer: fn(BackwardWriter<'b>) -> B,
        into_writer: fn(B) -> BackwardWriter<'b>,
    ) -> Self {
        Self {
            writer,
            from_writer,
            into_writer,
            messages_sent: 0,
        }
    }

    /// Returns the number of LPM-framed messages successfully written so far.
    #[inline]
    pub const fn messages_sent(&self) -> usize {
        self.messages_sent
    }

    /// Encodes a single outbound message using `f`, prepends its 5-byte gRPC LPM header,
    /// and preserves forward stream order in the underlying [`BackwardWriter`].
    #[inline]
    pub fn send<F>(&mut self, f: F) -> Result<(), GrpcStatus<'static>>
    where
        F: FnOnce(&mut B),
    {
        let end_pos = self.writer.cursor();
        let taken = take_writer(self.writer);
        let mut builder = (self.from_writer)(taken);
        f(&mut builder);
        *self.writer = (self.into_writer)(builder);
        prepend_grpc_header(self.writer, false, end_pos).map_err(GrpcStatus::from)?;
        self.writer.rotate_written_to_end(end_pos);
        self.messages_sent += 1;
        Ok(())
    }

    /// Appends a pre-serialized protobuf message `payload` as a 5-byte LPM frame in forward order.
    #[inline]
    pub fn send_bytes(&mut self, payload: &[u8]) -> Result<(), GrpcStatus<'static>> {
        let end_pos = self.writer.cursor();
        self.writer.write_bytes(payload).map_err(GrpcStatus::from)?;
        prepend_grpc_header(self.writer, false, end_pos).map_err(GrpcStatus::from)?;
        self.writer.rotate_written_to_end(end_pos);
        self.messages_sent += 1;
        Ok(())
    }
}

impl<'w, 'b, B> SendStream<B> for WriterSendStream<'w, 'b, B> {
    #[inline]
    fn send<F>(&mut self, f: F) -> Result<(), GrpcStatus<'static>>
    where
        F: FnOnce(&mut B),
    {
        WriterSendStream::send(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::GrpcCode;
    use roto_runtime::WireType;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct DummyView<'a>(&'a [u8]);

    impl<'a> DummyView<'a> {
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            if buf == b"bad" {
                return Err(DecodeError::InvalidVarint);
            }
            Ok(Self(buf))
        }
    }

    impl<'a> MessageView<'a> for DummyView<'a> {
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    struct DummyBuilder<'b> {
        writer: BackwardWriter<'b>,
    }

    impl<'b> DummyBuilder<'b> {
        const fn from_writer(writer: BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        fn write_val(&mut self, v: u64) {
            let _ = self.writer.write_varint(v);
            let _ = self.writer.write_tag(1, WireType::Varint);
        }
    }

    #[test]
    fn send_and_recv_multiple_messages_in_forward_wire_order() {
        let mut buf = [0u8; 128];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut tx =
                WriterSendStream::new(&mut writer, DummyBuilder::from_writer, |b| b.writer);
            assert_eq!(tx.messages_sent(), 0);
            SendStream::send(&mut tx, |b| b.write_val(10)).unwrap();
            tx.send(|b| b.write_val(20)).unwrap();
            tx.send(|b| b.write_val(30)).unwrap();
            assert_eq!(tx.messages_sent(), 3);
        }

        let framed = writer.finish().unwrap();
        let mut rx = LpmRecvStream::from_view_slice(framed);
        assert!(!rx.is_empty());
        assert_eq!(
            RecvStream::next(&mut rx).unwrap(),
            Some(DummyView(&[0x08, 10]))
        );
        assert_eq!(rx.next().unwrap(), Some(DummyView(&[0x08, 20])));
        assert_eq!(rx.recv().unwrap(), Some(DummyView(&[0x08, 30])));
        assert_eq!(rx.next().unwrap(), None);
        assert!(rx.is_empty());
        assert_eq!(rx.remainder(), &[]);
    }

    #[test]
    fn recv_stream_reports_compressed_truncated_and_invalid_payloads() {
        // Compressed frame -> Internal
        let compressed = [1, 0, 0, 0, 1, 0x00];
        let mut rx = LpmRecvStream::with_max_message_size(&compressed, 64, DummyView::from_bytes);
        assert_eq!(rx.next().unwrap_err().code, GrpcCode::Internal);

        // Invalid payload -> Internal
        let bad_payload = [0, 0, 0, 0, 3, b'b', b'a', b'd'];
        let mut rx_bad = LpmRecvStream::new(&bad_payload, DummyView::from_bytes);
        assert_eq!(rx_bad.next().unwrap_err().code, GrpcCode::Internal);

        // Truncated trailing frame -> Internal
        let truncated = [0, 0, 0, 0, 5, 1, 2];
        let mut rx_trunc = LpmRecvStream::new(&truncated, DummyView::from_bytes);
        assert_eq!(rx_trunc.next().unwrap_err().code, GrpcCode::Internal);
    }

    #[test]
    fn send_stream_reports_buffer_overflow_as_resource_exhausted() {
        let mut tiny = [0u8; 6];
        let mut writer = BackwardWriter::new(&mut tiny);
        let mut tx = WriterSendStream::new(&mut writer, DummyBuilder::from_writer, |b| b.writer);
        let err = tx.send_bytes(&[1, 2, 3]).unwrap_err();
        assert_eq!(err.code, GrpcCode::ResourceExhausted);
    }
}
