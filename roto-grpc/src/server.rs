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

    /// Dispatches an HTTP API (`google.api.http`) request when `http` codegen is enabled.
    ///
    /// Returns `None` if `req` did not match any HTTP route on this service.
    #[inline]
    fn dispatch_http<'a, 's, 'b>(
        &self,
        req: &crate::http::HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        let _ = (req, scratch, writer);
        None
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

    #[inline]
    fn dispatch_http<'a, 's, 'b>(
        &self,
        req: &crate::http::HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        self.call_http(req, scratch, writer)
    }
}

/// Zero-allocation asynchronous service router trait for [`AsyncGrpcServer`].
///
/// Blanket-implemented for every [`AsyncGrpcService`] as well as tuples of up to 4
/// [`AsyncGrpcService`] implementations `(S1, S2, ...)` so multiple async services can
/// be served over a single HTTP/2 connection without heap allocation.
pub trait AsyncRouter {
    /// Returns `true` if `service_name` matches a gRPC service registered in this router.
    #[inline]
    fn is_grpc_service(&self, service_name: &str) -> bool {
        let _ = service_name;
        false
    }

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

    /// Asynchronously dispatches an HTTP API (`google.api.http`) request when `http` codegen is enabled.
    ///
    /// Returns `None` if `req` did not match any HTTP route on this router.
    #[inline]
    fn dispatch_http_async<'a, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'a mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>>> + 'a
    {
        async move {
            let _ = (req, scratch, writer);
            None
        }
    }
}

#[inline]
fn call_service_http_async<'a, 'b, S: AsyncGrpcService>(
    svc: &'a S,
    req: &'a crate::http::HttpRequest<'a>,
    scratch: &'a mut [u8],
    writer: &'a mut BackwardWriter<'b>,
) -> impl Future<Output = Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>>> + 'a {
    svc.call_http_async(req, scratch, writer)
}

impl<S: AsyncGrpcService> AsyncRouter for S {
    #[inline]
    fn is_grpc_service(&self, service_name: &str) -> bool {
        service_name == S::SERVICE_NAME
    }

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

    #[inline]
    fn dispatch_http_async<'a, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'a mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>>> + 'a
    {
        call_service_http_async(self, req, scratch, writer)
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService> AsyncRouter for (A, B) {
    #[inline]
    fn is_grpc_service(&self, service_name: &str) -> bool {
        service_name == A::SERVICE_NAME || service_name == B::SERVICE_NAME
    }

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

    #[inline]
    async fn dispatch_http_async<'a, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'a mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = call_service_http_async(&self.0, req, scratch, writer).await {
            return Some(res);
        }
        call_service_http_async(&self.1, req, scratch, writer).await
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService, C: AsyncGrpcService> AsyncRouter for (A, B, C) {
    #[inline]
    fn is_grpc_service(&self, service_name: &str) -> bool {
        service_name == A::SERVICE_NAME
            || service_name == B::SERVICE_NAME
            || service_name == C::SERVICE_NAME
    }

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

    #[inline]
    async fn dispatch_http_async<'a, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'a mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = call_service_http_async(&self.0, req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = call_service_http_async(&self.1, req, scratch, writer).await {
            return Some(res);
        }
        call_service_http_async(&self.2, req, scratch, writer).await
    }
}

impl<A: AsyncGrpcService, B: AsyncGrpcService, C: AsyncGrpcService, D: AsyncGrpcService> AsyncRouter
    for (A, B, C, D)
{
    #[inline]
    fn is_grpc_service(&self, service_name: &str) -> bool {
        service_name == A::SERVICE_NAME
            || service_name == B::SERVICE_NAME
            || service_name == C::SERVICE_NAME
            || service_name == D::SERVICE_NAME
    }

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

    #[inline]
    async fn dispatch_http_async<'a, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'a mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = call_service_http_async(&self.0, req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = call_service_http_async(&self.1, req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = call_service_http_async(&self.2, req, scratch, writer).await {
            return Some(res);
        }
        call_service_http_async(&self.3, req, scratch, writer).await
    }
}

const MAX_HTTP_METHOD_LEN: usize = 16;
const MAX_CONTENT_TYPE_LEN: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallMode {
    SingleRequest,
    ClientStream,
    BidiStream,
    Http,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProtocolMode {
    Undecided,
    H2,
    H1,
}

/// A request whose headers were accepted and whose message(s) have not yet completed.
#[derive(Clone, Copy)]
struct PendingCall<const REASM_CAP: usize = DEFAULT_REASM_CAP> {
    stream_id: u32,
    path_len: usize,
    path: [u8; MAX_PATH_LEN],
    http_method_len: u8,
    http_method: [u8; MAX_HTTP_METHOD_LEN],
    http_content_type_len: u8,
    http_content_type: [u8; MAX_CONTENT_TYPE_LEN],
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
        http_method_len: 0,
        http_method: [0; MAX_HTTP_METHOD_LEN],
        http_content_type_len: 0,
        http_content_type: [0; MAX_CONTENT_TYPE_LEN],
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

    fn append_raw_bytes(&mut self, payload: &[u8]) -> Result<(), GrpcStatus<'static>> {
        if self.req_len + payload.len() > REASM_CAP {
            return Err(GrpcStatus::resource_exhausted(
                "HTTP request body exceeds reassembly capacity",
            ));
        }
        self.req_buf[self.req_len..self.req_len + payload.len()].copy_from_slice(payload);
        self.req_len += payload.len();
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
    end_stream_on_data: bool,
}

impl PendingResponse {
    const FREE: Self = Self {
        stream_id: 0,
        start: 0,
        end: 0,
        send_trailers: false,
        end_stream_on_data: false,
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
    protocol_mode: ProtocolMode,
    h1_rx_len: usize,
    h1_close_after_flush: bool,
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
            protocol_mode: ProtocolMode::Undecided,
            h1_rx_len: 0,
            h1_close_after_flush: false,
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
        self.protocol_mode = ProtocolMode::Undecided;
        self.h1_rx_len = 0;
        self.h1_close_after_flush = false;
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
            let end_stream_on_data = self.pending_resps[i].end_stream_on_data;
            if start < end {
                self.flush_conn();
                let floor = self.staging_floor();
                let (wire_part, stage_part) = self.out.buf.split_at_mut(floor);
                let staged = &stage_part[start - floor..end - floor];
                match send_partial_data_into(
                    &mut self.conn,
                    stream_id,
                    staged,
                    end_stream_on_data,
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

    fn reject_http_stream(&mut self, stream_id: u32, status: GrpcStatus<'_>) {
        take(&mut self.calls, stream_id);
        self.cancel_pending_response(stream_id);
        let msg_bytes = status.message.as_bytes();
        let cap = self.msg_buf.len();
        if !msg_bytes.is_empty() && msg_bytes.len() <= cap {
            let start = cap - msg_bytes.len();
            self.msg_buf[start..cap].copy_from_slice(msg_bytes);
            self.respond_http_from_msg_buf(
                stream_id,
                status.http_status(),
                crate::http::CONTENT_TYPE_TEXT_PLAIN.as_bytes(),
                start..cap,
            );
        } else {
            self.respond_http_from_msg_buf(
                stream_id,
                status.http_status(),
                crate::http::CONTENT_TYPE_TEXT_PLAIN.as_bytes(),
                cap..cap,
            );
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
                end_stream_on_data: false,
            };
            self.drain_pending_responses();
            return;
        }
        let _ = self
            .conn
            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
        self.flush_conn();
    }

    fn on_headers(
        &mut self,
        stream_id: u32,
        raw_data: bool,
        accepted: Option<Result<(), GrpcStatus<'static>>>,
    ) {
        if self.goaway_limit.is_some_and(|limit| stream_id > limit) {
            take(&mut self.calls, stream_id);
            let _ = self
                .conn
                .queue_rst_stream(stream_id, H2ErrorCode::RefusedStream);
            self.drain_pending_responses();
            return;
        }
        if raw_data {
            self.conn.set_stream_raw_data(stream_id, true);
        }
        match accepted {
            None => self.drain_pending_responses(),
            Some(Ok(())) => self.drain_pending_responses(),
            Some(Err(status)) => {
                if raw_data {
                    self.reject_http_stream(stream_id, status);
                } else {
                    self.reject_stream(stream_id, false, true, status);
                }
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
                            end_stream_on_data: false,
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

    fn respond_http_from_msg_buf(
        &mut self,
        stream_id: u32,
        status_code: u16,
        content_type: &[u8],
        range: core::ops::Range<usize>,
    ) {
        self.flush_conn();
        let floor = self.staging_floor();
        let msg = &self.msg_buf[range];
        let avail = floor.saturating_sub(self.out.len);
        let hdr_wire = FRAME_HEADER_LEN + 8 + content_type.len();
        if msg.is_empty() {
            if avail >= hdr_wire {
                if let Ok(hdr_n) = self.conn.send_http_response_headers(
                    stream_id,
                    status_code,
                    content_type,
                    true,
                    &mut self.out.buf[self.out.len..floor],
                ) {
                    self.out.len += hdr_n;
                    self.flush_conn();
                    return;
                }
            }
            let _ = self
                .conn
                .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
            self.flush_conn();
            return;
        }

        let max_frame = self.conn.peer_settings().max_frame_size as usize;
        let data_wire = msg.len() + msg.len().div_ceil(max_frame) * FRAME_HEADER_LEN;
        let full_wire = hdr_wire + data_wire;
        let stream_win = self.conn.stream_send_window(stream_id).unwrap_or(0);
        let win = self.conn.conn_send_window().min(stream_win).max(0) as usize;
        let has_staged = self.pending_resps.iter().any(|r| r.stream_id == stream_id);

        if full_wire <= self.out.cap() {
            if !has_staged && win >= msg.len() && avail >= full_wire {
                if let Ok(hdr_n) = self.conn.send_http_response_headers(
                    stream_id,
                    status_code,
                    content_type,
                    false,
                    &mut self.out.buf[self.out.len..floor],
                ) {
                    self.out.len += hdr_n;
                    let data_ok = send_partial_data_into(
                        &mut self.conn,
                        stream_id,
                        msg,
                        true,
                        &mut self.out.buf[..floor],
                        &mut self.out.len,
                    )
                    .is_ok_and(|(_, done)| done);
                    if !data_ok {
                        let _ = self
                            .conn
                            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
                        self.flush_conn();
                    }
                    return;
                }
            } else if avail >= hdr_wire + msg.len() {
                if let Some(slot) = self.pending_resps.iter().position(|r| r.stream_id == 0) {
                    if let Ok(hdr_n) = self.conn.send_http_response_headers(
                        stream_id,
                        status_code,
                        content_type,
                        false,
                        &mut self.out.buf[self.out.len..floor],
                    ) {
                        self.out.len += hdr_n;
                        let stage_start = floor - msg.len();
                        self.out.buf[stage_start..floor].copy_from_slice(msg);
                        self.pending_resps[slot] = PendingResponse {
                            stream_id,
                            start: stage_start,
                            end: floor,
                            send_trailers: false,
                            end_stream_on_data: true,
                        };
                        self.drain_pending_responses();
                        return;
                    }
                }
            }
        }
        let _ = self
            .conn
            .queue_rst_stream(stream_id, H2ErrorCode::InternalError);
        self.flush_conn();
    }

    fn send_ping(&mut self, data: [u8; 8]) -> Result<(), H2Error> {
        self.protocol_mode = ProtocolMode::H2;
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
        self.protocol_mode = ProtocolMode::H2;
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

    const fn should_close(&self) -> bool {
        self.h1_close_after_flush && self.out.len == 0
    }

    fn has_pending_work(&self) -> bool {
        match self.protocol_mode {
            ProtocolMode::Undecided | ProtocolMode::H1 => self.h1_rx_len > 0 || self.out.len() > 0,
            ProtocolMode::H2 => {
                self.out.len() > 0
                    || self.conn.has_pending_transmit()
                    || self.conn.has_partial_inbound_frame()
                    || self.calls.iter().any(|c| c.stream_id != 0)
                    || self.pending_resps.iter().any(|r| r.stream_id != 0)
            }
        }
    }

    fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        if self.protocol_mode == ProtocolMode::H2 {
            self.drain_pending_responses();
        }
        let n = self.out.drain(out);
        if n > 0 {
            self.active_connection = true;
        }
        if self.protocol_mode == ProtocolMode::H2 {
            self.drain_pending_responses();
        }
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

    fn finish_http_message(
        &mut self,
        stream_id: u32,
        result: Result<(u16, &'static str, core::ops::Range<usize>), GrpcStatus<'static>>,
    ) {
        match result {
            Ok((status_code, content_type, range)) => {
                self.respond_http_from_msg_buf(
                    stream_id,
                    status_code,
                    content_type.as_bytes(),
                    range,
                );
            }
            Err(status) => {
                self.reject_http_stream(stream_id, status);
            }
        }
        self.drain_pending_responses();
    }

    fn write_h1_bad_request(&mut self) {
        self.write_h1_fatal_error(&GrpcStatus::invalid_argument("bad request"));
    }

    fn write_h1_fatal_error(&mut self, status: &GrpcStatus<'_>) {
        self.h1_rx_len = 0;
        self.h1_close_after_flush = true;
        if let Ok(n) =
            crate::http::encode_h1_error(status, false, &mut self.out.buf[self.out.len..])
        {
            self.out.len += n;
        }
    }

    fn finish_h1_message(
        &mut self,
        keep_alive: bool,
        result: Result<(u16, &'static str, core::ops::Range<usize>), GrpcStatus<'static>>,
    ) {
        if !keep_alive {
            self.h1_close_after_flush = true;
        }
        match result {
            Ok((status_code, content_type, range)) => {
                let resp = crate::http::HttpResponse {
                    status_code,
                    content_type,
                    body: &self.msg_buf[range],
                };
                if let Ok(n) = crate::http::encode_h1_http_response(
                    &resp,
                    keep_alive,
                    &mut self.out.buf[self.out.len..],
                ) {
                    self.out.len += n;
                } else {
                    self.h1_close_after_flush = true;
                    let overflow_status = GrpcStatus::internal("response buffer overflow");
                    if let Ok(n) = crate::http::encode_h1_error(
                        &overflow_status,
                        false,
                        &mut self.out.buf[self.out.len..],
                    ) {
                        self.out.len += n;
                    }
                }
            }
            Err(status) => {
                if let Ok(n) = crate::http::encode_h1_error(
                    &status,
                    keep_alive,
                    &mut self.out.buf[self.out.len..],
                ) {
                    self.out.len += n;
                } else {
                    self.h1_close_after_flush = true;
                }
            }
        }
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
                raw_data,
                accepted,
            } => {
                self.on_headers(stream_id, raw_data, accepted);
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
            PostEvent::RejectHttp { stream_id, status } => {
                self.reject_http_stream(stream_id, status);
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

    fn resolve_undecided<'i>(&mut self, input: &mut &'i [u8]) -> Result<(), H2Error> {
        let cur = *input;
        if cur.is_empty() {
            return Ok(());
        }
        if self.h1_rx_len == 0 {
            match detect_protocol(cur) {
                Some(ProtocolMode::H2) => {
                    self.protocol_mode = ProtocolMode::H2;
                }
                Some(ProtocolMode::H1) => {
                    self.protocol_mode = ProtocolMode::H1;
                }
                None | Some(ProtocolMode::Undecided) => {
                    let (rx_buf, _) = self.conn.h1_buffers();
                    let take = cur.len().min(REASM_CAP);
                    rx_buf[..take].copy_from_slice(&cur[..take]);
                    self.h1_rx_len = take;
                    *input = &[];
                }
            }
        } else {
            let prev_len = self.h1_rx_len;
            let (rx_buf, _) = self.conn.h1_buffers();
            let take = cur.len().min(REASM_CAP.saturating_sub(prev_len));
            rx_buf[prev_len..prev_len + take].copy_from_slice(&cur[..take]);
            self.h1_rx_len = prev_len + take;
            match detect_protocol(&rx_buf[..self.h1_rx_len]) {
                Some(ProtocolMode::H1) => {
                    self.protocol_mode = ProtocolMode::H1;
                    *input = &cur[take..];
                }
                Some(ProtocolMode::H2) => {
                    let mut prefix = [0u8; 64];
                    let n_pre = prev_len.min(prefix.len());
                    prefix[..n_pre].copy_from_slice(&rx_buf[..n_pre]);
                    self.h1_rx_len = 0;
                    self.protocol_mode = ProtocolMode::H2;
                    let mut pre_slice = &prefix[..n_pre];
                    while !pre_slice.is_empty() {
                        match self.conn.recv(pre_slice) {
                            Ok((n, _)) => pre_slice = &pre_slice[n..],
                            Err(err) => {
                                self.flush_conn();
                                return Err(err);
                            }
                        }
                    }
                }
                None | Some(ProtocolMode::Undecided) => {
                    *input = &[];
                }
            }
        }
        Ok(())
    }
}

fn detect_protocol(buf: &[u8]) -> Option<ProtocolMode> {
    if buf.is_empty() {
        return None;
    }
    let mut method_len = 0usize;
    while method_len < buf.len() && method_len <= MAX_HTTP_METHOD_LEN {
        if buf[method_len].is_ascii_uppercase() {
            method_len += 1;
        } else {
            break;
        }
    }
    if method_len == 0 || method_len > MAX_HTTP_METHOD_LEN {
        return Some(ProtocolMode::H2);
    }
    if method_len == buf.len() {
        return None;
    }
    if method_len < 2 || &buf[..method_len] == b"PRI" || buf[method_len] != b' ' {
        return Some(ProtocolMode::H2);
    }
    if buf.len() == method_len + 1 {
        return None;
    }
    if buf[method_len + 1] != b'/' {
        return Some(ProtocolMode::H2);
    }
    if buf.windows(8).any(|w| w == b"HTTP/2.0") {
        return Some(ProtocolMode::H2);
    }
    if buf.contains(&b'\n') || buf.len() >= 64 {
        Some(ProtocolMode::H1)
    } else {
        None
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
    DispatchHttp {
        call: PendingCall<REASM_CAP>,
    },
    Other(PostEvent),
}

enum PostEvent {
    Headers {
        stream_id: u32,
        raw_data: bool,
        accepted: Option<Result<(), GrpcStatus<'static>>>,
    },
    Reject {
        stream_id: u32,
        headers_sent: bool,
        roto_content_type: bool,
        status: GrpcStatus<'static>,
    },
    RejectHttp {
        stream_id: u32,
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
    is_grpc_service: impl FnOnce(&str) -> bool,
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
            is_grpc_service,
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
                CallMode::Http => {
                    if let Err(status) = slot.append_raw_bytes(payload) {
                        *slot = PendingCall::FREE;
                        return PreDispatch::Other(PostEvent::RejectHttp { stream_id, status });
                    }
                    if end_stream {
                        let call = core::mem::replace(slot, PendingCall::FREE);
                        PreDispatch::DispatchHttp { call }
                    } else {
                        PreDispatch::Other(PostEvent::Other)
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
                    CallMode::Http => PreDispatch::DispatchHttp { call },
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

/// `#![no_std]` zero-allocation Sans-I/O gRPC and HTTP API server over HTTP/2 or HTTP/1.1.
///
/// Feed inbound bytes with [`Self::recv`] and drain outbound bytes with [`Self::poll_transmit`].
/// Unary, streaming, and HTTP API (`google.api.http`) calls are dispatched as soon as their
/// request payload is complete.
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

    /// Resets all HTTP/2 and HTTP/1.1 connection and stream state so this server instance and
    /// its buffers can be reused for a new client connection.
    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub(crate) fn reset_if_active(&mut self) {
        if self.core.active_connection {
            self.core.reset();
        }
    }

    /// Returns `true` if an HTTP/1.1 request requested connection close (or encountered a
    /// framing error) and all outbound response bytes have been drained.
    #[must_use]
    #[inline]
    pub const fn should_close(&self) -> bool {
        self.core.should_close()
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

    /// Consumes all of `input`, dispatching every completed gRPC or HTTP API call.
    ///
    /// Any [`H2Error`] is fatal for the connection: the peer has already been sent `GOAWAY`
    /// via [`Self::poll_transmit`] and no further input should be fed.
    pub fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        if !input.is_empty() {
            self.core.active_connection = true;
        }
        if self.core.protocol_mode == ProtocolMode::Undecided {
            self.core.resolve_undecided(&mut input)?;
        }
        match self.core.protocol_mode {
            ProtocolMode::Undecided => Ok(()),
            ProtocolMode::H2 => self.recv_h2(input),
            ProtocolMode::H1 => {
                self.recv_h1(input);
                Ok(())
            }
        }
    }

    fn recv_h1(&mut self, mut input: &[u8]) {
        let routes = self.routes;
        while !input.is_empty() || self.core.h1_rx_len > 0 {
            if self.core.h1_close_after_flush {
                self.core.h1_rx_len = 0;
                break;
            }
            if self.core.h1_rx_len > 0 && !input.is_empty() {
                let (rx_buf, _) = self.core.conn.h1_buffers();
                let avail = REASM_CAP.saturating_sub(self.core.h1_rx_len);
                if avail == 0 {
                    self.core.write_h1_bad_request();
                    break;
                }
                let take = input.len().min(avail);
                rx_buf[self.core.h1_rx_len..self.core.h1_rx_len + take]
                    .copy_from_slice(&input[..take]);
                self.core.h1_rx_len += take;
                input = &input[take..];
            }

            if self.core.h1_rx_len == 0 {
                match crate::http::parse_h1_request(input) {
                    Ok(Some(parsed)) => {
                        let keep_alive = parsed.keep_alive;
                        let consumed = parsed.consumed;
                        let scratch = self.core.conn.http_scratch_buf();
                        let result =
                            dispatch_http(routes, &parsed.request, scratch, self.core.msg_buf);
                        self.core.finish_h1_message(keep_alive, result);
                        input = &input[consumed..];
                    }
                    Ok(None) => {
                        if input.len() > REASM_CAP {
                            self.core.write_h1_bad_request();
                            break;
                        }
                        let (rx_buf, _) = self.core.conn.h1_buffers();
                        rx_buf[..input.len()].copy_from_slice(input);
                        self.core.h1_rx_len = input.len();
                        break;
                    }
                    Err(status) => {
                        self.core.write_h1_fatal_error(&status);
                        break;
                    }
                }
            } else {
                let h1_len = self.core.h1_rx_len;
                let (rx_buf, scratch) = self.core.conn.h1_buffers();
                match crate::http::parse_h1_request(&rx_buf[..h1_len]) {
                    Ok(Some(parsed)) => {
                        let keep_alive = parsed.keep_alive;
                        let consumed = parsed.consumed;
                        let result =
                            dispatch_http(routes, &parsed.request, scratch, self.core.msg_buf);
                        let _ = parsed;
                        let (rx_buf2, _) = self.core.conn.h1_buffers();
                        if consumed < h1_len {
                            rx_buf2.copy_within(consumed..h1_len, 0);
                        }
                        self.core.h1_rx_len = h1_len - consumed;
                        self.core.finish_h1_message(keep_alive, result);
                    }
                    Ok(None) => {
                        if input.is_empty() {
                            if h1_len == REASM_CAP {
                                self.core.write_h1_bad_request();
                            }
                            break;
                        }
                    }
                    Err(status) => {
                        self.core.write_h1_fatal_error(&status);
                        break;
                    }
                }
            }
        }
    }

    fn recv_h2(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
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
                |svc| routes.iter().any(|r| r.service_name() == svc),
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
                PreDispatch::DispatchHttp { call } => {
                    let scratch = self.core.conn.http_scratch_buf();
                    let result = build_h2_http_request(&call)
                        .and_then(|req| dispatch_http(routes, &req, scratch, self.core.msg_buf));
                    self.core.finish_http_message(call.stream_id, result);
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

/// `#![no_std]` zero-allocation asynchronous gRPC and HTTP API server over HTTP/2 or HTTP/1.1,
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
    /// Creates an async server dispatching to `router`, encoding responses into `msg_buf`, and
    /// queueing outbound bytes in `tx_buf`.
    pub fn new(router: &'a R, msg_buf: &'a mut [u8], tx_buf: &'a mut [u8]) -> Self {
        Self {
            core: ServerCore::new(msg_buf, tx_buf),
            router,
        }
    }

    /// Resets all HTTP/2 and HTTP/1.1 connection and stream state so this server instance and
    /// its buffers can be reused for a new client connection.
    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub(crate) fn reset_if_active(&mut self) {
        if self.core.active_connection {
            self.core.reset();
        }
    }

    /// Returns `true` if an HTTP/1.1 request requested connection close (or encountered a
    /// framing error) and all outbound response bytes have been drained.
    #[must_use]
    #[inline]
    pub const fn should_close(&self) -> bool {
        self.core.should_close()
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

    /// Asynchronously consumes all of `input`, awaiting [`AsyncRouter::dispatch_async`],
    /// [`AsyncRouter::dispatch_stream_async`], or [`AsyncRouter::dispatch_http_async`] for each
    /// completed call.
    pub async fn recv(&mut self, mut input: &[u8]) -> Result<(), H2Error> {
        if !input.is_empty() {
            self.core.active_connection = true;
        }
        if self.core.protocol_mode == ProtocolMode::Undecided {
            self.core.resolve_undecided(&mut input)?;
        }
        let router = self.router;
        match self.core.protocol_mode {
            ProtocolMode::Undecided => Ok(()),
            ProtocolMode::H1 => {
                while !input.is_empty() || self.core.h1_rx_len > 0 {
                    if self.core.h1_close_after_flush {
                        self.core.h1_rx_len = 0;
                        break;
                    }
                    if self.core.h1_rx_len > 0 && !input.is_empty() {
                        let (rx_buf, _) = self.core.conn.h1_buffers();
                        let avail = REASM_CAP.saturating_sub(self.core.h1_rx_len);
                        if avail == 0 {
                            self.core.write_h1_bad_request();
                            break;
                        }
                        let take = input.len().min(avail);
                        rx_buf[self.core.h1_rx_len..self.core.h1_rx_len + take]
                            .copy_from_slice(&input[..take]);
                        self.core.h1_rx_len += take;
                        input = &input[take..];
                    }

                    if self.core.h1_rx_len == 0 {
                        match crate::http::parse_h1_request(input) {
                            Ok(Some(parsed)) => {
                                let keep_alive = parsed.keep_alive;
                                let consumed = parsed.consumed;
                                let scratch = self.core.conn.http_scratch_buf();
                                let result = dispatch_http_async(
                                    router,
                                    &parsed.request,
                                    scratch,
                                    self.core.msg_buf,
                                )
                                .await;
                                self.core.finish_h1_message(keep_alive, result);
                                input = &input[consumed..];
                            }
                            Ok(None) => {
                                if input.len() > REASM_CAP {
                                    self.core.write_h1_bad_request();
                                    break;
                                }
                                let (rx_buf, _) = self.core.conn.h1_buffers();
                                rx_buf[..input.len()].copy_from_slice(input);
                                self.core.h1_rx_len = input.len();
                                break;
                            }
                            Err(status) => {
                                self.core.write_h1_fatal_error(&status);
                                break;
                            }
                        }
                    } else {
                        let h1_len = self.core.h1_rx_len;
                        let (rx_buf, scratch) = self.core.conn.h1_buffers();
                        match crate::http::parse_h1_request(&rx_buf[..h1_len]) {
                            Ok(Some(parsed)) => {
                                let keep_alive = parsed.keep_alive;
                                let consumed = parsed.consumed;
                                let result = dispatch_http_async(
                                    router,
                                    &parsed.request,
                                    scratch,
                                    self.core.msg_buf,
                                )
                                .await;
                                let _ = parsed;
                                let (rx_buf2, _) = self.core.conn.h1_buffers();
                                if consumed < h1_len {
                                    rx_buf2.copy_within(consumed..h1_len, 0);
                                }
                                self.core.h1_rx_len = h1_len - consumed;
                                self.core.finish_h1_message(keep_alive, result);
                            }
                            Ok(None) => {
                                if input.is_empty() {
                                    if h1_len == REASM_CAP {
                                        self.core.write_h1_bad_request();
                                    }
                                    break;
                                }
                            }
                            Err(status) => {
                                self.core.write_h1_fatal_error(&status);
                                break;
                            }
                        }
                    }
                }
                Ok(())
            }
            ProtocolMode::H2 => {
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
                        |svc| router.is_grpc_service(svc),
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
                                Ok(path) => {
                                    dispatch_async(router, path, payload, self.core.msg_buf)
                                        .await
                                        .map(|msg| (cap - msg.len())..cap)
                                }
                                Err(status) => Err(status),
                            };
                            self.core.finish_message(
                                call.stream_id,
                                call.roto_content_type,
                                result,
                            );
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
                            self.core.finish_message(
                                call.stream_id,
                                call.roto_content_type,
                                result,
                            );
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
                        PreDispatch::DispatchHttp { call } => {
                            let scratch = self.core.conn.http_scratch_buf();
                            let result = match build_h2_http_request(&call) {
                                Ok(req) => {
                                    dispatch_http_async(router, &req, scratch, self.core.msg_buf)
                                        .await
                                }
                                Err(status) => Err(status),
                            };
                            self.core.finish_http_message(call.stream_id, result);
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
        }
    }

    /// Drains queued outbound bytes into `out` and returns the number of bytes written.
    #[must_use]
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        self.core.poll_transmit(out)
    }
}

#[allow(clippy::too_many_arguments)]
fn record_headers<'p, const REASM_CAP: usize>(
    calls: &mut [PendingCall<REASM_CAP>],
    last_stream_id: &mut u32,
    goaway_limit: Option<u32>,
    stream_id: u32,
    header_block: &[u8],
    end_stream: bool,
    is_grpc_service: impl FnOnce(&str) -> bool,
    classify_call: impl FnOnce(&str, &str) -> CallMode,
) -> PreDispatch<'p, REASM_CAP> {
    if goaway_limit.is_some_and(|limit| stream_id > limit) {
        return PreDispatch::Other(PostEvent::Headers {
            stream_id,
            raw_data: false,
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
                CallMode::Http => PreDispatch::DispatchHttp { call },
                CallMode::SingleRequest => PreDispatch::Other(PostEvent::Headers {
                    stream_id,
                    raw_data: false,
                    accepted: Some(Err(GrpcStatus::internal("missing request message"))),
                }),
            }
        } else {
            PreDispatch::Other(PostEvent::Headers {
                stream_id,
                raw_data: call.mode == CallMode::Http,
                accepted: Some(Err(GrpcStatus::internal("missing request message"))),
            })
        }
    } else if stream_id <= *last_stream_id {
        // Request trailers after the call was answered.
        PreDispatch::Other(PostEvent::Headers {
            stream_id,
            raw_data: false,
            accepted: None,
        })
    } else {
        *last_stream_id = stream_id;
        match accept(
            calls,
            stream_id,
            header_block,
            end_stream,
            is_grpc_service,
            classify_call,
        ) {
            Ok((Some(call), _)) => match call.mode {
                CallMode::Http => PreDispatch::DispatchHttp { call },
                _ => PreDispatch::DispatchClientStream { call },
            },
            Ok((None, raw_data)) => PreDispatch::Other(PostEvent::Headers {
                stream_id,
                raw_data,
                accepted: Some(Ok(())),
            }),
            Err((status, raw_data)) => PreDispatch::Other(PostEvent::Headers {
                stream_id,
                raw_data,
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
    is_grpc_service: impl FnOnce(&str) -> bool,
    classify_call: impl FnOnce(&str, &str) -> CallMode,
) -> Result<(Option<PendingCall<REASM_CAP>>, bool), (GrpcStatus<'static>, bool)> {
    let mut scratch = [0u8; HPACK_SCRATCH_LEN];
    let mut decoder = HpackDecoder::<0>::stateless();
    let headers = GrpcRequestHeaders::decode(&mut decoder, header_block, &mut scratch)
        .map_err(|e| (GrpcStatus::from(e), false))?;
    let Some(path) = headers.path else {
        return Err((GrpcStatus::invalid_argument("missing :path"), false));
    };
    let is_grpc = headers.content_type.is_some_and(is_grpc_content_type)
        || core::str::from_utf8(path)
            .ok()
            .and_then(split_grpc_path)
            .is_some_and(|(svc, _)| is_grpc_service(svc));

    if !is_grpc {
        let Some(method) = headers.method else {
            return Err((GrpcStatus::invalid_argument("missing :method"), true));
        };
        if method.is_empty()
            || method.len() > MAX_HTTP_METHOD_LEN
            || core::str::from_utf8(method).is_err()
        {
            return Err((GrpcStatus::invalid_argument("invalid :method"), true));
        }
        if !path.starts_with(b"/") || core::str::from_utf8(path).is_err() {
            return Err((GrpcStatus::invalid_argument("invalid :path"), true));
        }
        if path.len() > MAX_PATH_LEN {
            return Err((GrpcStatus::invalid_argument(":path too long"), true));
        }
        let ct = headers.content_type.unwrap_or(b"");
        if ct.len() > MAX_CONTENT_TYPE_LEN || core::str::from_utf8(ct).is_err() {
            return Err((GrpcStatus::invalid_argument("invalid content-type"), true));
        }
        let mut call = PendingCall::FREE;
        call.stream_id = stream_id;
        call.path_len = path.len();
        call.path[..path.len()].copy_from_slice(path);
        call.http_method_len = method.len() as u8;
        call.http_method[..method.len()].copy_from_slice(method);
        call.http_content_type_len = ct.len() as u8;
        call.http_content_type[..ct.len()].copy_from_slice(ct);
        call.mode = CallMode::Http;
        if end_stream {
            return Ok((Some(call), true));
        }
        let Some(slot) = calls.iter_mut().find(|c| c.stream_id == 0) else {
            return Err((
                GrpcStatus::resource_exhausted("too many concurrent calls"),
                true,
            ));
        };
        *slot = call;
        return Ok((None, true));
    }

    if headers.method != Some(b"POST") {
        return Err((GrpcStatus::invalid_argument(":method must be POST"), false));
    }
    if headers.te != Some(b"trailers") {
        return Err((GrpcStatus::invalid_argument("te must be trailers"), false));
    }
    if !headers.content_type.is_some_and(is_grpc_content_type) {
        return Err((
            GrpcStatus::invalid_argument("content-type must be application/grpc"),
            false,
        ));
    }
    if path.len() > MAX_PATH_LEN {
        if end_stream {
            return Err((GrpcStatus::internal("missing request message"), false));
        }
        return Err((GrpcStatus::unimplemented(":path too long"), false));
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
            return Ok((Some(call), false));
        }
        return Err((GrpcStatus::internal("missing request message"), false));
    }
    let Some(call) = calls.iter_mut().find(|c| c.stream_id == 0) else {
        return Err((
            GrpcStatus::resource_exhausted("too many concurrent calls"),
            false,
        ));
    };
    call.stream_id = stream_id;
    call.path_len = path.len();
    call.path[..path.len()].copy_from_slice(path);
    call.mode = mode;
    call.headers_sent = false;
    call.roto_content_type = roto_content_type;
    call.req_len = 0;
    Ok((None, false))
}

fn build_h2_http_request<const REASM_CAP: usize>(
    call: &PendingCall<REASM_CAP>,
) -> Result<crate::http::HttpRequest<'_>, GrpcStatus<'static>> {
    let full_path = core::str::from_utf8(&call.path[..call.path_len])
        .map_err(|_| GrpcStatus::invalid_argument(":path is not UTF-8"))?;
    let (path, query) = crate::http::split_uri_path_and_query(full_path);
    let method_bytes = &call.http_method[..call.http_method_len as usize];
    let method = crate::http::HttpMethod::from_bytes(method_bytes)
        .ok_or_else(|| GrpcStatus::invalid_argument("invalid :method"))?;
    let content_type = if call.http_content_type_len == 0 {
        None
    } else {
        Some(
            core::str::from_utf8(&call.http_content_type[..call.http_content_type_len as usize])
                .map_err(|_| GrpcStatus::invalid_argument("invalid content-type"))?,
        )
    };
    Ok(crate::http::HttpRequest::new(
        method,
        path,
        query,
        content_type,
        &call.req_buf[..call.req_len],
    ))
}

/// Dispatches an HTTP API request across `routes` and returns `(status_code, content_type, range_in_msg_buf)`.
fn dispatch_http<'a>(
    routes: &[&dyn Route],
    req: &crate::http::HttpRequest<'a>,
    scratch: &mut [u8],
    msg_buf: &mut [u8],
) -> Result<(u16, &'static str, core::ops::Range<usize>), GrpcStatus<'static>> {
    for route in routes {
        let matched = {
            let mut writer = BackwardWriter::new(&mut *msg_buf);
            match route.dispatch_http(req, scratch, &mut writer) {
                Some(Ok(resp)) => {
                    let body_len = resp.body.len();
                    let status_code = resp.status_code;
                    let content_type = resp.content_type;
                    let body_ptr = resp.body.as_ptr() as usize;
                    let written = writer.written_len();
                    if body_len > 0 && written == 0 && body_len <= scratch.len() {
                        scratch[..body_len].copy_from_slice(resp.body);
                    }
                    let _ = resp;
                    let _ = writer;
                    Some(Ok((status_code, content_type, body_ptr, body_len, written)))
                }
                Some(Err(status)) => Some(Err(status)),
                None => None,
            }
        };
        if let Some(res) = matched {
            let (status_code, content_type, body_ptr, body_len, written) = res?;
            let cap = msg_buf.len();
            if body_len == 0 {
                return Ok((status_code, content_type, cap..cap));
            }
            let buf_start = msg_buf.as_ptr() as usize;
            let buf_end = buf_start + cap;
            if body_ptr >= buf_start && body_ptr + body_len <= buf_end {
                let start = body_ptr - buf_start;
                return Ok((status_code, content_type, start..(start + body_len)));
            }
            if written == 0 && body_len <= cap && body_len <= scratch.len() {
                let start = cap - body_len;
                msg_buf[start..cap].copy_from_slice(&scratch[..body_len]);
                return Ok((status_code, content_type, start..cap));
            }
            return Err(GrpcStatus::resource_exhausted(
                "HTTP response body exceeds buffer capacity",
            ));
        }
    }
    Err(GrpcStatus::not_found("not found"))
}

/// Asynchronously dispatches an HTTP API request to `router` and returns `(status_code, content_type, range_in_msg_buf)`.
async fn dispatch_http_async<'a, 'm, R: AsyncRouter>(
    router: &R,
    req: &crate::http::HttpRequest<'a>,
    scratch: &mut [u8],
    msg_buf: &'m mut [u8],
) -> Result<(u16, &'static str, core::ops::Range<usize>), GrpcStatus<'static>> {
    let cap = msg_buf.len();
    let buf_start = msg_buf.as_ptr() as usize;
    let buf_end = buf_start + cap;
    let mut writer = BackwardWriter::new(msg_buf);
    let Some(res) = router.dispatch_http_async(req, scratch, &mut writer).await else {
        return Err(GrpcStatus::not_found("not found"));
    };
    let resp = res?;
    let body_len = resp.body.len();
    let status_code = resp.status_code;
    let content_type = resp.content_type;
    let body_ptr = resp.body.as_ptr() as usize;
    if body_len == 0 {
        return Ok((status_code, content_type, cap..cap));
    }
    if body_ptr >= buf_start && body_ptr + body_len <= buf_end {
        let start = body_ptr - buf_start;
        return Ok((status_code, content_type, start..(start + body_len)));
    }
    if writer.written_len() == 0 && body_len <= cap && body_len <= scratch.len() {
        scratch[..body_len].copy_from_slice(resp.body);
        let _ = resp;
        if writer.write_bytes(&scratch[..body_len]).is_ok() {
            let start = cap - body_len;
            return Ok((status_code, content_type, start..cap));
        }
    }
    Err(GrpcStatus::resource_exhausted(
        "HTTP response body exceeds buffer capacity",
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::{decode_grpc_frame, write_grpc_frame_bytes};
    use crate::h2::{
        FrameHeader, FrameType, CLIENT_CONNECTION_PREFACE, FLAG_END_HEADERS, FLAG_END_STREAM,
        FRAME_HEADER_LEN,
    };
    use crate::hpack::{HpackDecoder, HpackEncoder};
    use crate::http::{
        parse_u64, split_custom_verb, write_str_field, write_varint_field, HttpMethod, HttpRequest,
        HttpResponse, PathCursor, QueryIter, CONTENT_TYPE_PROTOBUF, CONTENT_TYPE_TEXT_PLAIN,
    };
    use crate::status::GrpcCode;
    use roto_runtime::{decode_str_at, decode_varint_at, FieldIter, FieldMarker, WireType};

    struct ItemService;
    struct UserService;
    struct OrderService;
    struct AuditService;

    fn decode_item_fields(buf: &[u8]) -> Result<(u64, &str, &str), GrpcStatus<'static>> {
        let mut id = FieldMarker::none();
        let mut name = FieldMarker::none();
        let mut extra = FieldMarker::none();
        for item in FieldIter::new(buf) {
            let f = item.map_err(GrpcStatus::from)?;
            match (f.tag, f.wire_type) {
                (1, WireType::Varint) => id = f.marker,
                (2, WireType::LengthDelimited) => name = f.marker,
                (3, WireType::LengthDelimited) => extra = f.marker,
                _ => {}
            }
        }
        let id_val = if id.is_present() {
            decode_varint_at(buf, id).map_err(GrpcStatus::from)?
        } else {
            0
        };
        let name_val = if name.is_present() {
            decode_str_at(buf, name).map_err(GrpcStatus::from)?
        } else {
            ""
        };
        let extra_val = if extra.is_present() {
            decode_str_at(buf, extra).map_err(GrpcStatus::from)?
        } else {
            ""
        };
        Ok((id_val, name_val, extra_val))
    }

    fn encode_item_response<'b>(
        writer: &mut BackwardWriter<'b>,
        id: u64,
        name: &str,
        extra: &str,
    ) -> Result<&'b [u8], GrpcStatus<'static>> {
        write_str_field(writer, 3, extra)?;
        write_str_field(writer, 2, name)?;
        write_varint_field(writer, 1, id)?;
        crate::framing::take_writer(writer)
            .finish()
            .map_err(GrpcStatus::from)
    }

    impl GrpcService for ItemService {
        const SERVICE_NAME: &'static str = "test.v1.ItemService";

        fn call_unary(
            &self,
            method_name: &str,
            req_payload: &[u8],
            writer: &mut BackwardWriter<'_>,
        ) -> Result<(), GrpcStatus<'static>> {
            match method_name {
                "GetItem" => {
                    let (id, name, extra) = decode_item_fields(req_payload)?;
                    if id == 403 {
                        return Err(GrpcStatus::permission_denied("item access denied"));
                    }
                    let mut tmp = [0u8; 256];
                    let mut w = BackwardWriter::new(&mut tmp);
                    let encoded = encode_item_response(&mut w, id + 100, name, extra)?;
                    write_grpc_frame_bytes(writer, false, encoded).map_err(GrpcStatus::from)
                }
                _ => Err(GrpcStatus::unimplemented("unknown method")),
            }
        }

        fn call_http<'a, 's, 'b>(
            &self,
            req: &HttpRequest<'a>,
            scratch: &'s mut [u8],
            writer: &mut BackwardWriter<'b>,
        ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
            let (base_path, verb) = split_custom_verb(req.path);
            let mut cur = PathCursor::new(base_path)?;
            if !cur.literal("v1") || !cur.literal("items") {
                return None;
            }
            if cur.is_end() && verb.is_none() && req.method == HttpMethod::Post {
                // POST /v1/items with body `*`
                let res = (|| {
                    let (id, name, extra) = decode_item_fields(req.body)?;
                    let body = encode_item_response(writer, id + 1, name, extra)?;
                    Ok(HttpResponse::ok_protobuf(body))
                })();
                return Some(res);
            }
            let id_seg = cur.capture_single()?;
            if !cur.is_end() {
                return None;
            }
            if let Some(v) = verb {
                if v == "archive" && req.method == HttpMethod::Post {
                    let res = (|| {
                        let id = parse_u64(id_seg)?;
                        let body = encode_item_response(writer, id, "archived", v)?;
                        Ok(HttpResponse::ok_protobuf(body))
                    })();
                    return Some(res);
                }
                return None;
            }
            match req.method {
                HttpMethod::Get => {
                    let res = (|| {
                        let id = parse_u64(id_seg)?;
                        if id == 403 {
                            return Err(GrpcStatus::permission_denied("item access denied"));
                        }
                        let mut filter = "";
                        for (k, v) in QueryIter::new(req.query) {
                            if k == "filter" {
                                filter = v;
                            }
                        }
                        let _ = scratch;
                        let body = encode_item_response(writer, id, "fetched", filter)?;
                        Ok(HttpResponse::ok_protobuf(body))
                    })();
                    Some(res)
                }
                HttpMethod::Put | HttpMethod::Patch => {
                    let res = (|| {
                        let id = parse_u64(id_seg)?;
                        let (_, name, extra) = decode_item_fields(req.body)?;
                        let body = encode_item_response(writer, id, name, extra)?;
                        Ok(HttpResponse::ok_protobuf(body))
                    })();
                    Some(res)
                }
                HttpMethod::Delete => {
                    let res = (|| {
                        let id = parse_u64(id_seg)?;
                        let body = encode_item_response(writer, id, "deleted", "")?;
                        Ok(HttpResponse::ok_protobuf(body))
                    })();
                    Some(res)
                }
                _ => None,
            }
        }
    }

    impl AsyncGrpcService for ItemService {
        const SERVICE_NAME: &'static str = "test.v1.ItemService";

        async fn call_unary_async<'a, 'b>(
            &'a self,
            method_name: &'a str,
            req_payload: &'a [u8],
            writer: &'a mut BackwardWriter<'b>,
        ) -> Result<(), GrpcStatus<'static>> {
            self.call_unary(method_name, req_payload, writer)
        }

        async fn call_http_async<'a, 's, 'b>(
            &'a self,
            req: &'a HttpRequest<'a>,
            scratch: &'s mut [u8],
            writer: &'a mut BackwardWriter<'b>,
        ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>
        where
            's: 'a,
        {
            self.call_http(req, scratch, writer)
        }
    }

    macro_rules! impl_simple_http_service {
        ($ty:ident, $svc_name:expr, $prefix:expr, $tag_str:expr) => {
            impl GrpcService for $ty {
                const SERVICE_NAME: &'static str = $svc_name;

                fn call_unary(
                    &self,
                    _method_name: &str,
                    _req_payload: &[u8],
                    _writer: &mut BackwardWriter<'_>,
                ) -> Result<(), GrpcStatus<'static>> {
                    Err(GrpcStatus::unimplemented("unimplemented"))
                }

                fn call_http<'a, 's, 'b>(
                    &self,
                    req: &HttpRequest<'a>,
                    _scratch: &'s mut [u8],
                    _writer: &mut BackwardWriter<'b>,
                ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
                    if req.path == $prefix && req.method == HttpMethod::Get {
                        Some(Ok(HttpResponse::new(
                            200,
                            CONTENT_TYPE_TEXT_PLAIN,
                            $tag_str.as_bytes(),
                        )))
                    } else {
                        None
                    }
                }
            }

            impl AsyncGrpcService for $ty {
                const SERVICE_NAME: &'static str = $svc_name;

                async fn call_unary_async<'a, 'b>(
                    &'a self,
                    method_name: &'a str,
                    req_payload: &'a [u8],
                    writer: &'a mut BackwardWriter<'b>,
                ) -> Result<(), GrpcStatus<'static>> {
                    self.call_unary(method_name, req_payload, writer)
                }

                async fn call_http_async<'a, 's, 'b>(
                    &'a self,
                    req: &'a HttpRequest<'a>,
                    scratch: &'s mut [u8],
                    writer: &'a mut BackwardWriter<'b>,
                ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>
                where
                    's: 'a,
                {
                    self.call_http(req, scratch, writer)
                }
            }
        };
    }

    impl_simple_http_service!(UserService, "test.v1.UserService", "/v1/users", "user-ok");
    impl_simple_http_service!(
        OrderService,
        "test.v1.OrderService",
        "/v1/orders",
        "order-ok"
    );
    impl_simple_http_service!(
        AuditService,
        "test.v1.AuditService",
        "/v1/audits",
        "audit-ok"
    );

    #[test]
    fn route_and_async_router_tuple_dispatch_http() {
        let mut scratch = [0u8; 256];
        let mut msg_buf = [0u8; 256];

        // 2-tuple AsyncRouter
        let router2 = (ItemService, UserService);
        assert!(router2.is_grpc_service("test.v1.ItemService"));
        assert!(router2.is_grpc_service("test.v1.UserService"));
        assert!(!router2.is_grpc_service("test.v1.OrderService"));

        crate::transport::block_on_pair(
            async {
                let req_user = HttpRequest::from_uri(HttpMethod::Get, "/v1/users", None, &[]);
                {
                    let mut w = BackwardWriter::new(&mut msg_buf);
                    let res = router2
                        .dispatch_http_async(&req_user, &mut scratch, &mut w)
                        .await;
                    let resp = res.expect("matched").expect("ok");
                    assert_eq!(resp.status_code, 200);
                    assert_eq!(resp.body, b"user-ok");
                }

                // 3-tuple AsyncRouter
                let router3 = (ItemService, UserService, OrderService);
                assert!(router3.is_grpc_service("test.v1.OrderService"));
                assert!(!router3.is_grpc_service("test.v1.AuditService"));
                let req_order = HttpRequest::from_uri(HttpMethod::Get, "/v1/orders", None, &[]);
                {
                    let mut w = BackwardWriter::new(&mut msg_buf);
                    let res = router3
                        .dispatch_http_async(&req_order, &mut scratch, &mut w)
                        .await;
                    let resp = res.expect("matched").expect("ok");
                    assert_eq!(resp.body, b"order-ok");
                }

                // 4-tuple AsyncRouter
                let router4 = (ItemService, UserService, OrderService, AuditService);
                assert!(router4.is_grpc_service("test.v1.AuditService"));
                let req_audit = HttpRequest::from_uri(HttpMethod::Get, "/v1/audits", None, &[]);
                {
                    let mut w = BackwardWriter::new(&mut msg_buf);
                    let res = router4
                        .dispatch_http_async(&req_audit, &mut scratch, &mut w)
                        .await;
                    let resp = res.expect("matched").expect("ok");
                    assert_eq!(resp.body, b"audit-ok");
                }

                // Unmatched returns None
                let req_miss = HttpRequest::from_uri(HttpMethod::Get, "/v1/unknown", None, &[]);
                {
                    let mut w = BackwardWriter::new(&mut msg_buf);
                    let res = router4
                        .dispatch_http_async(&req_miss, &mut scratch, &mut w)
                        .await;
                    assert!(res.is_none());
                }
            },
            async {},
        );
    }

    fn drain_all_sync<const M: usize, const R: usize>(
        server: &mut GrpcServer<'_, M, R>,
        out: &mut [u8],
    ) -> usize {
        let mut total = 0;
        loop {
            let n = server.poll_transmit(&mut out[total..]);
            if n == 0 {
                break;
            }
            total += n;
        }
        total
    }

    fn drain_all_async<Rt: AsyncRouter, const M: usize, const R: usize>(
        server: &mut AsyncGrpcServer<'_, Rt, M, R>,
        out: &mut [u8],
    ) -> usize {
        let mut total = 0;
        loop {
            let n = server.poll_transmit(&mut out[total..]);
            if n == 0 {
                break;
            }
            total += n;
        }
        total
    }

    fn split_h1_response(raw: &[u8]) -> (&str, &[u8]) {
        let pos = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("HTTP/1.1 response header terminator");
        let head = core::str::from_utf8(&raw[..pos]).expect("UTF-8 headers");
        let body = &raw[pos + 4..];
        (head, body)
    }

    #[test]
    fn h1_methods_verbs_query_keepalive_close_and_errors_sync_and_async() {
        let routes: [&dyn Route; 2] = [&ItemService, &UserService];
        let mut msg_buf = [0u8; 1024];
        let mut tx_buf = [0u8; 2048];
        let mut wire = [0u8; 2048];
        let mut server: GrpcServer<4, 1024> = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);

        // 1. Fragmented 1-byte-at-a-time HTTP/1.1 GET with query parameter (keep-alive default)
        let get_req = b"GET /v1/items/42?filter=active HTTP/1.1\r\nHost: localhost\r\n\r\n";
        for &b in get_req {
            server.recv(&[b]).expect("recv 1 byte");
        }
        assert!(!server.should_close());
        let n = drain_all_sync(&mut server, &mut wire);
        assert!(!server.should_close());
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(head.contains("Content-Type: application/x-protobuf\r\n"));
        assert!(head.contains("Connection: keep-alive"));
        let (id, name, extra) = decode_item_fields(body).expect("decode protobuf body");
        assert_eq!((id, name, extra), (42, "fetched", "active"));

        // 2. POST /v1/items with binary protobuf body on the same keep-alive connection
        let mut req_body_buf = [0u8; 128];
        let req_body = {
            let mut w = BackwardWriter::new(&mut req_body_buf);
            encode_item_response(&mut w, 99, "created", "body-star").unwrap()
        };
        let mut post_wire = [0u8; 512];
        let post_hdr = b"POST /v1/items HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-protobuf\r\nContent-Length: ";
        let mut pos = 0;
        post_wire[..post_hdr.len()].copy_from_slice(post_hdr);
        pos += post_hdr.len();
        let len_digits = [
            b'0' + (req_body.len() / 10) as u8,
            b'0' + (req_body.len() % 10) as u8,
        ];
        post_wire[pos..pos + 2].copy_from_slice(&len_digits);
        pos += 2;
        post_wire[pos..pos + 4].copy_from_slice(b"\r\n\r\n");
        pos += 4;
        post_wire[pos..pos + req_body.len()].copy_from_slice(req_body);
        pos += req_body.len();

        server.recv(&post_wire[..pos]).expect("recv POST");
        let n = drain_all_sync(&mut server, &mut wire);
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
        let (id, name, extra) = decode_item_fields(body).expect("decode POST response");
        assert_eq!((id, name, extra), (100, "created", "body-star"));

        // 3. Custom `:verb` (POST /v1/items/77:archive), PUT, PATCH, DELETE, and 404 Not Found
        let custom_req =
            b"POST /v1/items/77:archive HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n";
        server.recv(custom_req).expect("recv custom verb");
        let n = drain_all_sync(&mut server, &mut wire);
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
        let (id, name, extra) = decode_item_fields(body).expect("decode custom verb");
        assert_eq!((id, name, extra), (77, "archived", "archive"));

        let del_req = b"DELETE /v1/items/55 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        server.recv(del_req).expect("recv DELETE");
        let n = drain_all_sync(&mut server, &mut wire);
        let (_, body) = split_h1_response(&wire[..n]);
        let (id, name, _) = decode_item_fields(body).expect("decode DELETE");
        assert_eq!((id, name), (55, "deleted"));

        // 4. Handler error status mapping (403 Forbidden) and 404 Not Found
        let forbidden_req = b"GET /v1/items/403 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        server.recv(forbidden_req).expect("recv 403");
        let n = drain_all_sync(&mut server, &mut wire);
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 403 Forbidden\r\n"));
        assert_eq!(body, b"item access denied");

        let not_found_req =
            b"GET /v1/nonexistent HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        server.recv(not_found_req).expect("recv 404 with close");
        assert!(!server.should_close()); // Not until outbound is drained!
        let n = drain_all_sync(&mut server, &mut wire);
        assert!(server.should_close());
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(head.contains("Connection: close"));
        assert_eq!(body, b"not found");

        // 5. Reset and verify AsyncGrpcServer handles pipelined HTTP/1.1 requests and 400 Bad Request
        let router = (ItemService, UserService);
        let mut async_msg_buf = [0u8; 1024];
        let mut async_tx_buf = [0u8; 2048];
        let mut async_server: AsyncGrpcServer<_, 4, 1024> =
            AsyncGrpcServer::new(&router, &mut async_msg_buf, &mut async_tx_buf);

        crate::transport::block_on_pair(
            async {
                let pipelined = b"GET /v1/users HTTP/1.1\r\nHost: localhost\r\n\r\nGET /v1/items/12?filter=piped HTTP/1.1\r\nHost: localhost\r\n\r\n";
                for chunk in pipelined.chunks(7) {
                    async_server.recv(chunk).await.expect("async recv chunk");
                }
                let n = drain_all_async(&mut async_server, &mut wire);
                let wire_str = core::str::from_utf8(&wire[..n]).unwrap_or("");
                assert!(wire_str.starts_with("HTTP/1.1 200 OK\r\n"));
                assert!(wire[..n].windows(7).any(|w| w == b"user-ok"));

                // Send malformed request on the same connection -> 400 Bad Request & close
                async_server
                    .recv(b"GET /v1/users HTTP/1.1\r\nContent-Length: notanumber\r\n\r\n")
                    .await
                    .expect("async recv bad request");
                let n = drain_all_async(&mut async_server, &mut wire);
                assert!(async_server.should_close());
                let (head, _) = split_h1_response(&wire[..n]);
                assert!(head.starts_with("HTTP/1.1 400 Bad Request\r\n"));
                assert!(head.contains("Connection: close"));
            },
            async {},
        );
    }

    fn encode_h2_http_request_headers(
        method: &[u8],
        path: &[u8],
        content_type: Option<&[u8]>,
        end_stream: bool,
        stream_id: u32,
        out: &mut [u8],
    ) -> usize {
        let enc = HpackEncoder::new();
        let mut pos = FRAME_HEADER_LEN;
        pos += enc
            .encode_literal(b":method", method, false, &mut out[pos..])
            .unwrap();
        pos += enc
            .encode_literal(b":scheme", b"http", false, &mut out[pos..])
            .unwrap();
        pos += enc
            .encode_literal(b":path", path, false, &mut out[pos..])
            .unwrap();
        pos += enc
            .encode_literal(b":authority", b"localhost", false, &mut out[pos..])
            .unwrap();
        if let Some(ct) = content_type {
            pos += enc
                .encode_literal(b"content-type", ct, false, &mut out[pos..])
                .unwrap();
        }
        let block_len = pos - FRAME_HEADER_LEN;
        let flags = FLAG_END_HEADERS | if end_stream { FLAG_END_STREAM } else { 0 };
        let hdr = FrameHeader::new(block_len as u32, FrameType::HEADERS, flags, stream_id);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        pos
    }

    fn encode_h2_data_frame(
        payload: &[u8],
        end_stream: bool,
        stream_id: u32,
        out: &mut [u8],
    ) -> usize {
        let flags = if end_stream { FLAG_END_STREAM } else { 0 };
        let hdr = FrameHeader::new(payload.len() as u32, FrameType::DATA, flags, stream_id);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        out[FRAME_HEADER_LEN..FRAME_HEADER_LEN + payload.len()].copy_from_slice(payload);
        FRAME_HEADER_LEN + payload.len()
    }

    struct DecodedH2Stream {
        status: Option<u16>,
        content_type: [u8; 64],
        content_type_len: usize,
        grpc_status: Option<GrpcCode>,
        data: [u8; 512],
        data_len: usize,
        headers_count: usize,
        data_frames: usize,
        end_stream_seen: bool,
    }

    impl DecodedH2Stream {
        const fn new() -> Self {
            Self {
                status: None,
                content_type: [0u8; 64],
                content_type_len: 0,
                grpc_status: None,
                data: [0u8; 512],
                data_len: 0,
                headers_count: 0,
                data_frames: 0,
                end_stream_seen: false,
            }
        }
    }

    fn parse_h2_server_frames(mut bytes: &[u8], stream_id: u32) -> DecodedH2Stream {
        let mut out = DecodedH2Stream::new();
        let mut dec = HpackDecoder::<0>::stateless();
        let mut scratch = [0u8; 256];
        while bytes.len() >= FRAME_HEADER_LEN {
            let hdr_arr: [u8; 9] = bytes[..9].try_into().unwrap();
            let hdr = FrameHeader::decode(hdr_arr);
            let len = hdr.length as usize;
            let payload = &bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + len];
            bytes = &bytes[FRAME_HEADER_LEN + len..];
            if hdr.stream_id != stream_id {
                continue;
            }
            if (hdr.flags & FLAG_END_STREAM) != 0 {
                out.end_stream_seen = true;
            }
            match hdr.frame_type {
                FrameType::HEADERS => {
                    out.headers_count += 1;
                    dec.decode(payload, &mut scratch, |hf| {
                        if hf.name == b":status" {
                            if let Ok(s) = core::str::from_utf8(hf.value) {
                                if let Ok(code) = parse_u64(s) {
                                    out.status = Some(code as u16);
                                }
                            }
                        } else if hf.name == b"content-type" {
                            let n = hf.value.len().min(out.content_type.len());
                            out.content_type[..n].copy_from_slice(&hf.value[..n]);
                            out.content_type_len = n;
                        } else if hf.name == b"grpc-status" {
                            out.grpc_status = GrpcCode::from_ascii_bytes(hf.value);
                        }
                        Ok(())
                    })
                    .expect("valid HPACK headers");
                }
                FrameType::DATA => {
                    out.data_frames += 1;
                    out.data[out.data_len..out.data_len + payload.len()].copy_from_slice(payload);
                    out.data_len += payload.len();
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn h2_http_api_and_multiplexed_grpc_calls_plus_protocol_reset() {
        let routes: [&dyn Route; 2] = [&ItemService, &UserService];
        let mut msg_buf = [0u8; 1024];
        let mut tx_buf = [0u8; 4096];
        let mut wire = [0u8; 4096];
        let mut server: GrpcServer<8, 1024> = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);

        // Start with an HTTP/1.1 connection first, then reset() and switch to HTTP/2!
        server
            .recv(b"GET /v1/users HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        let n = drain_all_sync(&mut server, &mut wire);
        assert!(server.should_close());
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
        assert_eq!(body, b"user-ok");

        // Reset server and switch to HTTP/2 prior-knowledge connection
        server.reset();
        assert!(!server.should_close());

        let mut client_in = [0u8; 2048];
        let mut in_pos = 0;
        client_in[..CLIENT_CONNECTION_PREFACE.len()].copy_from_slice(CLIENT_CONNECTION_PREFACE);
        in_pos += CLIENT_CONNECTION_PREFACE.len();
        // Empty client SETTINGS frame
        let settings_hdr = FrameHeader::new(0, FrameType::SETTINGS, 0, 0).encode();
        client_in[in_pos..in_pos + 9].copy_from_slice(&settings_hdr);
        in_pos += 9;

        // Stream 1: HTTP/2 GET /v1/items/50?filter=h2 (non-gRPC HTTP API call, END_STREAM on HEADERS)
        in_pos += encode_h2_http_request_headers(
            b"GET",
            b"/v1/items/50?filter=h2",
            None,
            true,
            1,
            &mut client_in[in_pos..],
        );

        // Stream 3: Standard gRPC unary call POST /test.v1.ItemService/GetItem (5-byte LPM framed)
        let mut grpc_req_buf = [0u8; 128];
        let grpc_lpm = {
            let mut tmp = [0u8; 64];
            let mut w = BackwardWriter::new(&mut tmp);
            let proto = encode_item_response(&mut w, 7, "grpc-item", "lpm").unwrap();
            let mut fw = BackwardWriter::new(&mut grpc_req_buf);
            write_grpc_frame_bytes(&mut fw, false, proto).unwrap();
            fw.finish().unwrap()
        };
        in_pos += HpackEncoder::new()
            .encode_grpc_request_headers(
                b"http",
                b"localhost",
                b"/test.v1.ItemService/GetItem",
                b"application/grpc+proto",
                &mut client_in[in_pos + FRAME_HEADER_LEN..],
            )
            .map(|block_len| {
                let hdr =
                    FrameHeader::new(block_len as u32, FrameType::HEADERS, FLAG_END_HEADERS, 3);
                client_in[in_pos..in_pos + 9].copy_from_slice(&hdr.encode());
                9 + block_len
            })
            .unwrap();
        in_pos += encode_h2_data_frame(grpc_lpm, true, 3, &mut client_in[in_pos..]);

        // Stream 5: HTTP/2 POST /v1/items with raw (non-LPM) protobuf body split across 2 DATA frames + empty END_STREAM DATA frame
        let mut raw_proto_buf = [0u8; 128];
        let raw_proto = {
            let mut w = BackwardWriter::new(&mut raw_proto_buf);
            encode_item_response(&mut w, 300, "h2-post", "raw-data").unwrap()
        };
        in_pos += encode_h2_http_request_headers(
            b"POST",
            b"/v1/items",
            Some(b"application/x-protobuf"),
            false,
            5,
            &mut client_in[in_pos..],
        );
        let split = raw_proto.len() / 2;
        in_pos += encode_h2_data_frame(&raw_proto[..split], false, 5, &mut client_in[in_pos..]);
        in_pos += encode_h2_data_frame(&raw_proto[split..], false, 5, &mut client_in[in_pos..]);
        in_pos += encode_h2_data_frame(&[], true, 5, &mut client_in[in_pos..]);

        // Stream 7: HTTP/2 GET /v1/items/403 -> 403 Forbidden HTTP error response
        in_pos += encode_h2_http_request_headers(
            b"GET",
            b"/v1/items/403",
            None,
            true,
            7,
            &mut client_in[in_pos..],
        );

        // Stream 9: HTTP/2 GET /v1/unknown -> 404 Not Found HTTP error response
        in_pos += encode_h2_http_request_headers(
            b"GET",
            b"/v1/unknown",
            None,
            true,
            9,
            &mut client_in[in_pos..],
        );

        server.recv(&client_in[..in_pos]).expect("recv H2 batch");
        let out_n = drain_all_sync(&mut server, &mut wire);

        // Verify Stream 1 (HTTP/2 GET): raw protobuf DATA, no gRPC trailers (headers_count == 1)
        let s1 = parse_h2_server_frames(&wire[..out_n], 1);
        assert_eq!(s1.status, Some(200));
        assert_eq!(
            &s1.content_type[..s1.content_type_len],
            CONTENT_TYPE_PROTOBUF.as_bytes()
        );
        assert_eq!(s1.headers_count, 1);
        assert!(s1.grpc_status.is_none());
        assert!(s1.end_stream_seen);
        let (id, name, extra) = decode_item_fields(&s1.data[..s1.data_len]).unwrap();
        assert_eq!((id, name, extra), (50, "fetched", "h2"));

        // Verify Stream 3 (Standard gRPC unary): LPM-framed DATA + gRPC trailers (headers_count == 2)
        let s3 = parse_h2_server_frames(&wire[..out_n], 3);
        assert_eq!(s3.status, Some(200));
        assert_eq!(s3.headers_count, 2);
        assert_eq!(s3.grpc_status, Some(GrpcCode::Ok));
        let (grpc_frame, rem) = decode_grpc_frame(&s3.data[..s3.data_len], 4096)
            .unwrap()
            .unwrap();
        assert!(!grpc_frame.compressed);
        assert!(rem.is_empty());
        let (id, name, extra) = decode_item_fields(grpc_frame.payload).unwrap();
        assert_eq!((id, name, extra), (107, "grpc-item", "lpm"));

        // Verify Stream 5 (HTTP/2 POST with fragmented raw DATA): raw protobuf response, headers_count == 1
        let s5 = parse_h2_server_frames(&wire[..out_n], 5);
        assert_eq!(s5.status, Some(200));
        assert_eq!(s5.headers_count, 1);
        assert!(s5.end_stream_seen);
        let (id, name, extra) = decode_item_fields(&s5.data[..s5.data_len]).unwrap();
        assert_eq!((id, name, extra), (301, "h2-post", "raw-data"));

        // Verify Stream 7 (403 Forbidden) and Stream 9 (404 Not Found)
        let s7 = parse_h2_server_frames(&wire[..out_n], 7);
        assert_eq!(s7.status, Some(403));
        assert_eq!(s7.headers_count, 1);
        assert_eq!(&s7.data[..s7.data_len], b"item access denied");

        let s9 = parse_h2_server_frames(&wire[..out_n], 9);
        assert_eq!(s9.status, Some(404));
        assert_eq!(s9.headers_count, 1);
        assert_eq!(&s9.data[..s9.data_len], b"not found");

        // Reset server back to HTTP/1.1 and verify clean HTTP/1.1 handling again
        server.reset();
        server
            .recv(b"GET /v1/items/88?filter=back-to-h1 HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let n = drain_all_sync(&mut server, &mut wire);
        let (head, body) = split_h1_response(&wire[..n]);
        assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
        let (id, name, extra) = decode_item_fields(body).unwrap();
        assert_eq!((id, name, extra), (88, "fetched", "back-to-h1"));
    }
}
