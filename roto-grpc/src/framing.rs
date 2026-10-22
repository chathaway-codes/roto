use core::iter::FusedIterator;

use roto_runtime::{BackwardWriter, EncodeError};

use crate::status::GrpcStatus;

/// Size in bytes of the gRPC Length-Prefixed-Message (LPM) header (`1` byte compression flag + `4` byte big-endian length).
pub const GRPC_HEADER_LEN: usize = 5;

/// Default maximum inbound gRPC message payload size (`4 MiB`).
pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

/// Zero-copy view of a single gRPC Length-Prefixed-Message (LPM) frame.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrpcFrame<'a> {
    /// Whether the payload is compressed (`true` when the LPM compression flag byte is `1`).
    pub compressed: bool,
    /// Borrowed raw message payload bytes (excluding the 5-byte LPM header).
    pub payload: &'a [u8],
}

/// Moves the [`BackwardWriter`] out of `writer`, leaving an empty placeholder writer in its place.
///
/// This enables constructing a `<Msg>Builder<'b>` via `from_writer(take_writer(writer))` in both
/// synchronous and `async` handlers before restoring `*writer = builder.writer`.
#[inline]
pub fn take_writer<'b>(writer: &mut BackwardWriter<'b>) -> BackwardWriter<'b> {
    core::mem::replace(writer, BackwardWriter::new(&mut []))
}

/// Prepends the 5-byte gRPC Length-Prefixed-Message (LPM) header
/// `[u8::from(compressed), len_be[0], len_be[1], len_be[2], len_be[3]]` onto `writer`
/// for a message payload written between `end_pos` and `writer.cursor()`.
pub fn prepend_grpc_header(
    writer: &mut BackwardWriter<'_>,
    compressed: bool,
    end_pos: usize,
) -> Result<(), EncodeError> {
    writer.check()?;
    let msg_len = end_pos - writer.cursor();
    let Ok(len_u32) = u32::try_from(msg_len) else {
        // Latch sticky BufferOverflow onto `writer`.
        let rem = writer.remaining_capacity();
        for _ in 0..=rem {
            let _ = writer.write_bytes(&[0]);
        }
        return Err(EncodeError::BufferOverflow);
    };
    let len_be = len_u32.to_be_bytes();
    let header = [
        u8::from(compressed),
        len_be[0],
        len_be[1],
        len_be[2],
        len_be[3],
    ];
    writer.write_bytes(&header)
}

/// Encodes a gRPC Length-Prefixed-Message (LPM) frame in a single backward pass onto `writer`.
///
/// Executes `f` on a sub-writer at the current cursor to serialize the Protobuf message payload,
/// computes the written message byte length (`msg_len = end_pos - writer.cursor()`), and prepends
/// the 5-byte LPM header `[u8::from(compressed), len_be[0], len_be[1], len_be[2], len_be[3]]`
/// directly in front of the payload with zero copies.
pub fn write_grpc_frame<F>(
    writer: &mut BackwardWriter<'_>,
    compressed: bool,
    f: F,
) -> Result<(), EncodeError>
where
    F: FnOnce(&mut BackwardWriter<'_>),
{
    let end_pos = writer.cursor();
    writer.with_sub_writer(|mut sub| {
        f(&mut sub);
        sub
    });
    prepend_grpc_header(writer, compressed, end_pos)
}

/// Convenience helper that writes a pre-encoded `payload` slice and prepends its 5-byte
/// gRPC LPM header onto `writer`.
#[inline]
pub fn write_grpc_frame_bytes(
    writer: &mut BackwardWriter<'_>,
    compressed: bool,
    payload: &[u8],
) -> Result<(), EncodeError> {
    write_grpc_frame(writer, compressed, |w| {
        let _ = w.write_bytes(payload);
    })
}

/// Encodes a single gRPC LPM frame into `buf` using a fresh [`BackwardWriter`] and returns
/// the contiguous written tail slice (`[5-byte LPM header + payload]`).
#[inline]
pub fn encode_grpc_message<F>(buf: &mut [u8], compressed: bool, f: F) -> Result<&[u8], EncodeError>
where
    F: FnOnce(&mut BackwardWriter<'_>),
{
    let mut writer = BackwardWriter::new(buf);
    write_grpc_frame(&mut writer, compressed, f)?;
    writer.finish()
}

/// Encodes a pre-serialized `payload` slice as a single gRPC LPM frame into `buf` and returns
/// the contiguous written tail slice (`[5-byte LPM header + payload]`).
#[inline]
pub fn encode_grpc_message_bytes<'a>(
    buf: &'a mut [u8],
    compressed: bool,
    payload: &[u8],
) -> Result<&'a [u8], EncodeError> {
    let mut writer = BackwardWriter::new(buf);
    write_grpc_frame_bytes(&mut writer, compressed, payload)?;
    writer.finish()
}

/// Attempts to decode a single gRPC Length-Prefixed-Message (LPM) frame from the front of `buf`.
///
/// Returns:
/// - `Ok(None)` if fewer than `5` header bytes or fewer than `5 + msg_len` total bytes are available.
/// - `Ok(Some((frame, remaining)))` when a complete frame is present.
/// - `Err(GrpcStatus)` with [`GrpcCode::Internal`](crate::status::GrpcCode::Internal) if the compression
///   flag byte is greater than `1`, or [`GrpcCode::ResourceExhausted`](crate::status::GrpcCode::ResourceExhausted)
///   if `msg_len > max_message_size`.
pub fn decode_grpc_frame<'a>(
    buf: &'a [u8],
    max_message_size: usize,
) -> Result<Option<(GrpcFrame<'a>, &'a [u8])>, GrpcStatus<'static>> {
    if buf.len() < GRPC_HEADER_LEN {
        return Ok(None);
    }
    let compressed_byte = buf[0];
    if compressed_byte > 1 {
        return Err(GrpcStatus::internal("invalid gRPC compression flag"));
    }
    let msg_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
    if msg_len > max_message_size {
        return Err(GrpcStatus::resource_exhausted(
            "gRPC message exceeds maximum size",
        ));
    }
    if buf.len() - GRPC_HEADER_LEN < msg_len {
        return Ok(None);
    }
    let frame_end = GRPC_HEADER_LEN + msg_len;
    let frame = GrpcFrame {
        compressed: compressed_byte == 1,
        payload: &buf[GRPC_HEADER_LEN..frame_end],
    };
    Ok(Some((frame, &buf[frame_end..])))
}

/// Streaming iterator over consecutive gRPC Length-Prefixed-Message (LPM) frames in a byte slice.
///
/// By default (`require_complete = false`), iteration stops cleanly (`None`) when fewer than
/// `5 + msg_len` trailing bytes remain, exposing the unconsumed tail via [`GrpcFrameIter::remainder`]
/// so a streaming HTTP/2 deframer can retain partial bytes for the next `DATA` frame.
/// When `require_complete = true` (via [`GrpcFrameIter::with_require_complete`]), any trailing
/// partial frame at the end of the buffer yields `Some(Err(GrpcStatus::internal(...)))`.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct GrpcFrameIter<'a> {
    buf: &'a [u8],
    max_message_size: usize,
    require_complete: bool,
    failed: bool,
}

/// Type alias for [`GrpcFrameIter`] when used as a streaming LPM frame decoder.
pub type LpmDecoder<'a> = GrpcFrameIter<'a>;

impl<'a> GrpcFrameIter<'a> {
    /// Creates a new [`GrpcFrameIter`] over `buf` with the given `max_message_size`.
    #[inline]
    pub const fn new(buf: &'a [u8], max_message_size: usize) -> Self {
        Self {
            buf,
            max_message_size,
            require_complete: false,
            failed: false,
        }
    }

    /// Configures whether trailing partial bytes at the end of `buf` should be reported as
    /// an error (`true`) or left in [`remainder`](Self::remainder) (`false`).
    #[inline]
    pub const fn with_require_complete(mut self, require_complete: bool) -> Self {
        self.require_complete = require_complete;
        self
    }

    /// Returns the unconsumed trailing slice of the buffer.
    #[inline]
    pub const fn remainder(&self) -> &'a [u8] {
        self.buf
    }

    /// Returns `true` if there are no unconsumed bytes remaining in the buffer.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

impl<'a> Iterator for GrpcFrameIter<'a> {
    type Item = Result<GrpcFrame<'a>, GrpcStatus<'static>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.buf.is_empty() {
            return None;
        }
        match decode_grpc_frame(self.buf, self.max_message_size) {
            Ok(Some((frame, rest))) => {
                self.buf = rest;
                Some(Ok(frame))
            }
            Ok(None) => {
                self.failed = true;
                if self.require_complete {
                    Some(Err(GrpcStatus::internal("truncated gRPC frame")))
                } else {
                    None
                }
            }
            Err(status) => {
                self.failed = true;
                Some(Err(status))
            }
        }
    }
}

impl FusedIterator for GrpcFrameIter<'_> {}

/// Returns a [`GrpcFrameIter`] yielding complete gRPC LPM frames from `buf`.
#[inline]
pub const fn iter_grpc_frames(buf: &[u8], max_message_size: usize) -> GrpcFrameIter<'_> {
    GrpcFrameIter::new(buf, max_message_size)
}
