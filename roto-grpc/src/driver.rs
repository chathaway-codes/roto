//! Plumbing shared by the [`crate::server`] and [`crate::client`] Sans-I/O drivers.

use crate::h2::{H2Connection, H2Error, FRAME_HEADER_LEN};

/// Scratch capacity for Huffman-decoded HPACK strings of a single header block.
pub(crate) const HPACK_SCRATCH_LEN: usize = 512;

/// Writes as much of `msg` as `DATA` frames into `dst[!*len..]` (bounded by `dst.len() - *len`,
/// connection/stream send windows, and peer `max_frame_size`), returning `(bytes_sent, completed)`.
///
/// When `end_stream` is `true`, `END_STREAM` is set on the final `DATA` frame (emitting a
/// 0-byte `DATA` frame if `msg` is already empty), and `completed` is `true` iff that final
/// `END_STREAM` frame was queued. When `end_stream` is `false`, `completed` is `true` iff
/// all bytes of `msg` were queued.
pub(crate) fn send_partial_data_into<const M: usize, const R: usize>(
    conn: &mut H2Connection<M, R>,
    stream_id: u32,
    msg: &[u8],
    end_stream: bool,
    dst: &mut [u8],
    len: &mut usize,
) -> Result<(usize, bool), H2Error> {
    if msg.is_empty() {
        if !end_stream {
            return Ok((0, true));
        }
        if dst.len().saturating_sub(*len) < FRAME_HEADER_LEN {
            return Ok((0, false));
        }
        let n = conn.send_data_frame(stream_id, &[], true, &mut dst[*len..])?;
        *len += n;
        return Ok((0, true));
    }

    let max_frame = conn.peer_settings().max_frame_size as usize;
    let mut sent = 0usize;
    while sent < msg.len() {
        let avail = dst.len().saturating_sub(*len);
        if avail <= FRAME_HEADER_LEN {
            break;
        }
        let stream_win = conn.stream_send_window(stream_id).unwrap_or(0);
        let win = conn.conn_send_window().min(stream_win).max(0) as usize;
        if win == 0 {
            break;
        }
        let chunk_len = (msg.len() - sent)
            .min(max_frame)
            .min(win)
            .min(avail - FRAME_HEADER_LEN);
        if chunk_len == 0 {
            break;
        }
        let is_last = sent + chunk_len == msg.len();
        let frame_end_stream = end_stream && is_last;
        let n = conn.send_data_frame(
            stream_id,
            &msg[sent..sent + chunk_len],
            frame_end_stream,
            &mut dst[*len..],
        )?;
        *len += n;
        sent += chunk_len;
    }
    Ok((sent, sent == msg.len()))
}

/// Caller-provided outbound byte queue that interleaves the connection's queued control
/// frames with stream frames in wire order.
#[derive(Debug)]
pub(crate) struct Outbound<'b> {
    pub(crate) buf: &'b mut [u8],
    pub(crate) len: usize,
}

impl<'b> Outbound<'b> {
    pub(crate) fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn cap(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn free(&self) -> usize {
        self.buf.len() - self.len
    }

    pub(crate) fn clear(&mut self) {
        self.len = 0;
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

    /// Writes as much of `msg` as `DATA` frames into the free tail, returning `(bytes_sent, completed)`.
    pub(crate) fn send_partial_data<const M: usize, const R: usize>(
        &mut self,
        conn: &mut H2Connection<M, R>,
        stream_id: u32,
        msg: &[u8],
        end_stream: bool,
    ) -> Result<(usize, bool), H2Error> {
        self.flush(conn);
        send_partial_data_into(conn, stream_id, msg, end_stream, self.buf, &mut self.len)
    }

    /// Copies up to `out.len()` queued bytes from the head of `self.buf[..self.len]` into `out`.
    pub(crate) fn drain(&mut self, out: &mut [u8]) -> usize {
        let n = self.len.min(out.len());
        out[..n].copy_from_slice(&self.buf[..n]);
        self.buf.copy_within(n..self.len, 0);
        self.len -= n;
        n
    }

    /// Drains queued bytes into `out` and returns the number of bytes written.
    pub(crate) fn poll_transmit<const M: usize, const R: usize>(
        &mut self,
        conn: &mut H2Connection<M, R>,
        out: &mut [u8],
    ) -> usize {
        self.flush(conn);
        self.drain(out)
    }
}

