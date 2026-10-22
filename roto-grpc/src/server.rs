//! Sans-I/O gRPC server drivers: routes unary calls arriving on an [`H2Connection`] to
//! statically registered [`GrpcService`] or [`AsyncGrpcService`] implementations.

use core::future::Future;

use roto_runtime::BackwardWriter;

use crate::driver::{send_partial_data_into, Outbound, HPACK_SCRATCH_LEN};
use crate::h2::{
    H2Connection, H2Error, H2ErrorCode, H2Event, Role, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP,
    FRAME_HEADER_LEN,
};
use crate::hpack::{
    is_grpc_content_type, GrpcRequestHeaders, HpackDecoder, GRPC_CONTENT_TYPE,
    GRPC_CONTENT_TYPE_ROTO,
};
use crate::service::{split_grpc_path, AsyncGrpcService, GrpcService};
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
pub trait Route: Sync {
    /// Full protobuf service name (`"<package>.<Service>"`).
    fn service_name(&self) -> &'static str;

    /// Wire-encoded `FileDescriptorProto` slices for this service and its transitive dependencies,
    /// when generated with `descriptors=true`.
    #[inline]
    fn file_descriptors(&self) -> &'static [&'static [u8]] {
        &[]
    }

    /// Returns `true` if `method_name` is a client-streaming or bidirectional-streaming RPC.
    #[inline]
    fn is_client_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Returns `true` if `method_name` is a bidirectional-streaming RPC.
    #[inline]
    fn is_bidi_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Dispatches a unary or server-streaming call to `method_name`, writing the LPM-framed
    /// response(s) onto `writer`.
    fn dispatch(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>>;

    /// Dispatches a client-streaming or bidirectional-streaming call to `method_name`, writing
    /// the LPM-framed response(s) onto `writer`.
    #[inline]
    fn dispatch_stream(
        &self,
        method_name: &str,
        req_frames: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let _ = (method_name, req_frames, writer);
        Err(GrpcStatus::unimplemented("unimplemented gRPC method"))
    }
}

impl<T: GrpcService + Sync> Route for T {
    #[inline]
    fn service_name(&self) -> &'static str {
        T::SERVICE_NAME
    }

    #[inline]
    fn file_descriptors(&self) -> &'static [&'static [u8]] {
        T::FILE_DESCRIPTOR_SET
    }

    #[inline]
    fn is_client_streaming(&self, method_name: &str) -> bool {
        <T as GrpcService>::is_client_streaming(self, method_name)
    }

    #[inline]
    fn is_bidi_streaming(&self, method_name: &str) -> bool {
        <T as GrpcService>::is_bidi_streaming(self, method_name)
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

    #[inline]
    fn dispatch_stream(
        &self,
        method_name: &str,
        req_frames: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        self.call_stream(method_name, req_frames, writer)
    }
}

/// Zero-allocation asynchronous service router trait for [`AsyncGrpcServer`].
///
/// Blanket-implemented for every [`AsyncGrpcService`] as well as tuples of up to 4
/// [`AsyncGrpcService`] implementations `(S1, S2, ...)` so multiple async services can
/// be served over a single HTTP/2 connection without heap allocation.
pub trait AsyncRouter {
    /// Returns `true` if `(service_name, method_name)` is a client-streaming or
    /// bidirectional-streaming RPC.
    #[inline]
    fn is_client_streaming(&self, service_name: &str, method_name: &str) -> bool {
        let _ = (service_name, method_name);
        false
    }

    /// Returns `true` if `(service_name, method_name)` is a bidirectional-streaming RPC.
    #[inline]
    fn is_bidi_streaming(&self, service_name: &str, method_name: &str) -> bool {
        let _ = (service_name, method_name);
        false
    }

    /// Asynchronously dispatches a unary or server-streaming call to `(service_name, method_name)`,
    /// writing the LPM-framed response(s) onto `writer`.
    fn dispatch_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a;

    /// Asynchronously dispatches a client-streaming or bidirectional-streaming call to
    /// `(service_name, method_name)`, writing the LPM-framed response(s) onto `writer`.
    #[inline]
    fn dispatch_stream_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a {
        async move {
            let _ = (service_name, method_name, req_frames, writer);
            Err(GrpcStatus::unimplemented("unimplemented gRPC method"))
        }
    }
}

impl<S: AsyncGrpcService> AsyncRouter for S {
    #[inline]
    fn is_client_streaming(&self, service_name: &str, method_name: &str) -> bool {
        service_name == S::SERVICE_NAME
            && <S as AsyncGrpcService>::is_client_streaming(self, method_name)
    }

    #[inline]
    fn is_bidi_streaming(&self, service_name: &str, method_name: &str) -> bool {
        service_name == S::SERVICE_NAME
            && <S as AsyncGrpcService>::is_bidi_streaming(self, method_name)
    }

    #[inline]
    async fn dispatch_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name != S::SERVICE_NAME {
            return Err(GrpcStatus::unimplemented("unknown gRPC service"));
        }
        self.call_unary_async(method_name, req_payload, writer)
            .await
    }

    #[inline]
    async fn dispatch_stream_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name != S::SERVICE_NAME {
            return Err(GrpcStatus::unimplemented("unknown gRPC service"));
        }
        self.call_stream_async(method_name, req_frames, writer)
            .await
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService> AsyncRouter for (A, B) {
    #[inline]
    fn is_client_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_client_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_client_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    fn is_bidi_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_bidi_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_bidi_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    async fn dispatch_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }

    #[inline]
    async fn dispatch_stream_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService, C: AsyncGrpcService> AsyncRouter for (A, B, C) {
    #[inline]
    fn is_client_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_client_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_client_streaming(method_name)
        } else if service_name == C::SERVICE_NAME {
            self.2.is_client_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    fn is_bidi_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_bidi_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_bidi_streaming(method_name)
        } else if service_name == C::SERVICE_NAME {
            self.2.is_bidi_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    async fn dispatch_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == C::SERVICE_NAME {
            self.2
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }

    #[inline]
    async fn dispatch_stream_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == C::SERVICE_NAME {
            self.2
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService, C: AsyncGrpcService, D: AsyncGrpcService> AsyncRouter
    for (A, B, C, D)
{
    #[inline]
    fn is_client_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_client_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_client_streaming(method_name)
        } else if service_name == C::SERVICE_NAME {
            self.2.is_client_streaming(method_name)
        } else if service_name == D::SERVICE_NAME {
            self.3.is_client_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    fn is_bidi_streaming(&self, service_name: &str, method_name: &str) -> bool {
        if service_name == A::SERVICE_NAME {
            self.0.is_bidi_streaming(method_name)
        } else if service_name == B::SERVICE_NAME {
            self.1.is_bidi_streaming(method_name)
        } else if service_name == C::SERVICE_NAME {
            self.2.is_bidi_streaming(method_name)
        } else if service_name == D::SERVICE_NAME {
            self.3.is_bidi_streaming(method_name)
        } else {
            false
        }
    }

    #[inline]
    async fn dispatch_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == C::SERVICE_NAME {
            self.2
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else if service_name == D::SERVICE_NAME {
            self.3
                .call_unary_async(method_name, req_payload, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }

    #[inline]
    async fn dispatch_stream_async<'a, 'b>(
        &'a self,
        service_name: &'a str,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        if service_name == A::SERVICE_NAME {
            self.0
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == B::SERVICE_NAME {
            self.1
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == C::SERVICE_NAME {
            self.2
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else if service_name == D::SERVICE_NAME {
            self.3
                .call_stream_async(method_name, req_frames, writer)
                .await
        } else {
            Err(GrpcStatus::unimplemented("unknown gRPC service"))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallMode {
    SingleRequest,
    ClientStream,
    BidiStream,
}

/// A request whose headers were accepted and whose message(s) have not yet completed.
#[derive(Clone, Copy)]
struct PendingCall<const REASM_CAP: usize = DEFAULT_REASM_CAP> {
    stream_id: u32,
    path_len: usize,
    path: [u8; MAX_PATH_LEN],
    mode: CallMode,
    headers_sent: bool,
    roto_content_type: bool,
    req_buf: [u8; REASM_CAP],
    req_len: usize,
}

impl<const REASM_CAP: usize> PendingCall<REASM_CAP> {
    const FREE: Self = Self {
        stream_id: 0,
        path_len: 0,
        path: [0; MAX_PATH_LEN],
        mode: CallMode::SingleRequest,
        headers_sent: false,
        roto_content_type: true,
        req_buf: [0; REASM_CAP],
        req_len: 0,
    };

    fn append_lpm_frame(&mut self, payload: &[u8]) -> Result<(), GrpcStatus<'static>> {
        let Ok(len_u32) = u32::try_from(payload.len()) else {
            return Err(GrpcStatus::resource_exhausted(
                "streaming request message too large",
            ));
        };
        let frame_len = crate::framing::GRPC_HEADER_LEN + payload.len();
        if self.req_len + frame_len > REASM_CAP {
            return Err(GrpcStatus::resource_exhausted(
                "streaming request exceeds reassembly capacity",
            ));
        }
        let len_be = len_u32.to_be_bytes();
        self.req_buf[self.req_len..self.req_len + 5]
            .copy_from_slice(&[0, len_be[0], len_be[1], len_be[2], len_be[3]]);
        self.req_buf[self.req_len + 5..self.req_len + frame_len].copy_from_slice(payload);
        self.req_len += frame_len;
        Ok(())
    }
}

/// An accepted response whose `HEADERS` were sent and whose remaining LPM payload
/// (staged at the tail of `tx_buf`) and optional `grpc-status: 0` trailers are awaiting
/// flow-control window credit or `tx_buf` space.
#[derive(Clone, Copy, Debug)]
struct PendingResponse {
    stream_id: u32,
    start: usize,
    end: usize,
    send_trailers: bool,
}

impl PendingResponse {
    const FREE: Self = Self {
        stream_id: 0,
        start: 0,
        end: 0,
        send_trailers: false,
    };
}

struct ServerCore<
    'a,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    conn: H2Connection<MAX_STREAMS, REASM_CAP>,
    calls: [PendingCall<REASM_CAP>; MAX_STREAMS],
    pending_resps: [PendingResponse; MAX_STREAMS],
    last_stream_id: u32,
    goaway_limit: Option<u32>,
    last_ping_ack: Option<[u8; 8]>,
    active_connection: bool,
    msg_buf: &'a mut [u8],
    out: Outbound<'a>,
}

impl<'a, const MAX_STREAMS: usize, const REASM_CAP: usize> ServerCore<'a, MAX_STREAMS, REASM_CAP> {
    fn new(msg_buf: &'a mut [u8], tx_buf: &'a mut [u8]) -> Self {
        let mut conn = H2Connection::new(Role::Server);
        conn.set_auto_window_update(true);
        Self {
            conn,
            calls: [PendingCall::FREE; MAX_STREAMS],
            pending_resps: [PendingResponse::FREE; MAX_STREAMS],
            last_stream_id: 0,
            goaway_limit: None,
            last_ping_ack: None,
            active_connection: false,
            msg_buf,
            out: Outbound::new(tx_buf),
        }
    }

    fn reset(&mut self) {
        self.conn.reset();
        self.calls = [PendingCall::FREE; MAX_STREAMS];
        self.pending_resps = [PendingResponse::FREE; MAX_STREAMS];
        self.last_stream_id = 0;
        self.goaway_limit = None;
        self.last_ping_ack = None;
        self.active_connection = false;
        self.out.clear();
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
            .filter(|(_, r)| r.stream_id != 0 && r.start < r.end && r.end <= write_end)
            .max_by_key(|(_, r)| r.end)
        {
            let start = self.pending_resps[idx].start;
            let end = self.pending_resps[idx].end;
            let len = end - start;
            if end < write_end {
                self.out.buf.copy_within(start..end, write_end - len);
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
                if !self.pending_resps[i].send_trailers {
                    self.pending_resps[i] = PendingResponse::FREE;
                    any_progressed = true;
                    continue;
                }
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

    fn reject_stream(
        &mut self,
        stream_id: u32,
        headers_sent: bool,
        roto_content_type: bool,
        status: GrpcStatus<'_>,
    ) {
        take(&mut self.calls, stream_id);
        self.cancel_pending_response(stream_id);
        self.flush_conn();
        let floor = self.staging_floor();
        let content_type = if roto_content_type {
            GRPC_CONTENT_TYPE_ROTO
        } else {
            GRPC_CONTENT_TYPE
        };
        let sent = if self.out.len < floor {
            if headers_sent {
                self.conn.send_grpc_trailers(
                    stream_id,
                    status,
                    &mut self.out.buf[self.out.len..floor],
                )
            } else {
                self.conn.send_grpc_trailers_only(
                    stream_id,
                    status,
                    content_type,
                    &mut self.out.buf[self.out.len..floor],
                )
            }
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

    fn finish_bidi_stream(&mut self, stream_id: u32) {
        self.drain_pending_responses();
        if let Some(slot) = self
            .pending_resps
            .iter_mut()
            .find(|r| r.stream_id == stream_id)
        {
            slot.send_trailers = true;
            self.drain_pending_responses();
            return;
        }
        self.flush_conn();
        let floor = self.staging_floor();
        if floor.saturating_sub(self.out.len) >= OK_TRAILERS_WIRE_LEN {
            if let Ok(tr_n) = self.conn.send_grpc_trailers(
                stream_id,
                GrpcStatus::ok(),
                &mut self.out.buf[self.out.len..floor],
            ) {
                self.out.len += tr_n;
                self.flush_conn();
                return;
            }
        } else if let Some(slot) = self.pending_resps.iter_mut().find(|r| r.stream_id == 0) {
            *slot = PendingResponse {
                stream_id,
                start: floor,
                end: floor,
                send_trailers: true,
            };
            self.drain_pending_responses();
            return;
        }
        let _ = self
            .conn
            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
        self.flush_conn();
    }

    fn on_headers(&mut self, stream_id: u32, accepted: Option<Result<(), GrpcStatus<'static>>>) {
        if self.goaway_limit.is_some_and(|limit| stream_id > limit) {
            take(&mut self.calls, stream_id);
            let _ = self
                .conn
                .queue_rst_stream(stream_id, H2ErrorCode::RefusedStream);
            self.drain_pending_responses();
            return;
        }
        match accepted {
            None => self.drain_pending_responses(),
            Some(Ok(())) => self.drain_pending_responses(),
            Some(Err(status)) => {
                self.reject_stream(stream_id, false, true, status);
                self.drain_pending_responses();
            }
        }
    }

    fn respond_from_msg_buf(
        &mut self,
        stream_id: u32,
        range: core::ops::Range<usize>,
        send_headers: bool,
        send_trailers: bool,
        roto_content_type: bool,
    ) {
        self.flush_conn();
        let floor = self.staging_floor();
        let msg = &self.msg_buf[range];
        let avail = floor.saturating_sub(self.out.len);
        let max_frame = self.conn.peer_settings().max_frame_size as usize;
        let hdr_wire = if send_headers {
            RESPONSE_HEADERS_WIRE_LEN
        } else {
            0
        };
        let tr_wire = if send_trailers {
            OK_TRAILERS_WIRE_LEN
        } else {
            0
        };
        let data_wire = if msg.is_empty() {
            0
        } else {
            msg.len() + msg.len().div_ceil(max_frame) * FRAME_HEADER_LEN
        };
        let full_wire = hdr_wire + data_wire + tr_wire;
        let stream_win = self.conn.stream_send_window(stream_id).unwrap_or(0);
        let win = self.conn.conn_send_window().min(stream_win).max(0) as usize;
        let content_type = if roto_content_type {
            GRPC_CONTENT_TYPE_ROTO
        } else {
            GRPC_CONTENT_TYPE
        };
        let has_staged = self.pending_resps.iter().any(|r| r.stream_id == stream_id);

        if full_wire <= self.out.cap() {
            // Fast path: full response fits in current send windows and available wire region.
            if !has_staged && win >= msg.len() && avail >= full_wire {
                let hdr_ok = if send_headers {
                    if let Ok(hdr_n) = self.conn.send_grpc_response_headers(
                        stream_id,
                        content_type,
                        &mut self.out.buf[self.out.len..floor],
                    ) {
                        self.out.len += hdr_n;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                };
                if hdr_ok {
                    let data_ok = send_partial_data_into(
                        &mut self.conn,
                        stream_id,
                        msg,
                        false,
                        &mut self.out.buf[..floor],
                        &mut self.out.len,
                    )
                    .is_ok_and(|(_, done)| done);
                    let tr_ok = if send_trailers {
                        data_ok
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
                                .is_ok()
                    } else {
                        data_ok
                    };
                    if !tr_ok {
                        let _ = self
                            .conn
                            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
                        self.flush_conn();
                    }
                    return;
                }
            } else if avail >= hdr_wire + msg.len() {
                if let Some(existing_idx) = self
                    .pending_resps
                    .iter()
                    .position(|r| r.stream_id == stream_id)
                {
                    if !msg.is_empty() {
                        let old_start = self.pending_resps[existing_idx].start;
                        let old_end = self.pending_resps[existing_idx].end;
                        let msg_len = msg.len();
                        self.out.buf.copy_within(floor..old_end, floor - msg_len);
                        self.out.buf[old_end - msg_len..old_end].copy_from_slice(msg);
                        for r in &mut self.pending_resps {
                            if r.stream_id != 0 && r.start < r.end && r.end <= old_start {
                                r.start -= msg_len;
                                r.end -= msg_len;
                            }
                        }
                        self.pending_resps[existing_idx].start -= msg_len;
                    }
                    self.pending_resps[existing_idx].send_trailers |= send_trailers;
                    self.drain_pending_responses();
                    return;
                } else if let Some(slot) = self.pending_resps.iter().position(|r| r.stream_id == 0)
                {
                    // Staged path: validate slot and staging space upfront before mutating H2 state.
                    let hdr_ok = if send_headers {
                        if let Ok(hdr_n) = self.conn.send_grpc_response_headers(
                            stream_id,
                            content_type,
                            &mut self.out.buf[self.out.len..floor],
                        ) {
                            self.out.len += hdr_n;
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    };
                    if hdr_ok {
                        let stage_start = floor - msg.len();
                        self.out.buf[stage_start..floor].copy_from_slice(msg);
                        self.pending_resps[slot] = PendingResponse {
                            stream_id,
                            start: stage_start,
                            end: floor,
                            send_trailers,
                        };
                        self.drain_pending_responses();
                        return;
                    }
                }
            }
        }
        let status =
            GrpcStatus::resource_exhausted("response exceeds transmit buffer or send window");
        self.reject_stream(stream_id, !send_headers, roto_content_type, status);
    }

    fn send_ping(&mut self, data: [u8; 8]) -> Result<(), H2Error> {
        self.conn.queue_ping(false, data)?;
        self.flush_conn();
        Ok(())
    }

    fn send_goaway_at(
        &mut self,
        last_stream_id: u32,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.goaway_limit = Some(last_stream_id);
        for call in &mut self.calls {
            if call.stream_id > last_stream_id {
                let sid = call.stream_id;
                *call = PendingCall::FREE;
                let _ = self.conn.queue_rst_stream(sid, H2ErrorCode::RefusedStream);
            }
        }
        self.conn
            .queue_goaway(last_stream_id, error_code, debug_data)?;
        self.flush_conn();
        Ok(())
    }

    fn has_pending_work(&self) -> bool {
        self.out.len() > 0
            || self.conn.has_pending_transmit()
            || self.conn.has_partial_inbound_frame()
            || self.calls.iter().any(|c| c.stream_id != 0)
            || self.pending_resps.iter().any(|r| r.stream_id != 0)
    }

    fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.drain_pending_responses();
        let n = self.out.drain(out);
        if n > 0 {
            self.active_connection = true;
        }
        self.drain_pending_responses();
        n
    }

    fn finish_message(
        &mut self,
        stream_id: u32,
        roto_content_type: bool,
        result: Result<core::ops::Range<usize>, GrpcStatus<'static>>,
    ) {
        match result {
            Ok(range) => {
                self.respond_from_msg_buf(stream_id, range, true, true, roto_content_type);
            }
            Err(status) => self.reject_stream(stream_id, false, roto_content_type, status),
        }
        self.drain_pending_responses();
    }

    fn finish_bidi_message(
        &mut self,
        stream_id: u32,
        send_headers: bool,
        send_trailers: bool,
        roto_content_type: bool,
        result: Result<core::ops::Range<usize>, GrpcStatus<'static>>,
    ) {
        match result {
            Ok(range) => {
                self.respond_from_msg_buf(
                    stream_id,
                    range,
                    send_headers,
                    send_trailers,
                    roto_content_type,
                );
            }
            Err(status) => {
                self.reject_stream(stream_id, !send_headers, roto_content_type, status);
            }
        }
        self.drain_pending_responses();
    }

    fn apply_post_event(&mut self, post: PostEvent) -> bool {
        match post {
            PostEvent::Headers {
                stream_id,
                accepted,
            } => {
                self.on_headers(stream_id, accepted);
                true
            }
            PostEvent::Reject {
                stream_id,
                headers_sent,
                roto_content_type,
                status,
            } => {
                self.reject_stream(stream_id, headers_sent, roto_content_type, status);
                self.drain_pending_responses();
                true
            }
            PostEvent::FinishBidiStream { stream_id } => {
                self.finish_bidi_stream(stream_id);
                true
            }
            PostEvent::CancelStream { stream_id } => {
                take(&mut self.calls, stream_id);
                self.cancel_pending_response(stream_id);
                self.drain_pending_responses();
                true
            }
            PostEvent::PingAck(data) => {
                self.last_ping_ack = Some(data);
                self.drain_pending_responses();
                true
            }
            PostEvent::Other => {
                self.drain_pending_responses();
                true
            }
            PostEvent::Idle => {
                self.drain_pending_responses();
                false
            }
        }
    }
}

enum PreDispatch<'p, const REASM_CAP: usize> {
    DispatchUnary {
        call: PendingCall<REASM_CAP>,
        compressed: bool,
        payload: &'p [u8],
    },
    DispatchClientStream {
        call: PendingCall<REASM_CAP>,
    },
    DispatchBidiMessage {
        call: PendingCall<REASM_CAP>,
        send_headers: bool,
        send_trailers: bool,
    },
    Other(PostEvent),
}

enum PostEvent {
    Headers {
        stream_id: u32,
        accepted: Option<Result<(), GrpcStatus<'static>>>,
    },
    Reject {
        stream_id: u32,
        headers_sent: bool,
        roto_content_type: bool,
        status: GrpcStatus<'static>,
    },
    FinishBidiStream {
        stream_id: u32,
    },
    CancelStream {
        stream_id: u32,
    },
    PingAck([u8; 8]),
    Other,
    Idle,
}

fn classify_event<'p, const REASM_CAP: usize>(
    calls: &mut [PendingCall<REASM_CAP>],
    last_stream_id: &mut u32,
    goaway_limit: Option<u32>,
    n: usize,
    event: Option<H2Event<'p>>,
    classify_call: impl FnOnce(&str, &str) -> CallMode,
) -> PreDispatch<'p, REASM_CAP> {
    match event {
        Some(H2Event::Headers {
            stream_id,
            header_block,
            end_stream,
        }) => record_headers(
            calls,
            last_stream_id,
            goaway_limit,
            stream_id,
            header_block,
            end_stream,
            classify_call,
        ),
        Some(H2Event::GrpcMessage {
            stream_id,
            compressed,
            payload,
            end_stream,
        }) => {
            let Some(slot) = calls.iter_mut().find(|c| c.stream_id == stream_id) else {
                return PreDispatch::Other(PostEvent::Other);
            };
            match slot.mode {
                CallMode::SingleRequest => {
                    let call = core::mem::replace(slot, PendingCall::FREE);
                    PreDispatch::DispatchUnary {
                        call,
                        compressed,
                        payload,
                    }
                }
                CallMode::ClientStream => {
                    if compressed {
                        let roto_ct = slot.roto_content_type;
                        *slot = PendingCall::FREE;
                        return PreDispatch::Other(PostEvent::Reject {
                            stream_id,
                            headers_sent: false,
                            roto_content_type: roto_ct,
                            status: GrpcStatus::internal("message compression not supported"),
                        });
                    }
                    if let Err(status) = slot.append_lpm_frame(payload) {
                        let roto_ct = slot.roto_content_type;
                        *slot = PendingCall::FREE;
                        return PreDispatch::Other(PostEvent::Reject {
                            stream_id,
                            headers_sent: false,
                            roto_content_type: roto_ct,
                            status,
                        });
                    }
                    if end_stream {
                        let call = core::mem::replace(slot, PendingCall::FREE);
                        PreDispatch::DispatchClientStream { call }
                    } else {
                        PreDispatch::Other(PostEvent::Other)
                    }
                }
                CallMode::BidiStream => {
                    if compressed {
                        let headers_sent = slot.headers_sent;
                        let roto_ct = slot.roto_content_type;
                        *slot = PendingCall::FREE;
                        return PreDispatch::Other(PostEvent::Reject {
                            stream_id,
                            headers_sent,
                            roto_content_type: roto_ct,
                            status: GrpcStatus::internal("message compression not supported"),
                        });
                    }
                    slot.req_len = 0;
                    if let Err(status) = slot.append_lpm_frame(payload) {
                        let headers_sent = slot.headers_sent;
                        let roto_ct = slot.roto_content_type;
                        *slot = PendingCall::FREE;
                        return PreDispatch::Other(PostEvent::Reject {
                            stream_id,
                            headers_sent,
                            roto_content_type: roto_ct,
                            status,
                        });
                    }
                    let send_headers = !slot.headers_sent;
                    slot.headers_sent = true;
                    let call = if end_stream {
                        core::mem::replace(slot, PendingCall::FREE)
                    } else {
                        let snapshot = *slot;
                        slot.req_len = 0;
                        snapshot
                    };
                    PreDispatch::DispatchBidiMessage {
                        call,
                        send_headers,
                        send_trailers: end_stream,
                    }
                }
            }
        }
        Some(H2Event::EndStream { stream_id }) => {
            if let Some(call) = take(calls, stream_id) {
                match call.mode {
                    CallMode::ClientStream => PreDispatch::DispatchClientStream { call },
                    CallMode::BidiStream => {
                        if call.headers_sent {
                            PreDispatch::Other(PostEvent::FinishBidiStream { stream_id })
                        } else {
                            PreDispatch::DispatchClientStream { call }
                        }
                    }
                    CallMode::SingleRequest => PreDispatch::Other(PostEvent::Reject {
                        stream_id,
                        headers_sent: false,
                        roto_content_type: call.roto_content_type,
                        status: GrpcStatus::internal("missing request message"),
                    }),
                }
            } else {
                PreDispatch::Other(PostEvent::Other)
            }
        }
        Some(H2Event::RstStream { stream_id, .. })
        | Some(H2Event::StreamError { stream_id, .. }) => {
            PreDispatch::Other(PostEvent::CancelStream { stream_id })
        }
        Some(H2Event::Ping { ack: true, data }) => PreDispatch::Other(PostEvent::PingAck(data)),
        Some(_) => PreDispatch::Other(PostEvent::Other),
        None if n == 0 => PreDispatch::Other(PostEvent::Idle),
        None => PreDispatch::Other(PostEvent::Other),
    }
}

fn validate_call<const REASM_CAP: usize>(
    call: &PendingCall<REASM_CAP>,
    compressed: bool,
) -> Result<&str, GrpcStatus<'static>> {
    if compressed {
        return Err(GrpcStatus::internal("message compression not supported"));
    }
    core::str::from_utf8(&call.path[..call.path_len])
        .map_err(|_| GrpcStatus::invalid_argument(":path is not UTF-8"))
}

/// `#![no_std]` zero-allocation Sans-I/O gRPC server over one HTTP/2 connection.
///
/// Feed inbound bytes with [`Self::recv`] and drain outbound bytes with [`Self::poll_transmit`].
/// Unary and streaming calls on any number of concurrent streams (up to `MAX_STREAMS`) are
/// dispatched as soon as their request message(s) are complete.
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
    core: ServerCore<'a, MAX_STREAMS, REASM_CAP>,
    routes: &'a [&'a dyn Route],
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> core::fmt::Debug
    for GrpcServer<'_, MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GrpcServer")
            .field("conn", &self.core.conn)
            .field("routes_len", &self.routes.len())
            .field("last_stream_id", &self.core.last_stream_id)
            .field("msg_buf_cap", &self.core.msg_buf.len())
            .field("out_len", &self.core.out.len())
            .finish()
    }
}

impl<'a, const MAX_STREAMS: usize, const REASM_CAP: usize> GrpcServer<'a, MAX_STREAMS, REASM_CAP> {
    /// Creates a server dispatching to `routes`, encoding responses into `msg_buf`, and
    /// queueing outbound bytes in `tx_buf`.
    pub fn new(routes: &'a [&'a dyn Route], msg_buf: &'a mut [u8], tx_buf: &'a mut [u8]) -> Self {
        Self {
            core: ServerCore::new(msg_buf, tx_buf),
            routes,
        }
    }

    /// Resets all HTTP/2 connection and stream state so this server instance and its buffers
    /// can be reused for a new client connection.
    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub(crate) fn reset_if_active(&mut self) {
        if self.core.active_connection {
            self.core.reset();
        }
    }

    /// Returns a shared reference to the underlying [`H2Connection`].
    #[must_use]
    #[inline]
    pub const fn connection(&self) -> &H2Connection<MAX_STREAMS, REASM_CAP> {
        &self.core.conn
    }

    /// Returns a mutable reference to the underlying [`H2Connection`].
    #[inline]
    pub fn connection_mut(&mut self) -> &mut H2Connection<MAX_STREAMS, REASM_CAP> {
        &mut self.core.conn
    }

    /// Returns the highest client stream ID accepted so far (`0` if none).
    #[must_use]
    #[inline]
    pub const fn last_stream_id(&self) -> u32 {
        self.core.last_stream_id
    }

    /// Returns `(last_stream_id, error_code)` if a `GOAWAY` frame has been received from the client.
    #[must_use]
    #[inline]
    pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
        self.core.conn.goaway_received()
    }

    /// Queues an outbound HTTP/2 `PING` frame (`ack = false`) with the 8-byte `data` payload.
    pub fn send_ping(&mut self, data: [u8; 8]) -> Result<(), H2Error> {
        self.core.send_ping(data)
    }

    /// Takes the 8-byte opaque payload of the most recently received `PING` ACK frame, if any.
    #[must_use]
    #[inline]
    pub fn take_ping_ack(&mut self) -> Option<[u8; 8]> {
        self.core.last_ping_ack.take()
    }

    /// Initiates graceful shutdown by queuing a `GOAWAY` frame advertising [`Self::last_stream_id`].
    pub fn send_goaway(
        &mut self,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.core
            .send_goaway_at(self.core.last_stream_id, error_code, debug_data)
    }

    /// Queues a `GOAWAY` frame with an explicit `last_stream_id`, refusing any client streams
    /// with `stream_id > last_stream_id`.
    pub fn send_goaway_at(
        &mut self,
        last_stream_id: u32,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.core
            .send_goaway_at(last_stream_id, error_code, debug_data)
    }

    /// Returns `true` if the server has buffered outbound frames, a partially received inbound
    /// frame, or any in-flight call/response awaiting completion.
    #[must_use]
    pub fn has_pending_work(&self) -> bool {
        self.core.has_pending_work()
    }

    /// Consumes all of `input`, dispatching every completed call.
    ///
    /// Any [`H2Error`] is fatal for the connection: the peer has already been sent `GOAWAY`
    /// via [`Self::poll_transmit`] and no further input should be fed.
    pub fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        if !input.is_empty() {
            self.core.active_connection = true;
        }
        let routes = self.routes;
        while !input.is_empty() {
            let (n, event) = match self.core.conn.recv(input) {
                Ok(res) => res,
                Err(err) => {
                    self.core.flush_conn();
                    return Err(err);
                }
            };
            input = &input[n..];
            match classify_event(
                &mut self.core.calls,
                &mut self.core.last_stream_id,
                self.core.goaway_limit,
                n,
                event,
                |svc, method| {
                    let Some(route) = routes.iter().find(|r| r.service_name() == svc) else {
                        return CallMode::SingleRequest;
                    };
                    if route.is_bidi_streaming(method) {
                        CallMode::BidiStream
                    } else if route.is_client_streaming(method) {
                        CallMode::ClientStream
                    } else {
                        CallMode::SingleRequest
                    }
                },
            ) {
                PreDispatch::DispatchUnary {
                    call,
                    compressed,
                    payload,
                } => {
                    let cap = self.core.msg_buf.len();
                    let result = validate_call(&call, compressed).and_then(|path| {
                        dispatch(routes, path, payload, self.core.msg_buf)
                            .map(|msg| (cap - msg.len())..cap)
                    });
                    self.core
                        .finish_message(call.stream_id, call.roto_content_type, result);
                }
                PreDispatch::DispatchClientStream { call } => {
                    let cap = self.core.msg_buf.len();
                    let result = validate_call(&call, false).and_then(|path| {
                        dispatch_stream(
                            routes,
                            path,
                            &call.req_buf[..call.req_len],
                            self.core.msg_buf,
                        )
                        .map(|msg| (cap - msg.len())..cap)
                    });
                    self.core
                        .finish_message(call.stream_id, call.roto_content_type, result);
                }
                PreDispatch::DispatchBidiMessage {
                    call,
                    send_headers,
                    send_trailers,
                } => {
                    let cap = self.core.msg_buf.len();
                    let result = validate_call(&call, false).and_then(|path| {
                        dispatch_stream(
                            routes,
                            path,
                            &call.req_buf[..call.req_len],
                            self.core.msg_buf,
                        )
                        .map(|msg| (cap - msg.len())..cap)
                    });
                    self.core.finish_bidi_message(
                        call.stream_id,
                        send_headers,
                        send_trailers,
                        call.roto_content_type,
                        result,
                    );
                }
                PreDispatch::Other(post) => {
                    if !self.core.apply_post_event(post) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.core.poll_transmit(out)
    }
}

/// `#![no_std]` zero-allocation asynchronous gRPC server over one HTTP/2 connection,
/// dispatching to an [`AsyncRouter`] (any [`AsyncGrpcService`] or tuple of [`AsyncGrpcService`]s).
#[must_use]
pub struct AsyncGrpcServer<
    'a,
    R,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    core: ServerCore<'a, MAX_STREAMS, REASM_CAP>,
    router: &'a R,
}

impl<R, const MAX_STREAMS: usize, const REASM_CAP: usize> core::fmt::Debug
    for AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AsyncGrpcServer")
            .field("conn", &self.core.conn)
            .field("last_stream_id", &self.core.last_stream_id)
            .field("msg_buf_cap", &self.core.msg_buf.len())
            .field("out_len", &self.core.out.len())
            .finish()
    }
}

impl<'a, R: AsyncRouter, const MAX_STREAMS: usize, const REASM_CAP: usize>
    AsyncGrpcServer<'a, R, MAX_STREAMS, REASM_CAP>
{
    /// Creates an asynchronous server dispatching to `router`, encoding responses into `msg_buf`,
    /// and queueing outbound bytes in `tx_buf`.
    pub fn new(router: &'a R, msg_buf: &'a mut [u8], tx_buf: &'a mut [u8]) -> Self {
        Self {
            core: ServerCore::new(msg_buf, tx_buf),
            router,
        }
    }

    /// Resets all HTTP/2 connection and stream state so this server instance and its buffers
    /// can be reused for a new client connection.
    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub(crate) fn reset_if_active(&mut self) {
        if self.core.active_connection {
            self.core.reset();
        }
    }

    /// Returns a shared reference to the underlying [`H2Connection`].
    #[must_use]
    #[inline]
    pub const fn connection(&self) -> &H2Connection<MAX_STREAMS, REASM_CAP> {
        &self.core.conn
    }

    /// Returns a mutable reference to the underlying [`H2Connection`].
    #[inline]
    pub fn connection_mut(&mut self) -> &mut H2Connection<MAX_STREAMS, REASM_CAP> {
        &mut self.core.conn
    }

    /// Returns the highest client stream ID accepted so far (`0` if none).
    #[must_use]
    #[inline]
    pub const fn last_stream_id(&self) -> u32 {
        self.core.last_stream_id
    }

    /// Returns `(last_stream_id, error_code)` if a `GOAWAY` frame has been received from the client.
    #[must_use]
    #[inline]
    pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
        self.core.conn.goaway_received()
    }

    /// Queues an outbound HTTP/2 `PING` frame (`ack = false`) with the 8-byte `data` payload.
    pub fn send_ping(&mut self, data: [u8; 8]) -> Result<(), H2Error> {
        self.core.send_ping(data)
    }

    /// Takes the 8-byte opaque payload of the most recently received `PING` ACK frame, if any.
    #[must_use]
    #[inline]
    pub fn take_ping_ack(&mut self) -> Option<[u8; 8]> {
        self.core.last_ping_ack.take()
    }

    /// Initiates graceful shutdown by queuing a `GOAWAY` frame advertising [`Self::last_stream_id`].
    pub fn send_goaway(
        &mut self,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.core
            .send_goaway_at(self.core.last_stream_id, error_code, debug_data)
    }

    /// Queues a `GOAWAY` frame with an explicit `last_stream_id`, refusing any client streams
    /// with `stream_id > last_stream_id`.
    pub fn send_goaway_at(
        &mut self,
        last_stream_id: u32,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        self.core
            .send_goaway_at(last_stream_id, error_code, debug_data)
    }

    /// Returns `true` if the server has buffered outbound frames, a partially received inbound
    /// frame, or any in-flight call/response awaiting completion.
    #[must_use]
    pub fn has_pending_work(&self) -> bool {
        self.core.has_pending_work()
    }

    /// Asynchronously consumes all of `input`, awaiting [`AsyncRouter::dispatch_async`] or
    /// [`AsyncRouter::dispatch_stream_async`] for each completed call.
    pub async fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        if !input.is_empty() {
            self.core.active_connection = true;
        }
        let router = self.router;
        while !input.is_empty() {
            let (n, event) = match self.core.conn.recv(input) {
                Ok(res) => res,
                Err(err) => {
                    self.core.flush_conn();
                    return Err(err);
                }
            };
            input = &input[n..];
            match classify_event(
                &mut self.core.calls,
                &mut self.core.last_stream_id,
                self.core.goaway_limit,
                n,
                event,
                |svc, method| {
                    if router.is_bidi_streaming(svc, method) {
                        CallMode::BidiStream
                    } else if router.is_client_streaming(svc, method) {
                        CallMode::ClientStream
                    } else {
                        CallMode::SingleRequest
                    }
                },
            ) {
                PreDispatch::DispatchUnary {
                    call,
                    compressed,
                    payload,
                } => {
                    let cap = self.core.msg_buf.len();
                    let result = match validate_call(&call, compressed) {
                        Ok(path) => dispatch_async(router, path, payload, self.core.msg_buf)
                            .await
                            .map(|msg| (cap - msg.len())..cap),
                        Err(status) => Err(status),
                    };
                    self.core
                        .finish_message(call.stream_id, call.roto_content_type, result);
                }
                PreDispatch::DispatchClientStream { call } => {
                    let cap = self.core.msg_buf.len();
                    let result = match validate_call(&call, false) {
                        Ok(path) => dispatch_stream_async(
                            router,
                            path,
                            &call.req_buf[..call.req_len],
                            self.core.msg_buf,
                        )
                        .await
                        .map(|msg| (cap - msg.len())..cap),
                        Err(status) => Err(status),
                    };
                    self.core
                        .finish_message(call.stream_id, call.roto_content_type, result);
                }
                PreDispatch::DispatchBidiMessage {
                    call,
                    send_headers,
                    send_trailers,
                } => {
                    let cap = self.core.msg_buf.len();
                    let result = match validate_call(&call, false) {
                        Ok(path) => dispatch_stream_async(
                            router,
                            path,
                            &call.req_buf[..call.req_len],
                            self.core.msg_buf,
                        )
                        .await
                        .map(|msg| (cap - msg.len())..cap),
                        Err(status) => Err(status),
                    };
                    self.core.finish_bidi_message(
                        call.stream_id,
                        send_headers,
                        send_trailers,
                        call.roto_content_type,
                        result,
                    );
                }
                PreDispatch::Other(post) => {
                    if !self.core.apply_post_event(post) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.core.poll_transmit(out)
    }
}

fn record_headers<'p, const REASM_CAP: usize>(
    calls: &mut [PendingCall<REASM_CAP>],
    last_stream_id: &mut u32,
    goaway_limit: Option<u32>,
    stream_id: u32,
    header_block: &[u8],
    end_stream: bool,
    classify_call: impl FnOnce(&str, &str) -> CallMode,
) -> PreDispatch<'p, REASM_CAP> {
    if goaway_limit.is_some_and(|limit| stream_id > limit) {
        return PreDispatch::Other(PostEvent::Headers {
            stream_id,
            accepted: None,
        });
    }
    if let Some(call) = take(calls, stream_id) {
        if end_stream {
            match call.mode {
                CallMode::ClientStream => PreDispatch::DispatchClientStream { call },
                CallMode::BidiStream => {
                    if call.headers_sent {
                        PreDispatch::Other(PostEvent::FinishBidiStream { stream_id })
                    } else {
                        PreDispatch::DispatchClientStream { call }
                    }
                }
                CallMode::SingleRequest => PreDispatch::Other(PostEvent::Headers {
                    stream_id,
                    accepted: Some(Err(GrpcStatus::internal("missing request message"))),
                }),
            }
        } else {
            PreDispatch::Other(PostEvent::Headers {
                stream_id,
                accepted: Some(Err(GrpcStatus::internal("missing request message"))),
            })
        }
    } else if stream_id <= *last_stream_id {
        // Request trailers after the call was answered.
        PreDispatch::Other(PostEvent::Headers {
            stream_id,
            accepted: None,
        })
    } else {
        *last_stream_id = stream_id;
        match accept(calls, stream_id, header_block, end_stream, classify_call) {
            Ok(Some(call)) => PreDispatch::DispatchClientStream { call },
            Ok(None) => PreDispatch::Other(PostEvent::Headers {
                stream_id,
                accepted: Some(Ok(())),
            }),
            Err(status) => PreDispatch::Other(PostEvent::Headers {
                stream_id,
                accepted: Some(Err(status)),
            }),
        }
    }
}

/// Removes and returns the pending call on `stream_id`, if any.
fn take<const REASM_CAP: usize>(
    calls: &mut [PendingCall<REASM_CAP>],
    stream_id: u32,
) -> Option<PendingCall<REASM_CAP>> {
    let call = calls.iter_mut().find(|c| c.stream_id == stream_id)?;
    Some(core::mem::replace(call, PendingCall::FREE))
}

/// Validates request headers and records the call until its message(s) arrive.
fn accept<const REASM_CAP: usize>(
    calls: &mut [PendingCall<REASM_CAP>],
    stream_id: u32,
    header_block: &[u8],
    end_stream: bool,
    classify_call: impl FnOnce(&str, &str) -> CallMode,
) -> Result<Option<PendingCall<REASM_CAP>>, GrpcStatus<'static>> {
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
    if path.len() > MAX_PATH_LEN {
        if end_stream {
            return Err(GrpcStatus::internal("missing request message"));
        }
        return Err(GrpcStatus::unimplemented(":path too long"));
    }
    let roto_content_type = headers.content_type == Some(GRPC_CONTENT_TYPE_ROTO);
    let mode = core::str::from_utf8(path)
        .ok()
        .and_then(split_grpc_path)
        .map_or(CallMode::SingleRequest, |(svc, method)| {
            classify_call(svc, method)
        });
    if end_stream {
        if matches!(mode, CallMode::ClientStream | CallMode::BidiStream) {
            let mut call = PendingCall::FREE;
            call.stream_id = stream_id;
            call.path_len = path.len();
            call.path[..path.len()].copy_from_slice(path);
            call.mode = mode;
            call.roto_content_type = roto_content_type;
            return Ok(Some(call));
        }
        return Err(GrpcStatus::internal("missing request message"));
    }
    let Some(call) = calls.iter_mut().find(|c| c.stream_id == 0) else {
        return Err(GrpcStatus::resource_exhausted("too many concurrent calls"));
    };
    call.stream_id = stream_id;
    call.path_len = path.len();
    call.path[..path.len()].copy_from_slice(path);
    call.mode = mode;
    call.headers_sent = false;
    call.roto_content_type = roto_content_type;
    call.req_len = 0;
    Ok(None)
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

/// Routes a client-streaming or bidirectional-streaming `path` to its service and returns
/// the LPM-framed response(s) encoded into `msg_buf`.
fn dispatch_stream<'m>(
    routes: &[&dyn Route],
    path: &str,
    req_frames: &[u8],
    msg_buf: &'m mut [u8],
) -> Result<&'m [u8], GrpcStatus<'static>> {
    let Some((service, method)) = split_grpc_path(path) else {
        return Err(GrpcStatus::unimplemented("malformed gRPC path"));
    };
    let Some(route) = routes.iter().find(|r| r.service_name() == service) else {
        return Err(GrpcStatus::unimplemented("unknown gRPC service"));
    };
    let mut writer = BackwardWriter::new(msg_buf);
    route.dispatch_stream(method, req_frames, &mut writer)?;
    Ok(writer.finish()?)
}

/// Asynchronously routes `path` to `router` and returns the LPM-framed response encoded into `msg_buf`.
async fn dispatch_async<'m, R: AsyncRouter>(
    router: &R,
    path: &str,
    payload: &[u8],
    msg_buf: &'m mut [u8],
) -> Result<&'m [u8], GrpcStatus<'static>> {
    let Some((service, method)) = split_grpc_path(path) else {
        return Err(GrpcStatus::unimplemented("malformed gRPC path"));
    };
    let mut writer = BackwardWriter::new(msg_buf);
    router
        .dispatch_async(service, method, payload, &mut writer)
        .await?;
    Ok(writer.finish()?)
}

/// Asynchronously routes a client-streaming or bidirectional-streaming `path` to `router` and
/// returns the LPM-framed response(s) encoded into `msg_buf`.
async fn dispatch_stream_async<'m, R: AsyncRouter>(
    router: &R,
    path: &str,
    req_frames: &[u8],
    msg_buf: &'m mut [u8],
) -> Result<&'m [u8], GrpcStatus<'static>> {
    let Some((service, method)) = split_grpc_path(path) else {
        return Err(GrpcStatus::unimplemented("malformed gRPC path"));
    };
    let mut writer = BackwardWriter::new(msg_buf);
    router
        .dispatch_stream_async(service, method, req_frames, &mut writer)
        .await?;
    Ok(writer.finish()?)
}
