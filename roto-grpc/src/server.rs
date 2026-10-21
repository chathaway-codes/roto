//! Sans-I/O gRPC server driver: routes unary calls arriving on an [`H2Connection`] to
//! statically registered [`GrpcService`] implementations.

use roto_runtime::BackwardWriter;

use crate::driver::{Outbound, HPACK_SCRATCH_LEN};
use crate::h2::{
    H2Connection, H2Error, H2ErrorCode, H2Event, Role, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP,
};
use crate::hpack::{
    is_grpc_content_type, GrpcRequestHeaders, HpackDecoder, GRPC_CONTENT_TYPE_ROTO,
};
use crate::service::{split_grpc_path, GrpcService};
use crate::status::GrpcStatus;

/// Maximum `:path` length buffered per in-flight call.
pub const MAX_PATH_LEN: usize = 256;

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
/// - **Whole-message send window**: Responses are encoded into `msg_buf` and queued into `tx_buf`
///   immediately; each outbound response must fit in `tx_buf` and within the peer's available
///   HTTP/2 send window (streaming outbound flow-control resumption is tracked in `#035`).
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
            last_stream_id: 0,
            msg_buf,
            out: Outbound::new(tx_buf),
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
                    self.out.flush(&mut self.conn);
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
                        self.out.flush(&mut self.conn);
                        continue; // Request trailers after the call was answered.
                    } else {
                        self.last_stream_id = stream_id;
                        match accept(&mut self.calls, stream_id, header_block, end_stream) {
                            Ok(()) => {
                                self.out.flush(&mut self.conn);
                                continue;
                            }
                            Err(status) => status,
                        }
                    };
                    reject(&mut self.conn, &mut self.out, stream_id, status);
                }
                Some(H2Event::GrpcMessage {
                    stream_id,
                    compressed,
                    payload,
                    ..
                }) => {
                    let Some(call) = take(&mut self.calls, stream_id) else {
                        self.out.flush(&mut self.conn);
                        continue; // Extra message on an answered call.
                    };
                    let result = if compressed {
                        Err(GrpcStatus::internal("message compression not supported"))
                    } else {
                        match core::str::from_utf8(&call.path[..call.path_len]) {
                            Ok(path) => dispatch(self.routes, path, payload, self.msg_buf),
                            Err(_) => Err(GrpcStatus::invalid_argument(":path is not UTF-8")),
                        }
                    };
                    respond(&mut self.conn, &mut self.out, stream_id, result);
                }
                Some(H2Event::EndStream { stream_id }) => {
                    if take(&mut self.calls, stream_id).is_some() {
                        let status = GrpcStatus::internal("missing request message");
                        reject(&mut self.conn, &mut self.out, stream_id, status);
                    }
                }
                Some(H2Event::RstStream { stream_id, .. })
                | Some(H2Event::StreamError { stream_id, .. }) => {
                    take(&mut self.calls, stream_id);
                }
                Some(_) => {}
                None if n == 0 => {
                    self.out.flush(&mut self.conn);
                    break;
                }
                None => {}
            }
            self.out.flush(&mut self.conn);
        }
        Ok(())
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.out.poll_transmit(&mut self.conn, out)
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

/// Queues a full response (`HEADERS`, `DATA`, trailers) or a Trailers-Only error.
fn respond<const M: usize, const R: usize>(
    conn: &mut H2Connection<M, R>,
    out: &mut Outbound<'_>,
    stream_id: u32,
    result: Result<&[u8], GrpcStatus<'_>>,
) {
    let status = match result {
        Ok(msg) => {
            out.flush(conn);
            let mark = out.len();
            let sent = out
                .push(conn, |c, b| {
                    c.send_grpc_response_headers(stream_id, GRPC_CONTENT_TYPE_ROTO, b)
                })
                .is_ok()
                && out.send_data(conn, stream_id, msg, false).is_ok();
            if sent {
                let trailers = out.push(conn, |c, b| {
                    c.send_grpc_trailers(stream_id, GrpcStatus::ok(), b)
                });
                if trailers.is_err() {
                    let _ = conn.queue_rst_stream(stream_id, H2ErrorCode::InternalError);
                    out.flush(conn);
                }
                return;
            }
            out.truncate(mark);
            GrpcStatus::resource_exhausted("response exceeds transmit buffer or send window")
        }
        Err(status) => status,
    };
    reject(conn, out, stream_id, status);
}

/// Queues a Trailers-Only response, falling back to `RST_STREAM` if it does not fit.
fn reject<const M: usize, const R: usize>(
    conn: &mut H2Connection<M, R>,
    out: &mut Outbound<'_>,
    stream_id: u32,
    status: GrpcStatus<'_>,
) {
    let sent = out.push(conn, |c, b| {
        c.send_grpc_trailers_only(stream_id, status, GRPC_CONTENT_TYPE_ROTO, b)
    });
    if sent.is_err() {
        let _ = conn.queue_rst_stream(stream_id, H2ErrorCode::InternalError);
        out.flush(conn);
    }
}
