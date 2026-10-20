//! Plumbing shared by the [`crate::server`] and [`crate::client`] Sans-I/O drivers.

use crate::h2::{H2Connection, H2Error, FRAME_HEADER_LEN};
use crate::status::GrpcStatus;

/// Scratch capacity for Huffman-decoded HPACK strings of a single header block.
pub(crate) const HPACK_SCRATCH_LEN: usize = 512;

const SEND_EXHAUSTED: GrpcStatus<'static> =
    GrpcStatus::resource_exhausted("message exceeds transmit buffer or send window");

/// Caller-provided outbound byte queue that interleaves the connection's queued control
/// frames with stream frames in wire order.
#[derive(Debug)]
pub(crate) struct Outbound<'b> {
    buf: &'b mut [u8],
    len: usize,
}

impl<'b> Outbound<'b> {
    pub(crate) fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Discards everything written after `mark` (a previous [`Self::len`]).
    pub(crate) fn truncate(&mut self, mark: usize) {
        self.len = mark;
    }

    /// Moves queued connection control frames into this queue, ahead of any later stream frames.
    pub(crate) fn flush<const M: usize, const R: usize>(&mut self, conn: &mut H2Connection<M, R>) {
        self.len += conn.poll_transmit(&mut self.buf[self.len..]);
    }

    /// Appends the frame bytes written by `f` into the free tail of this queue.
    pub(crate) fn push<const M: usize, const R: usize>(
        &mut self,
        conn: &mut H2Connection<M, R>,
        f: impl FnOnce(&mut H2Connection<M, R>, &mut [u8]) -> Result<usize, H2Error>,
    ) -> Result<(), H2Error> {
        self.flush(conn);
        self.len += f(conn, &mut self.buf[self.len..])?;
        Ok(())
    }

    /// Returns `true` if `msg` fits as `DATA` frames into the free tail and both send windows.
    pub(crate) fn can_send<const M: usize, const R: usize>(
        &self,
        conn: &H2Connection<M, R>,
        reserve: usize,
        window: i64,
        msg: &[u8],
    ) -> bool {
        let max = conn.peer_settings().max_frame_size as usize;
        let wire = msg.len() + msg.len().div_ceil(max) * FRAME_HEADER_LEN;
        msg.len() as i64 <= window.min(conn.conn_send_window())
            && reserve + wire <= self.buf.len() - self.len
    }

    /// Writes the LPM-framed `msg` as `DATA` frames split at the peer's `max_frame_size`,
    /// setting `END_STREAM` on the last one if `end_stream`. Fails without side effects if the
    /// queue or the send windows cannot take the whole message.
    pub(crate) fn send_data<const M: usize, const R: usize>(
        &mut self,
        conn: &mut H2Connection<M, R>,
        stream_id: u32,
        msg: &[u8],
        end_stream: bool,
    ) -> Result<(), GrpcStatus<'static>> {
        self.flush(conn);
        let window = conn.stream_send_window(stream_id).unwrap_or(0);
        if !self.can_send(conn, 0, window, msg) {
            return Err(SEND_EXHAUSTED);
        }
        let max = conn.peer_settings().max_frame_size as usize;
        let mut chunks = msg.chunks(max).peekable();
        while let Some(chunk) = chunks.next() {
            let last = end_stream && chunks.peek().is_none();
            self.len += conn
                .send_data_frame(stream_id, chunk, last, &mut self.buf[self.len..])
                .map_err(|_| SEND_EXHAUSTED)?;
        }
        Ok(())
    }

    /// Drains queued bytes into `out` and returns the number of bytes written.
    pub(crate) fn poll_transmit<const M: usize, const R: usize>(
        &mut self,
        conn: &mut H2Connection<M, R>,
        out: &mut [u8],
    ) -> usize {
        self.flush(conn);
        let n = self.len.min(out.len());
        out[..n].copy_from_slice(&self.buf[..n]);
        self.buf.copy_within(n..self.len, 0);
        self.len -= n;
        n
    }
}
