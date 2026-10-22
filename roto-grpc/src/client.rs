//! Sans-I/O gRPC client driver multiplexing unary calls over one [`H2Connection`].

use roto_runtime::BackwardWriter;

use crate::driver::{Outbound, HPACK_SCRATCH_LEN};
use crate::framing::encode_grpc_message;
use crate::h2::{
    H2Connection, H2Error, H2ErrorCode, H2Event, Role, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP,
    FRAME_HEADER_LEN,
};
use crate::hpack::{is_grpc_content_type, GrpcTrailers, HpackDecoder, GRPC_CONTENT_TYPE_ROTO};
use crate::server::GrpcServer;
use crate::service::UnaryChannel;
use crate::status::{percent_decode, GrpcCode, GrpcStatus};

/// Upper bound on the HPACK-encoded request header block size excluding `:scheme`,
/// `:authority`, and `:path` values.
const REQUEST_HEADERS_OVERHEAD: usize = 64;

/// The outcome of a finished unary call: the response payload or the error status, stored in
/// the call's buffer.
#[must_use]
pub struct Reply<'b> {
    pub(crate) code: GrpcCode,
    pub(crate) len: usize,
    pub(crate) buf: &'b mut [u8],
}

impl core::fmt::Debug for Reply<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Reply")
            .field("code", &self.code)
            .field("len", &self.len)
            .field("buf_cap", &self.buf.len())
            .finish()
    }
}

impl<'b> Reply<'b> {
    fn failed(buf: &'b mut [u8], code: GrpcCode, message: &str) -> Self {
        let len = write_truncated(buf, message.as_bytes());
        Self { code, len, buf }
    }

    /// Returns the [`GrpcCode`] of the completed call.
    #[inline]
    pub const fn code(&self) -> GrpcCode {
        self.code
    }

    /// Returns the raw response protobuf payload, or the call's [`GrpcStatus`].
    pub fn result(&self) -> Result<&[u8], GrpcStatus<'_>> {
        reply_result(self.code, &self.buf[..self.len])
    }

    /// Releases the call buffer for reuse.
    #[must_use]
    pub fn into_buf(self) -> &'b mut [u8] {
        self.buf
    }
}

/// Copies as much of `message` into `buf` as fits without splitting a multi-byte UTF-8 codepoint.
fn write_truncated(buf: &mut [u8], message: &[u8]) -> usize {
    let mut len = message.len().min(buf.len());
    if len < message.len() {
        while len > 0 && (message[len] & 0xC0) == 0x80 {
            len -= 1;
        }
    }
    buf[..len].copy_from_slice(&message[..len]);
    len
}

pub(crate) fn reply_result(code: GrpcCode, bytes: &[u8]) -> Result<&[u8], GrpcStatus<'_>> {
    if code == GrpcCode::Ok {
        Ok(bytes)
    } else {
        Err(GrpcStatus::new(
            code,
            core::str::from_utf8(bytes).unwrap_or(""),
        ))
    }
}

/// Maps a non-`200` HTTP `:status` value to a [`GrpcCode`] per the gRPC HTTP-to-gRPC status
/// mapping specification.
fn http_status_to_grpc_code(status: Option<&[u8]>) -> GrpcCode {
    match status {
        Some(b"400") => GrpcCode::Internal,
        Some(b"401") => GrpcCode::Unauthenticated,
        Some(b"403") => GrpcCode::PermissionDenied,
        Some(b"404") => GrpcCode::Unimplemented,
        Some(b"429" | b"502" | b"503" | b"504") => GrpcCode::Unavailable,
        _ => GrpcCode::Unknown,
    }
}

struct Call<'b> {
    stream_id: u32,
    buf: &'b mut [u8],
    req_offset: usize,
    req_end: usize,
    req_done: bool,
    got_headers: bool,
    streaming_response: bool,
    msg_len: Option<usize>,
    done: Option<(GrpcCode, usize)>,
}

impl Call<'_> {
    fn finish(&mut self, code: GrpcCode, message: &[u8]) {
        if self.done.is_none() {
            self.req_done = true;
            let len = write_truncated(self.buf, message);
            self.done = Some((code, len));
        }
    }

    fn on_headers(&mut self, header_block: &[u8], end_stream: bool) {
        if self.done.is_some() {
            return;
        }
        let mut scratch = [0u8; HPACK_SCRATCH_LEN];
        let mut decoder = HpackDecoder::<0>::stateless();
        let Ok(headers) = GrpcTrailers::decode(&mut decoder, header_block, &mut scratch) else {
            return self.finish(GrpcCode::Internal, b"malformed response headers");
        };
        if !self.got_headers {
            self.got_headers = true;
            if headers.status != Some(b"200") {
                let code = http_status_to_grpc_code(headers.status);
                return self.finish(code, b"non-200 HTTP :status");
            }
            if !headers.content_type.is_some_and(is_grpc_content_type) {
                return self.finish(GrpcCode::Unknown, b"invalid response content-type");
            }
        }
        if !end_stream {
            return;
        }
        match (headers.grpc_status, self.msg_len) {
            (None, _) => self.finish(GrpcCode::Internal, b"missing grpc-status"),
            (Some(GrpcCode::Ok), Some(len)) => {
                self.req_done = true;
                self.done = Some((GrpcCode::Ok, len));
            }
            (Some(GrpcCode::Ok), None) => {
                self.finish(GrpcCode::Internal, b"missing response message");
            }
            (Some(code), _) => {
                let raw = headers.grpc_message.unwrap_or_default();
                let mut msg_scratch = [0u8; HPACK_SCRATCH_LEN];
                let decoded = percent_decode(raw, &mut msg_scratch).unwrap_or("");
                self.finish(code, decoded.as_bytes());
            }
        }
    }

    fn on_message(&mut self, compressed: bool, payload: &[u8]) {
        if self.done.is_some() {
            return;
        }
        if compressed {
            return self.finish(GrpcCode::Internal, b"message compression not supported");
        }
        let max_len = if self.req_done {
            self.buf.len()
        } else {
            self.req_offset
        };
        if self.streaming_response {
            let offset = self.msg_len.unwrap_or(0);
            let frame_len = crate::framing::GRPC_HEADER_LEN + payload.len();
            let Ok(len_u32) = u32::try_from(payload.len()) else {
                return self.finish(GrpcCode::ResourceExhausted, b"response exceeds call buffer");
            };
            if offset.saturating_add(frame_len) > max_len {
                return self.finish(GrpcCode::ResourceExhausted, b"response exceeds call buffer");
            }
            self.buf[offset] = 0;
            self.buf[offset + 1..offset + 5].copy_from_slice(&len_u32.to_be_bytes());
            self.buf[offset + 5..offset + frame_len].copy_from_slice(payload);
            self.msg_len = Some(offset + frame_len);
        } else {
            if self.msg_len.is_some() {
                return self.finish(GrpcCode::Internal, b"multiple response messages");
            }
            if payload.len() > max_len {
                return self.finish(GrpcCode::ResourceExhausted, b"response exceeds call buffer");
            }
            self.buf[..payload.len()].copy_from_slice(payload);
            self.msg_len = Some(payload.len());
        }
    }
}

/// `#![no_std]` zero-allocation Sans-I/O gRPC client over one HTTP/2 connection.
///
/// Start any number of concurrent calls (up to `MAX_STREAMS`) with [`Self::start_call`],
/// [`Self::start_server_streaming_call`], [`Self::start_client_streaming_call`], or
/// [`Self::start_bidi_streaming_call`], shuttle bytes with [`Self::poll_transmit`] and
/// [`Self::recv`], then collect each call's [`Reply`] with [`Self::take_reply`]. Each call
/// borrows a caller buffer that first holds the encoded request and then the response payload
/// (or error message).
///
/// # Design Constraints
///
/// - **Zero-allocation flow-control streaming**: Outbound requests are encoded into the call
///   buffer and streamed as `DATA` frames across `tx_buf` drains and peer `WINDOW_UPDATE`
///   frames, allowing requests larger than the initial 65,535-byte HTTP/2 send window or
///   smaller `tx_buf` capacities.
/// - **Stateless HPACK**: Advertises `SETTINGS_HEADER_TABLE_SIZE = 0` and decodes header blocks
///   with a zero-capacity dynamic table.
#[must_use]
pub struct GrpcClient<
    'b,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    conn: H2Connection<MAX_STREAMS, REASM_CAP>,
    scheme: &'b str,
    authority: &'b str,
    calls: [Option<Call<'b>>; MAX_STREAMS],
    last_ping_ack: Option<[u8; 8]>,
    out: Outbound<'b>,
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> core::fmt::Debug
    for GrpcClient<'_, MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let active_calls = self.calls.iter().filter(|c| c.is_some()).count();
        f.debug_struct("GrpcClient")
            .field("conn", &self.conn)
            .field("scheme", &self.scheme)
            .field("authority", &self.authority)
            .field("active_calls", &active_calls)
            .field("out_len", &self.out.len())
            .finish()
    }
}

impl<'b, const MAX_STREAMS: usize, const REASM_CAP: usize> GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
    /// Creates a client sending `:scheme`/`:authority` and queueing outbound bytes in `tx_buf`.
    pub fn new(scheme: &'b str, authority: &'b str, tx_buf: &'b mut [u8]) -> Self {
        let mut conn = H2Connection::new(Role::Client);
        conn.set_auto_window_update(true);
        Self {
            conn,
            scheme,
            authority,
            calls: [const { None }; MAX_STREAMS],
            last_ping_ack: None,
            out: Outbound::new(tx_buf),
        }
    }

    /// Resets all HTTP/2 connection and stream state (dropping any uncollected in-flight calls)
    /// so this client instance and its transmit buffer can be reused for a new connection.
    pub fn reset(&mut self) {
        self.conn.reset();
        self.calls = [const { None }; MAX_STREAMS];
        self.last_ping_ack = None;
        self.out.clear();
    }

    /// Returns a shared reference to the underlying [`H2Connection`].
    #[must_use]
    #[inline]
    pub const fn connection(&self) -> &H2Connection<MAX_STREAMS, REASM_CAP> {
        &self.conn
    }

    /// Returns a mutable reference to the underlying [`H2Connection`].
    #[inline]
    pub fn connection_mut(&mut self) -> &mut H2Connection<MAX_STREAMS, REASM_CAP> {
        &mut self.conn
    }

    /// Queues an outbound HTTP/2 `PING` frame (`ack = false`) with the 8-byte `data` payload.
    pub fn send_ping(&mut self, data: [u8; 8]) -> Result<(), H2Error> {
        self.conn.queue_ping(false, data)?;
        self.out.flush(&mut self.conn);
        Ok(())
    }

    /// Takes the 8-byte opaque payload of the most recently received `PING` ACK frame, if any.
    #[must_use]
    #[inline]
    pub fn take_ping_ack(&mut self) -> Option<[u8; 8]> {
        self.last_ping_ack.take()
    }

    /// Queues an outbound HTTP/2 `GOAWAY` frame on stream `0`.
    pub fn send_goaway(
        &mut self,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.conn.queue_goaway(0, error_code, debug_data)?;
        self.out.flush(&mut self.conn);
        Ok(())
    }

    /// Returns `(last_stream_id, error_code)` if a `GOAWAY` frame has been received from the server.
    #[must_use]
    #[inline]
    pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
        self.conn.goaway_received()
    }

    /// Returns `true` if any call is currently awaiting its response.
    #[must_use]
    pub fn has_in_flight_calls(&self) -> bool {
        self.calls.iter().flatten().any(|c| c.done.is_none())
    }

    fn start_call_with_frames(
        &mut self,
        slot: usize,
        path: &str,
        buf: &'b mut [u8],
        start: usize,
        streaming_response: bool,
    ) -> Result<u32, Reply<'b>> {
        let buf_len = buf.len();
        self.out.flush(&mut self.conn);
        let headers_len =
            REQUEST_HEADERS_OVERHEAD + self.scheme.len() + self.authority.len() + path.len();
        let peer = self.conn.peer_settings();
        if headers_len > peer.max_frame_size as usize
            || FRAME_HEADER_LEN + headers_len > self.out.free()
        {
            return Err(Reply::failed(
                buf,
                GrpcCode::ResourceExhausted,
                "request headers exceed transmit buffer or max frame size",
            ));
        }

        let mut stream_id = 0;
        let (scheme, authority) = (self.scheme.as_bytes(), self.authority.as_bytes());
        let opened = self.out.push(&mut self.conn, |c, b| {
            let (id, n) = c.send_grpc_request_headers(
                scheme,
                authority,
                path.as_bytes(),
                GRPC_CONTENT_TYPE_ROTO,
                false,
                b,
            )?;
            stream_id = id;
            Ok(n)
        });
        if let Err(err) = opened {
            return Err(Reply::failed(buf, GrpcCode::Unavailable, err.reason));
        }
        let Ok((sent, req_done)) =
            self.out
                .send_partial_data(&mut self.conn, stream_id, &buf[start..buf_len], true)
        else {
            let _ = self
                .conn
                .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
            self.out.flush(&mut self.conn);
            return Err(Reply::failed(
                buf,
                GrpcCode::Internal,
                "failed to queue request data",
            ));
        };
        self.calls[slot] = Some(Call {
            stream_id,
            buf,
            req_offset: start + sent,
            req_end: buf_len,
            req_done,
            got_headers: false,
            streaming_response,
            msg_len: if streaming_response { Some(0) } else { None },
            done: None,
        });
        Ok(stream_id)
    }

    fn start_single_req_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_req: F,
        streaming_response: bool,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let Some(slot) = self.calls.iter().position(Option::is_none) else {
            return Err(Reply::failed(
                buf,
                GrpcCode::ResourceExhausted,
                "too many concurrent calls",
            ));
        };
        let buf_len = buf.len();
        let Ok(lpm) = encode_grpc_message(buf, false, encode_req) else {
            return Err(Reply::failed(
                buf,
                GrpcCode::ResourceExhausted,
                "request exceeds call buffer",
            ));
        };
        let start = buf_len - lpm.len();
        self.start_call_with_frames(slot, path, buf, start, streaming_response)
    }

    fn start_multi_req_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_reqs: F,
        streaming_response: bool,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        let Some(slot) = self.calls.iter().position(Option::is_none) else {
            return Err(Reply::failed(
                buf,
                GrpcCode::ResourceExhausted,
                "too many concurrent calls",
            ));
        };
        let buf_len = buf.len();
        let start = {
            let mut writer = BackwardWriter::new(buf);
            if let Err(status) = encode_reqs(&mut writer) {
                return Err(Reply::failed(buf, status.code, status.message));
            }
            let Ok(lpm_frames) = writer.finish() else {
                return Err(Reply::failed(
                    buf,
                    GrpcCode::ResourceExhausted,
                    "request exceeds call buffer",
                ));
            };
            buf_len - lpm_frames.len()
        };
        self.start_call_with_frames(slot, path, buf, start, streaming_response)
    }

    /// Encodes a unary request to `path` into `buf` and queues its `HEADERS` and initial `DATA`
    /// frames (any remaining request bytes stream automatically across `poll_transmit` and `recv`).
    ///
    /// Returns the call's stream ID, or `Err(Reply)` (handing `buf` back) if the call could not
    /// be started.
    pub fn start_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_req: F,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        self.start_single_req_call(path, buf, encode_req, false)
    }

    /// Frames a pre-serialized unary request payload `req_bytes` to `path` into `buf` and queues
    /// its `HEADERS` and initial `DATA` frames.
    ///
    /// Returns the call's stream ID, or `Err(Reply)` (handing `buf` back) if the call could not
    /// be started.
    #[inline]
    pub fn start_call_bytes(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        req_bytes: &[u8],
    ) -> Result<u32, Reply<'b>> {
        self.start_call(path, buf, |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Encodes a server-streaming request to `path` into `buf` and queues its `HEADERS` and
    /// initial `DATA` frames. Completed response messages are collected into `Reply::result()`
    /// as concatenated 5-byte LPM frames.
    pub fn start_server_streaming_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_req: F,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        self.start_single_req_call(path, buf, encode_req, true)
    }

    /// Frames a pre-serialized server-streaming request payload `req_bytes` to `path` into `buf`.
    #[inline]
    pub fn start_server_streaming_call_bytes(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        req_bytes: &[u8],
    ) -> Result<u32, Reply<'b>> {
        self.start_server_streaming_call(path, buf, |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Encodes 0..N client-streaming request frames to `path` into `buf` and queues `HEADERS`
    /// and initial `DATA` frames.
    pub fn start_client_streaming_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_reqs: F,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        self.start_multi_req_call(path, buf, encode_reqs, false)
    }

    /// Encodes 0..N bidirectional-streaming request frames to `path` into `buf` and queues
    /// `HEADERS` and initial `DATA` frames. Completed response messages are collected into
    /// `Reply::result()` as concatenated 5-byte LPM frames.
    pub fn start_bidi_streaming_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_reqs: F,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        self.start_multi_req_call(path, buf, encode_reqs, true)
    }

    fn drain_pending_requests(&mut self) {
        self.out.flush(&mut self.conn);
        for call in self.calls.iter_mut().flatten() {
            if call.req_done || call.done.is_some() {
                continue;
            }
            match self.out.send_partial_data(
                &mut self.conn,
                call.stream_id,
                &call.buf[call.req_offset..call.req_end],
                true,
            ) {
                Ok((sent, req_done)) => {
                    call.req_offset += sent;
                    call.req_done = req_done;
                }
                Err(err) => {
                    let _ = self
                        .conn
                        .queue_rst_stream(call.stream_id, H2ErrorCode::InternalError);
                    call.finish(err.code.to_grpc_code(), err.reason.as_bytes());
                }
            }
        }
        self.out.flush(&mut self.conn);
    }

    /// Consumes all of `input`, updating in-flight calls.
    ///
    /// Any [`H2Error`] is fatal for the connection; all in-flight calls are finished with
    /// [`GrpcCode::Unavailable`] so their buffers can be recovered via [`Self::take_reply`].
    pub fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        while !input.is_empty() {
            let (n, event) = match self.conn.recv(input) {
                Ok(res) => res,
                Err(err) => {
                    self.out.flush(&mut self.conn);
                    self.fail_all_in_flight(GrpcCode::Unavailable, b"connection error");
                    return Err(err);
                }
            };
            input = &input[n..];
            match event {
                Some(H2Event::Headers {
                    stream_id,
                    header_block,
                    end_stream,
                }) => {
                    if let Some(call) = find(&mut self.calls, stream_id) {
                        let was_done = call.done.is_some();
                        call.on_headers(header_block, end_stream);
                        if !was_done
                            && call.done.is_some()
                            && self.conn.stream_state(stream_id).is_active()
                        {
                            let _ = self.conn.queue_rst_stream(stream_id, H2ErrorCode::Cancel);
                        }
                    }
                }
                Some(H2Event::GrpcMessage {
                    stream_id,
                    compressed,
                    payload,
                    end_stream,
                }) => {
                    if let Some(call) = find(&mut self.calls, stream_id) {
                        let was_done = call.done.is_some();
                        call.on_message(compressed, payload);
                        if end_stream {
                            call.finish(GrpcCode::Internal, b"missing trailers");
                        }
                        if !was_done
                            && call.done.is_some()
                            && self.conn.stream_state(stream_id).is_active()
                        {
                            let _ = self.conn.queue_rst_stream(stream_id, H2ErrorCode::Cancel);
                        }
                    }
                }
                Some(H2Event::EndStream { stream_id }) => {
                    if let Some(call) = find(&mut self.calls, stream_id) {
                        let was_done = call.done.is_some();
                        call.finish(GrpcCode::Internal, b"missing trailers");
                        if !was_done && self.conn.stream_state(stream_id).is_active() {
                            let _ = self.conn.queue_rst_stream(stream_id, H2ErrorCode::Cancel);
                        }
                    }
                }
                Some(H2Event::RstStream {
                    stream_id,
                    error_code,
                }) => {
                    if let Some(call) = find(&mut self.calls, stream_id) {
                        call.finish(error_code.to_grpc_code(), b"stream reset by peer");
                    }
                }
                Some(H2Event::StreamError {
                    stream_id,
                    error_code,
                }) => {
                    if let Some(call) = find(&mut self.calls, stream_id) {
                        call.finish(error_code.to_grpc_code(), b"stream error");
                    }
                }
                Some(H2Event::GoAway { last_stream_id, .. }) => {
                    for call in self.calls.iter_mut().flatten() {
                        if call.stream_id > last_stream_id {
                            call.finish(GrpcCode::Unavailable, b"connection going away");
                        }
                    }
                }
                Some(H2Event::Ping { ack: true, data }) => {
                    self.last_ping_ack = Some(data);
                }
                Some(_) => {}
                None if n == 0 => {
                    self.drain_pending_requests();
                    break;
                }
                None => {}
            }
            self.drain_pending_requests();
        }
        Ok(())
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.drain_pending_requests();
        let n = self.out.poll_transmit(&mut self.conn, out);
        self.drain_pending_requests();
        n
    }

    /// Removes and returns the [`Reply`] of call `stream_id` once it has finished.
    #[must_use]
    pub fn take_reply(&mut self, stream_id: u32) -> Option<Reply<'b>> {
        let slot = self.calls.iter_mut().find(|c| {
            c.as_ref()
                .is_some_and(|c| c.stream_id == stream_id && c.done.is_some())
        })?;
        let call = slot.take()?;
        let (code, len) = call.done?;
        Some(Reply {
            code,
            len,
            buf: call.buf,
        })
    }

    pub(crate) fn fail_call(&mut self, stream_id: u32, code: GrpcCode, message: &[u8]) {
        if let Some(call) = find(&mut self.calls, stream_id) {
            call.finish(code, message);
        }
    }

    pub(crate) fn fail_all_in_flight(&mut self, code: GrpcCode, message: &[u8]) {
        for call in self.calls.iter_mut().flatten() {
            call.finish(code, message);
        }
    }
}

fn find<'c, 'b>(calls: &'c mut [Option<Call<'b>>], stream_id: u32) -> Option<&'c mut Call<'b>> {
    calls
        .iter_mut()
        .flatten()
        .find(|c| c.stream_id == stream_id)
}

/// In-memory [`UnaryChannel`] running every call through a [`GrpcClient`] and [`GrpcServer`]
/// pair over real HTTP/2 framing, so generated `<Service>Client<C>` stubs work end to end.
#[must_use]
pub struct H2LoopbackChannel<
    'b,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
    server: GrpcServer<'b, MAX_STREAMS, REASM_CAP>,
    buf: &'b mut [u8],
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> core::fmt::Debug
    for H2LoopbackChannel<'_, MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("H2LoopbackChannel")
            .field("client", &self.client)
            .field("server", &self.server)
            .field("buf_cap", &self.buf.len())
            .finish()
    }
}

impl<'b, const MAX_STREAMS: usize, const REASM_CAP: usize>
    H2LoopbackChannel<'b, MAX_STREAMS, REASM_CAP>
{
    /// Pairs `client` with `server`, using `call_buf` for each call's request and response.
    pub fn new(
        client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
        server: GrpcServer<'b, MAX_STREAMS, REASM_CAP>,
        call_buf: &'b mut [u8],
    ) -> Self {
        Self {
            client,
            server,
            buf: call_buf,
        }
    }

    /// Returns a shared reference to the underlying [`GrpcClient`].
    #[must_use]
    pub fn client(&self) -> &GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
        &self.client
    }

    /// Returns a mutable reference to the underlying [`GrpcClient`].
    pub fn client_mut(&mut self) -> &mut GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
        &mut self.client
    }

    /// Returns a shared reference to the underlying [`GrpcServer`].
    #[must_use]
    pub fn server(&self) -> &GrpcServer<'b, MAX_STREAMS, REASM_CAP> {
        &self.server
    }

    /// Returns a mutable reference to the underlying [`GrpcServer`].
    pub fn server_mut(&mut self) -> &mut GrpcServer<'b, MAX_STREAMS, REASM_CAP> {
        &mut self.server
    }

    /// Shuttles bytes between both endpoints until call `stream_id` finishes.
    fn pump(&mut self, stream_id: u32) -> Result<Reply<'b>, GrpcStatus<'static>> {
        let mut wire = [0u8; 1024];
        loop {
            if let Some(reply) = self.client.take_reply(stream_id) {
                return Ok(reply);
            }
            let n = self.client.poll_transmit(&mut wire);
            self.server.recv(&wire[..n])?;
            let m = self.server.poll_transmit(&mut wire);
            self.client.recv(&wire[..m])?;
            if n == 0 && m == 0 {
                return Err(GrpcStatus::internal("loopback connection stalled"));
            }
        }
    }
    fn finish_started<'a>(
        &'a mut self,
        started: Result<u32, Reply<'b>>,
    ) -> Result<&'a [u8], GrpcStatus<'a>> {
        let reply = match started {
            Ok(stream_id) => match self.pump(stream_id) {
                Ok(reply) => reply,
                Err(status) => {
                    self.client
                        .fail_call(stream_id, status.code, status.message.as_bytes());
                    if let Some(reply) = self.client.take_reply(stream_id) {
                        self.buf = reply.into_buf();
                    }
                    return Err(status);
                }
            },
            Err(reply) => reply,
        };
        self.buf = reply.buf;
        reply_result(reply.code, &self.buf[..reply.len])
    }
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> UnaryChannel
    for H2LoopbackChannel<'_, MAX_STREAMS, REASM_CAP>
{
    fn call_unary<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let buf = core::mem::take(&mut self.buf);
        let started = self.client.start_call(path, buf, encode_req);
        self.finish_started(started)
    }

    fn call_server_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let buf = core::mem::take(&mut self.buf);
        let started = self
            .client
            .start_server_streaming_call(path, buf, encode_req);
        self.finish_started(started)
    }

    fn call_client_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        let buf = core::mem::take(&mut self.buf);
        let started = self
            .client
            .start_client_streaming_call(path, buf, encode_reqs);
        self.finish_started(started)
    }

    fn call_bidi_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        let buf = core::mem::take(&mut self.buf);
        let started = self
            .client
            .start_bidi_streaming_call(path, buf, encode_reqs);
        self.finish_started(started)
    }
}
