//! Sans-I/O gRPC server driver: routes unary calls arriving on an [`H2Connection`] to
//! statically registered [`GrpcService`] implementations.

use roto_runtime::BackwardWriter;

use crate::driver::{send_partial_data_into, Outbound, HPACK_SCRATCH_LEN};
use crate::h2::{
    H2Connection, H2Error, H2ErrorCode, H2Event, Role, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP,
    FRAME_HEADER_LEN,
};
use crate::hpack::{
    is_grpc_content_type, GrpcRequestHeaders, HpackDecoder, GRPC_CONTENT_TYPE_ROTO,
};
use crate::service::{split_grpc_path, GrpcService};
use crate::status::GrpcStatus;

/// Maximum `:path` length buffered per in-flight call.
pub const MAX_PATH_LEN: usize = 256;

/// Upper bound on HPACK-encoded `:status: 200` + `content-type: application/grpc+roto`
/// response headers frame (`9`-byte header + `24`-byte HPACK block).
const RESPONSE_HEADERS_WIRE_LEN: usize = FRAME_HEADER_LEN + 24;

/// Upper bound on HPACK-encoded `grpc-status: 0` trailers frame (`9`-byte header + `15`-byte HPACK block).
const OK_TRAILERS_WIRE_LEN: usize = FRAME_HEADER_LEN + 15;

/// Object-safe view of a [`GrpcService`], allowing zero-allocation registration of
/// heterogeneous services as a `&[&dyn Route]` slice.
///
/// Blanket-implemented for every [`GrpcService`] (whose associated `SERVICE_NAME` constant
/// makes it non-dyn-compatible itself).
pub trait Route {
    /// Full protobuf service name (`"<package>.<Service>"`).
    fn service_name(&self) -> &'static str;

    /// Dispatches a unary call to `method_name`, writing the LPM-framed response onto `writer`.
    fn dispatch(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>>;
}

impl<T: GrpcService> Route for T {
    #[inline]
    fn service_name(&self) -> &'static str {
        T::SERVICE_NAME
    }

    #[inline]
    fn dispatch(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        self.call_unary(method_name, req_payload, writer)
    }
}

/// A request whose headers were accepted and whose message has not yet arrived.
#[derive(Clone, Copy)]
struct PendingCall {
    stream_id: u32,
    path_len: usize,
    path: [u8; MAX_PATH_LEN],
}

impl PendingCall {
    const FREE: Self = Self {
        stream_id: 0,
        path_len: 0,
        path: [0; MAX_PATH_LEN],
    };
}

/// An accepted unary response whose `HEADERS` were sent and whose remaining LPM payload
/// (staged at the tail of `tx_buf`) and `grpc-status: 0` trailers are awaiting flow-control
/// window credit or `tx_buf` space.
#[derive(Clone, Copy, Debug)]
struct PendingResponse {
    stream_id: u32,
    start: usize,
    end: usize,
}

impl PendingResponse {
    const FREE: Self = Self {
        stream_id: 0,
        start: 0,
        end: 0,
    };
}

/// `#![no_std]` zero-allocation Sans-I/O gRPC server over one HTTP/2 connection.
///
/// Feed inbound bytes with [`Self::recv`] and drain outbound bytes with [`Self::poll_transmit`].
/// Unary calls on any number of concurrent streams (up to `MAX_STREAMS`) are dispatched as soon
/// as their request message is complete.
///
/// # Design Constraints
///
/// - **Route registration**: `routes` is a `&[&dyn Route]` slice, supporting both static slice
///   literals (`&[&svc_a, &svc_b]`) and runtime-assembled slices without heap allocation.
/// - **Zero-allocation flow-control streaming**: Responses are encoded into `msg_buf` and
///   streamed as `DATA` frames across peer `WINDOW_UPDATE` frames and `tx_buf` drains,
///   staging any window-stalled response tail inside `tx_buf` so single responses or
///   concurrent response bursts exceeding the initial 65,535-byte HTTP/2 send window complete
///   without heap allocation.
/// - **Stateless HPACK**: Advertises `SETTINGS_HEADER_TABLE_SIZE = 0` and decodes header blocks
///   with a zero-capacity dynamic table.
#[must_use]
pub struct GrpcServer<
    'a,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    conn: H2Connection<MAX_STREAMS, REASM_CAP>,
    routes: &'a [&'a dyn Route],
    calls: [PendingCall; MAX_STREAMS],
    pending_resps: [PendingResponse; MAX_STREAMS],
    last_stream_id: u32,
    msg_buf: &'a mut [u8],
    out: Outbound<'a>,
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> core::fmt::Debug
    for GrpcServer<'_, MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GrpcServer")
            .field("conn", &self.conn)
            .field("routes_len", &self.routes.len())
            .field("last_stream_id", &self.last_stream_id)
            .field("msg_buf_cap", &self.msg_buf.len())
            .field("out_len", &self.out.len())
            .finish()
    }
}

impl<'a, const MAX_STREAMS: usize, const REASM_CAP: usize> GrpcServer<'a, MAX_STREAMS, REASM_CAP> {
    /// Creates a server dispatching to `routes`, encoding responses into `msg_buf`, and
    /// queueing outbound bytes in `tx_buf`.
    pub fn new(routes: &'a [&'a dyn Route], msg_buf: &'a mut [u8], tx_buf: &'a mut [u8]) -> Self {
        let mut conn = H2Connection::new(Role::Server);
        conn.set_auto_window_update(true);
        Self {
            conn,
            routes,
            calls: [PendingCall::FREE; MAX_STREAMS],
            pending_resps: [PendingResponse::FREE; MAX_STREAMS],
            last_stream_id: 0,
            msg_buf,
            out: Outbound::new(tx_buf),
        }
    }

    fn flush_conn(&mut self) {
        let limit = self.staging_floor();
        if self.out.len < limit {
            self.out.len += self
                .conn
                .poll_transmit(&mut self.out.buf[self.out.len..limit]);
        }
    }

    fn staging_floor(&self) -> usize {
        self.pending_resps
            .iter()
            .filter(|r| r.stream_id != 0 && r.start < r.end)
            .map(|r| r.start)
            .min()
            .unwrap_or(self.out.cap())
    }

    fn cancel_pending_response(&mut self, stream_id: u32) {
        for slot in &mut self.pending_resps {
            if slot.stream_id == stream_id {
                *slot = PendingResponse::FREE;
            }
        }
        self.compact_staging();
    }

    fn compact_staging(&mut self) {
        let mut write_end = self.out.cap();
        while let Some((idx, _)) = self
            .pending_resps
            .iter()
            .enumerate()
            .filter(|(_, r)| r.stream_id != 0 && r.end <= write_end)
            .max_by_key(|(_, r)| r.end)
        {
            let start = self.pending_resps[idx].start;
            let end = self.pending_resps[idx].end;
            let len = end - start;
            if end < write_end {
                if len > 0 {
                    self.out.buf.copy_within(start..end, write_end - len);
                }
                self.pending_resps[idx].start = write_end - len;
                self.pending_resps[idx].end = write_end;
            }
            write_end -= len;
        }
    }

    fn drain_pending_responses(&mut self) {
        self.flush_conn();
        let mut any_progressed = false;
        for i in 0..MAX_STREAMS {
            let stream_id = self.pending_resps[i].stream_id;
            if stream_id == 0 {
                continue;
            }
            let start = self.pending_resps[i].start;
            let end = self.pending_resps[i].end;
            if start < end {
                self.flush_conn();
                let floor = self.staging_floor();
                let (wire_part, stage_part) = self.out.buf.split_at_mut(floor);
                let staged = &stage_part[start - floor..end - floor];
                match send_partial_data_into(
                    &mut self.conn,
                    stream_id,
                    staged,
                    false,
                    wire_part,
                    &mut self.out.len,
                ) {
                    Ok((sent, _)) => {
                        if sent > 0 {
                            self.pending_resps[i].start += sent;
                            any_progressed = true;
                        }
                    }
                    Err(_) => {
                        self.pending_resps[i] = PendingResponse::FREE;
                        any_progressed = true;
                        continue;
                    }
                }
            }

            if self.pending_resps[i].stream_id != 0 && self.pending_resps[i].start == end {
                self.flush_conn();
                let floor = self.staging_floor();
                if floor.saturating_sub(self.out.len) >= OK_TRAILERS_WIRE_LEN {
                    let wire_part = &mut self.out.buf[self.out.len..floor];
                    if let Ok(n) =
                        self.conn
                            .send_grpc_trailers(stream_id, GrpcStatus::ok(), wire_part)
                    {
                        self.out.len += n;
                    } else {
                        let _ = self
                            .conn
                            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
                    }
                    self.pending_resps[i] = PendingResponse::FREE;
                    any_progressed = true;
                }
            }
        }
        if any_progressed {
            self.compact_staging();
        }
        self.flush_conn();
    }

    fn reject(&mut self, stream_id: u32, status: GrpcStatus<'_>) {
        self.flush_conn();
        let floor = self.staging_floor();
        let sent = if self.out.len < floor {
            self.conn
                .send_grpc_trailers_only(
                    stream_id,
                    status,
                    GRPC_CONTENT_TYPE_ROTO,
                    &mut self.out.buf[self.out.len..floor],
                )
                .map(|n| {
                    self.out.len += n;
                })
        } else {
            Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "transmit buffer full",
            ))
        };
        if sent.is_err() {
            let _ = self
                .conn
                .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
            self.flush_conn();
        }
    }

    /// Consumes all of `input`, dispatching every completed unary call.
    ///
    /// Any [`H2Error`] is fatal for the connection: the peer has already been sent `GOAWAY`
    /// via [`Self::poll_transmit`] and no further input should be fed.
    pub fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        while !input.is_empty() {
            let (n, event) = match self.conn.recv(input) {
                Ok(res) => res,
                Err(err) => {
                    self.flush_conn();
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
                    let status = if take(&mut self.calls, stream_id).is_some() {
                        GrpcStatus::internal("missing request message")
                    } else if stream_id <= self.last_stream_id {
                        self.drain_pending_responses();
                        continue; // Request trailers after the call was answered.
                    } else {
                        self.last_stream_id = stream_id;
                        match accept(&mut self.calls, stream_id, header_block, end_stream) {
                            Ok(()) => {
                                self.drain_pending_responses();
                                continue;
                            }
                            Err(status) => status,
                        }
                    };
                    self.reject(stream_id, status);
                }
                Some(H2Event::GrpcMessage {
                    stream_id,
                    compressed,
                    payload,
                    ..
                }) => {
                    let Some(call) = take(&mut self.calls, stream_id) else {
                        self.drain_pending_responses();
                        continue; // Extra message on an answered call.
                    };
                    let result = if compressed {
                        Err(GrpcStatus::internal("message compression not supported"))
                    } else {
                        match core::str::from_utf8(&call.path[..call.path_len]) {
                            Ok(path) => {
                                let cap = self.msg_buf.len();
                                dispatch(self.routes, path, payload, self.msg_buf)
                                    .map(|msg| (cap - msg.len())..cap)
                            }
                            Err(_) => Err(GrpcStatus::invalid_argument(":path is not UTF-8")),
                        }
                    };
                    match result {
                        Ok(range) => self.respond_from_msg_buf(stream_id, range),
                        Err(status) => self.reject(stream_id, status),
                    }
                }
                Some(H2Event::EndStream { stream_id }) => {
                    if take(&mut self.calls, stream_id).is_some() {
                        let status = GrpcStatus::internal("missing request message");
                        self.reject(stream_id, status);
                    }
                }
                Some(H2Event::RstStream { stream_id, .. })
                | Some(H2Event::StreamError { stream_id, .. }) => {
                    take(&mut self.calls, stream_id);
                    self.cancel_pending_response(stream_id);
                }
                Some(_) => {}
                None if n == 0 => {
                    self.drain_pending_responses();
                    break;
                }
                None => {}
            }
            self.drain_pending_responses();
        }
        Ok(())
    }

    fn respond_from_msg_buf(&mut self, stream_id: u32, range: core::ops::Range<usize>) {
        self.flush_conn();
        let floor = self.staging_floor();
        let msg = &self.msg_buf[range];
        let avail = floor.saturating_sub(self.out.len);
        let max_frame = self.conn.peer_settings().max_frame_size as usize;
        let full_wire = RESPONSE_HEADERS_WIRE_LEN
            + msg.len()
            + msg.len().div_ceil(max_frame) * FRAME_HEADER_LEN
            + OK_TRAILERS_WIRE_LEN;
        let stream_win = self.conn.stream_send_window(stream_id).unwrap_or(0);
        let win = self.conn.conn_send_window().min(stream_win).max(0) as usize;

        if full_wire <= self.out.cap() {
            // Fast path: full response fits in current send windows and available wire region.
            if win >= msg.len() && avail >= full_wire {
                if let Ok(hdr_n) = self.conn.send_grpc_response_headers(
                    stream_id,
                    GRPC_CONTENT_TYPE_ROTO,
                    &mut self.out.buf[self.out.len..floor],
                ) {
                    self.out.len += hdr_n;
                    let data_ok = send_partial_data_into(
                        &mut self.conn,
                        stream_id,
                        msg,
                        false,
                        &mut self.out.buf[..floor],
                        &mut self.out.len,
                    )
                    .is_ok_and(|(_, done)| done);
                    let tr_ok = data_ok
                        && self
                            .conn
                            .send_grpc_trailers(
                                stream_id,
                                GrpcStatus::ok(),
                                &mut self.out.buf[self.out.len..floor],
                            )
                            .map(|tr_n| {
                                self.out.len += tr_n;
                            })
                            .is_ok();
                    if !tr_ok {
                        let _ = self
                            .conn
                            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
                        self.flush_conn();
                    }
                    return;
                }
            } else if avail >= RESPONSE_HEADERS_WIRE_LEN + msg.len() {
                // Staged path: validate slot and staging space upfront before mutating H2 state.
                if let Some(slot) = self.pending_resps.iter().position(|r| r.stream_id == 0) {
                    if let Ok(hdr_n) = self.conn.send_grpc_response_headers(
                        stream_id,
                        GRPC_CONTENT_TYPE_ROTO,
                        &mut self.out.buf[self.out.len..floor],
                    ) {
                        self.out.len += hdr_n;
                        let stage_start = floor - msg.len();
                        self.out.buf[stage_start..floor].copy_from_slice(msg);
                        self.pending_resps[slot] = PendingResponse {
                            stream_id,
                            start: stage_start,
                            end: floor,
                        };
                        self.drain_pending_responses();
                        return;
                    }
                }
            }
        }
        let status =
            GrpcStatus::resource_exhausted("response exceeds transmit buffer or send window");
        self.reject(stream_id, status);
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.drain_pending_responses();
        let n = self.out.drain(out);
        self.drain_pending_responses();
        n
    }
}

/// Removes and returns the pending call on `stream_id`, if any.
fn take(calls: &mut [PendingCall], stream_id: u32) -> Option<PendingCall> {
    let call = calls.iter_mut().find(|c| c.stream_id == stream_id)?;
    Some(core::mem::replace(call, PendingCall::FREE))
}

/// Validates request headers and records the call until its message arrives.
fn accept(
    calls: &mut [PendingCall],
    stream_id: u32,
    header_block: &[u8],
    end_stream: bool,
) -> Result<(), GrpcStatus<'static>> {
    let mut scratch = [0u8; HPACK_SCRATCH_LEN];
    let mut decoder = HpackDecoder::<0>::stateless();
    let headers = GrpcRequestHeaders::decode(&mut decoder, header_block, &mut scratch)?;
    if headers.method != Some(b"POST") {
        return Err(GrpcStatus::invalid_argument(":method must be POST"));
    }
    if headers.te != Some(b"trailers") {
        return Err(GrpcStatus::invalid_argument("te must be trailers"));
    }
    if !headers.content_type.is_some_and(is_grpc_content_type) {
        return Err(GrpcStatus::invalid_argument(
            "content-type must be application/grpc",
        ));
    }
    let Some(path) = headers.path else {
        return Err(GrpcStatus::invalid_argument("missing :path"));
    };
    if end_stream {
        return Err(GrpcStatus::internal("missing request message"));
    }
    if path.len() > MAX_PATH_LEN {
        return Err(GrpcStatus::unimplemented(":path too long"));
    }
    let Some(call) = calls.iter_mut().find(|c| c.stream_id == 0) else {
        return Err(GrpcStatus::resource_exhausted("too many concurrent calls"));
    };
    call.stream_id = stream_id;
    call.path_len = path.len();
    call.path[..path.len()].copy_from_slice(path);
    Ok(())
}

/// Routes `path` to its service and returns the LPM-framed response encoded into `msg_buf`.
fn dispatch<'m>(
    routes: &[&dyn Route],
    path: &str,
    payload: &[u8],
    msg_buf: &'m mut [u8],
) -> Result<&'m [u8], GrpcStatus<'static>> {
    let Some((service, method)) = split_grpc_path(path) else {
        return Err(GrpcStatus::unimplemented("malformed gRPC path"));
    };
    let Some(route) = routes.iter().find(|r| r.service_name() == service) else {
        return Err(GrpcStatus::unimplemented("unknown gRPC service"));
    };
    let mut writer = BackwardWriter::new(msg_buf);
    route.dispatch(method, payload, &mut writer)?;
    Ok(writer.finish()?)
}

