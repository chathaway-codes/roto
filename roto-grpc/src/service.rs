use core::future::Future;

use roto_runtime::BackwardWriter;

use crate::framing::{decode_grpc_frame, encode_grpc_message, DEFAULT_MAX_MESSAGE_SIZE};
use crate::metadata::{
    FixedMetadata, MetadataMap, RpcContext, RpcMetadata, EMPTY_METADATA,
};
use crate::status::GrpcStatus;

/// Splits an HTTP/2 `:path` pseudo-header of the form `"/{service_name}/{method_name}"`
/// into `Some((service_name, method_name))`.
///
/// Returns `None` if `path` does not begin with `'/'` or does not contain exactly one
/// subsequent `'/'` separating non-empty `service_name` and `method_name` segments.
#[inline]
pub fn split_grpc_path(path: &str) -> Option<(&str, &str)> {
    let without_leading = path.strip_prefix('/')?;
    let (service_name, method_name) = without_leading.split_once('/')?;
    if service_name.is_empty() || method_name.is_empty() || method_name.contains('/') {
        return None;
    }
    Some((service_name, method_name))
}

/// Extracts `method_name` from an HTTP/2 `:path` (`"/{expected_service}/{method_name}"`)
/// if and only if the service segment matches `expected_service`.
#[inline]
pub fn extract_method_name<'a>(path: &'a str, expected_service: &str) -> Option<&'a str> {
    let (svc, method) = split_grpc_path(path)?;
    if svc == expected_service {
        Some(method)
    } else {
        None
    }
}

/// Synchronous `#![no_std]` zero-allocation gRPC service dispatch trait implemented by
/// generated `<Service>Server<T>` wrappers.
pub trait GrpcService {
    /// Full protobuf service name: `"<package>.<Service>"` (or `"<Service>"` if package is empty).
    const SERVICE_NAME: &'static str;

    /// Wire-encoded `FileDescriptorProto` slices for this service and its transitive dependencies
    /// when generated with `descriptors=true` (empty by default).
    const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = &[];

    /// Returns `true` if `method_name` is a client-streaming or bidirectional-streaming RPC
    /// accepting zero or more LPM-framed request messages.
    #[inline]
    fn is_client_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Returns `true` if `method_name` is a bidirectional-streaming RPC
    /// (`rpc Foo(stream Req) returns (stream Resp)`), which streams responses interactively
    /// as request messages arrive rather than waiting for `END_STREAM`.
    #[inline]
    fn is_bidi_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Dispatches a unary or server-streaming RPC call for `method_name` (the part after `"/{SERVICE_NAME}/"`).
    ///
    /// `req_payload` is the raw protobuf message slice (with the 5-byte gRPC LPM header already stripped).
    /// The implementation decodes the request view, invokes the service handler to populate the response
    /// builder (or `SendStream`) on `writer`, and prepends the 5-byte gRPC LPM header(s) onto `writer`.
    fn call_unary(
        &self,
        meta: &mut RpcMetadata<'_>,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>>;

    /// Dispatches a client-streaming or bidirectional-streaming RPC call for `method_name`.
    ///
    /// `req_frames` contains zero or more concatenated 5-byte LPM-framed request messages.
    #[inline]
    fn call_stream(
        &self,
        meta: &mut RpcMetadata<'_>,
        method_name: &str,
        req_frames: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let _ = (meta, method_name, req_frames, writer);
        Err(GrpcStatus::unimplemented("unimplemented gRPC method"))
    }

    /// Dispatches a unary or server-streaming RPC call using the full HTTP/2 `:path` pseudo-header
    /// (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_path(
        &self,
        meta: &mut RpcMetadata<'_>,
        path: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
            return Err(GrpcStatus::unimplemented(
                "unimplemented gRPC service or path",
            ));
        };
        self.call_unary(meta, method_name, req_payload, writer)
    }

    /// Dispatches a client-streaming or bidirectional-streaming RPC call using the full HTTP/2
    /// `:path` pseudo-header (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_stream_path(
        &self,
        meta: &mut RpcMetadata<'_>,
        path: &str,
        req_frames: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
            return Err(GrpcStatus::unimplemented(
                "unimplemented gRPC service or path",
            ));
        };
        self.call_stream(meta, method_name, req_frames, writer)
    }

    /// Dispatches an HTTP API (`google.api.http`) request when `http` codegen is enabled.
    ///
    /// Returns `None` if `req` did not match any HTTP route on this service.
    #[inline]
    fn call_http<'a, 's, 'b>(
        &self,
        req: &crate::http::HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>> {
        let _ = (req, scratch, writer);
        None
    }
}

/// Asynchronous `#![no_std]` zero-allocation gRPC service dispatch trait implemented by
/// generated `<Service>Server<T>` wrappers.
pub trait AsyncGrpcService {
    /// Full protobuf service name: `"<package>.<Service>"` (or `"<Service>"` if package is empty).
    const SERVICE_NAME: &'static str;

    /// Wire-encoded `FileDescriptorProto` slices for this service and its transitive dependencies
    /// when generated with `descriptors=true` (empty by default).
    const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = &[];

    /// Returns `true` if `method_name` is a client-streaming or bidirectional-streaming RPC
    /// accepting zero or more LPM-framed request messages.
    #[inline]
    fn is_client_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Returns `true` if `method_name` is a bidirectional-streaming RPC
    /// (`rpc Foo(stream Req) returns (stream Resp)`), which streams responses interactively
    /// as request messages arrive rather than waiting for `END_STREAM`.
    #[inline]
    fn is_bidi_streaming(&self, method_name: &str) -> bool {
        let _ = method_name;
        false
    }

    /// Asynchronously dispatches a unary or server-streaming RPC call for `method_name`
    /// (the part after `"/{SERVICE_NAME}/"`).
    ///
    /// `req_payload` is the raw protobuf message slice (with the 5-byte gRPC LPM header already stripped).
    fn call_unary_async<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a;

    /// Asynchronously dispatches a client-streaming or bidirectional-streaming RPC call for `method_name`.
    ///
    /// `req_frames` contains zero or more concatenated 5-byte LPM-framed request messages
    /// received on the stream prior to `END_STREAM`.
    #[inline]
    fn call_stream_async<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        method_name: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a {
        async move {
            let _ = (meta, method_name, req_frames, writer);
            Err(GrpcStatus::unimplemented("unimplemented gRPC method"))
        }
    }

    /// Asynchronously dispatches a unary or server-streaming RPC call using the full HTTP/2
    /// `:path` pseudo-header (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_path_async<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        path: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a {
        async move {
            let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
                return Err(GrpcStatus::unimplemented(
                    "unimplemented gRPC service or path",
                ));
            };
            self.call_unary_async(meta, method_name, req_payload, writer)
                .await
        }
    }

    /// Asynchronously dispatches a client-streaming or bidirectional-streaming RPC call using
    /// the full HTTP/2 `:path` pseudo-header (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_stream_path_async<'a, 'b>(
        &'a self,
        meta: &'a mut RpcMetadata<'_>,
        path: &'a str,
        req_frames: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a {
        async move {
            let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
                return Err(GrpcStatus::unimplemented(
                    "unimplemented gRPC service or path",
                ));
            };
            self.call_stream_async(meta, method_name, req_frames, writer)
                .await
        }
    }

    /// Asynchronously dispatches an HTTP API (`google.api.http`) request when `http` codegen is enabled.
    ///
    /// Returns `None` if `req` did not match any HTTP route on this service.
    #[inline]
    fn call_http_async<'a, 's, 'b>(
        &'a self,
        req: &'a crate::http::HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Option<Result<crate::http::HttpResponse<'b>, GrpcStatus<'static>>>> + 'a
    where
        's: 'a,
    {
        async move {
            let _ = (req, scratch, writer);
            None
        }
    }
}

/// Common request/response metadata accessors shared by synchronous ([`UnaryChannel`]) and
/// asynchronous ([`AsyncUnaryChannel`]) client channels.
pub trait GrpcChannel {
    /// Returns a mutable reference to the outbound request metadata store to be sent with the
    /// next RPC call, if supported by the channel.
    #[inline]
    fn request_metadata_mut(&mut self) -> Option<&mut dyn MetadataMap> {
        None
    }

    /// Returns the inbound initial response metadata headers received on the most recent RPC call.
    #[inline]
    fn response_headers(&self) -> &dyn MetadataMap {
        &EMPTY_METADATA
    }

    /// Returns the inbound trailing response metadata received on the most recent RPC call.
    #[inline]
    fn response_trailers(&self) -> &dyn MetadataMap {
        &EMPTY_METADATA
    }
}

impl<C: GrpcChannel + ?Sized> GrpcChannel for &mut C {
    #[inline]
    fn request_metadata_mut(&mut self) -> Option<&mut dyn MetadataMap> {
        (**self).request_metadata_mut()
    }

    #[inline]
    fn response_headers(&self) -> &dyn MetadataMap {
        (**self).response_headers()
    }

    #[inline]
    fn response_trailers(&self) -> &dyn MetadataMap {
        (**self).response_trailers()
    }
}

/// Synchronous `#![no_std]` zero-allocation client channel trait used by generated
/// `<Service>Client<C>` stubs for unary and streaming RPCs.
pub trait UnaryChannel: GrpcChannel {
    /// Executes a unary gRPC call to `path` (e.g. `"/pkg.Service/Method"`).
    ///
    /// `encode_req` writes the protobuf request fields into a [`BackwardWriter`]; the channel
    /// frames the 5-byte gRPC LPM header, transmits the request, receives the response, strips
    /// the response 5-byte LPM header, and returns the raw response protobuf payload slice
    /// borrowing from `self`.
    fn call_unary<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>);

    /// Executes a unary gRPC call to `path` using pre-serialized request payload bytes `req_bytes`.
    #[inline]
    fn call_unary_bytes<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &[u8],
    ) -> Result<&'a [u8], GrpcStatus<'a>> {
        self.call_unary(path, |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Executes a server-streaming gRPC call to `path`, returning the concatenated 5-byte
    /// LPM-framed response messages received prior to `END_STREAM`.
    #[inline]
    fn call_server_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let _ = (path, encode_req);
        Err(GrpcStatus::unimplemented(
            "server-streaming not supported by channel",
        ))
    }

    /// Executes a server-streaming gRPC call to `path` using pre-serialized request payload bytes `req_bytes`.
    #[inline]
    fn call_server_streaming_bytes<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &[u8],
    ) -> Result<&'a [u8], GrpcStatus<'a>> {
        self.call_server_streaming(path, |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Executes a client-streaming gRPC call to `path` where `encode_reqs` writes zero or more
    /// 5-byte LPM-framed request messages into a [`BackwardWriter`], and returns the raw
    /// single-message response payload slice (with its 5-byte LPM header stripped).
    #[inline]
    fn call_client_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        let _ = (path, encode_reqs);
        Err(GrpcStatus::unimplemented(
            "client-streaming not supported by channel",
        ))
    }

    /// Executes a bidirectional-streaming gRPC call to `path` where `encode_reqs` writes zero
    /// or more 5-byte LPM-framed request messages into a [`BackwardWriter`], and returns the
    /// concatenated 5-byte LPM-framed response messages received prior to `END_STREAM`.
    #[inline]
    fn call_bidi_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        let _ = (path, encode_reqs);
        Err(GrpcStatus::unimplemented(
            "bidi-streaming not supported by channel",
        ))
    }
}

impl<C: UnaryChannel + ?Sized> UnaryChannel for &mut C {
    #[inline]
    fn call_unary<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        (**self).call_unary(path, encode_req)
    }

    #[inline]
    fn call_unary_bytes<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &[u8],
    ) -> Result<&'a [u8], GrpcStatus<'a>> {
        (**self).call_unary_bytes(path, req_bytes)
    }

    #[inline]
    fn call_server_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        (**self).call_server_streaming(path, encode_req)
    }

    #[inline]
    fn call_server_streaming_bytes<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &[u8],
    ) -> Result<&'a [u8], GrpcStatus<'a>> {
        (**self).call_server_streaming_bytes(path, req_bytes)
    }

    #[inline]
    fn call_client_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        (**self).call_client_streaming(path, encode_reqs)
    }

    #[inline]
    fn call_bidi_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        (**self).call_bidi_streaming(path, encode_reqs)
    }
}

/// Asynchronous `#![no_std]` zero-allocation client channel trait used by generated
/// `<Service>Client<C>` stubs for unary and streaming RPCs.
pub trait AsyncUnaryChannel: GrpcChannel {
    /// Asynchronously executes a unary gRPC call to `path` (e.g. `"/pkg.Service/Method"`).
    ///
    /// `encode_req` writes the protobuf request fields into a [`BackwardWriter`]; the channel
    /// frames the 5-byte gRPC LPM header, transmits the request, awaits the response, strips
    /// the response 5-byte LPM header, and returns the raw response protobuf payload slice
    /// borrowing from `self`.
    fn call_unary_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a;

    /// Asynchronously executes a unary gRPC call to `path` using pre-serialized request payload bytes `req_bytes`.
    #[inline]
    fn call_unary_bytes_async<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &'a [u8],
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
    {
        self.call_unary_async(path, move |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Asynchronously executes a server-streaming gRPC call to `path`, returning the
    /// concatenated 5-byte LPM-framed response messages received prior to `END_STREAM`.
    #[inline]
    fn call_server_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        async move {
            let _ = (path, encode_req);
            Err(GrpcStatus::unimplemented(
                "server-streaming not supported by channel",
            ))
        }
    }

    /// Asynchronously executes a server-streaming gRPC call to `path` using pre-serialized
    /// request payload bytes `req_bytes`.
    #[inline]
    fn call_server_streaming_bytes_async<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &'a [u8],
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
    {
        self.call_server_streaming_async(path, move |writer| {
            let _ = writer.write_bytes(req_bytes);
        })
    }

    /// Asynchronously executes a client-streaming gRPC call to `path` where `encode_reqs`
    /// writes zero or more 5-byte LPM-framed request messages into a [`BackwardWriter`],
    /// and returns the raw single-message response payload slice.
    #[inline]
    fn call_client_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        async move {
            let _ = (path, encode_reqs);
            Err(GrpcStatus::unimplemented(
                "client-streaming not supported by channel",
            ))
        }
    }

    /// Asynchronously executes a bidirectional-streaming gRPC call to `path` where
    /// `encode_reqs` writes zero or more 5-byte LPM-framed request messages into a
    /// [`BackwardWriter`], and returns the concatenated 5-byte LPM-framed response messages.
    #[inline]
    fn call_bidi_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        async move {
            let _ = (path, encode_reqs);
            Err(GrpcStatus::unimplemented(
                "bidi-streaming not supported by channel",
            ))
        }
    }
}

impl<C: AsyncUnaryChannel + ?Sized> AsyncUnaryChannel for &mut C {
    #[inline]
    fn call_unary_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        (**self).call_unary_async(path, encode_req)
    }

    #[inline]
    fn call_unary_bytes_async<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &'a [u8],
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
    {
        (**self).call_unary_bytes_async(path, req_bytes)
    }

    #[inline]
    fn call_server_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        (**self).call_server_streaming_async(path, encode_req)
    }

    #[inline]
    fn call_server_streaming_bytes_async<'a>(
        &'a mut self,
        path: &'static str,
        req_bytes: &'a [u8],
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
    {
        (**self).call_server_streaming_bytes_async(path, req_bytes)
    }

    #[inline]
    fn call_client_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        (**self).call_client_streaming_async(path, encode_reqs)
    }

    #[inline]
    fn call_bidi_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> impl Future<Output = Result<&'a [u8], GrpcStatus<'a>>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        (**self).call_bidi_streaming_async(path, encode_reqs)
    }
}

/// Zero-allocation in-memory loopback channel connecting a client stub directly to a
/// [`GrpcService`] or [`AsyncGrpcService`] using caller-provided request and response buffers.
#[derive(Debug)]
pub struct LoopbackChannel<'buf, S, M: MetadataMap = FixedMetadata> {
    pub service: S,
    pub req_buf: &'buf mut [u8],
    pub resp_buf: &'buf mut [u8],
    pub max_message_size: usize,
    pub metadata: RpcContext<M>,
}

impl<'buf, S> LoopbackChannel<'buf, S, FixedMetadata> {
    /// Creates a new [`LoopbackChannel`] with [`DEFAULT_MAX_MESSAGE_SIZE`] and default
    /// [`FixedMetadata`] storage.
    #[inline]
    pub fn new(service: S, req_buf: &'buf mut [u8], resp_buf: &'buf mut [u8]) -> Self {
        Self {
            service,
            req_buf,
            resp_buf,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            metadata: RpcContext::new(),
        }
    }
}

impl<'buf, S, M: MetadataMap + Default> LoopbackChannel<'buf, S, M> {
    /// Creates a new [`LoopbackChannel`] with [`DEFAULT_MAX_MESSAGE_SIZE`] and custom
    /// [`MetadataMap`] storage `M`.
    #[inline]
    pub fn with_metadata_store(
        service: S,
        req_buf: &'buf mut [u8],
        resp_buf: &'buf mut [u8],
    ) -> Self {
        Self {
            service,
            req_buf,
            resp_buf,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            metadata: RpcContext::new(),
        }
    }
}

impl<'buf, S, M: MetadataMap> LoopbackChannel<'buf, S, M> {
    /// Configures the maximum inbound/outbound message payload size enforced during LPM deframing.
    #[inline]
    pub const fn with_max_message_size(mut self, max_message_size: usize) -> Self {
        self.max_message_size = max_message_size;
        self
    }
}

impl<S, M: MetadataMap> GrpcChannel for LoopbackChannel<'_, S, M> {
    #[inline]
    fn request_metadata_mut(&mut self) -> Option<&mut dyn MetadataMap> {
        Some(&mut self.metadata.request)
    }

    #[inline]
    fn response_headers(&self) -> &dyn MetadataMap {
        &self.metadata.response_headers
    }

    #[inline]
    fn response_trailers(&self) -> &dyn MetadataMap {
        &self.metadata.response_trailers
    }
}

impl<S: GrpcService, M: MetadataMap> UnaryChannel for LoopbackChannel<'_, S, M> {
    fn call_unary<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let req_framed =
                encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
            let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
                .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
            if !req_rest.is_empty() {
                return Err(GrpcStatus::internal(
                    "unexpected trailing bytes after request LPM frame",
                ));
            }
            Ok(req_frame.payload)
        })();
        let req_payload = match req_res {
            Ok(payload) => payload,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_path(&mut meta, path, req_payload, &mut resp_writer)
        };
        self.metadata.request.clear();
        res?;
        let resp_framed = resp_writer.finish().map_err(GrpcStatus::from)?;
        let (resp_frame, resp_rest) = decode_grpc_frame(resp_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated response LPM frame"))?;
        if !resp_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after response LPM frame",
            ));
        }
        Ok(resp_frame.payload)
    }

    fn call_server_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let req_framed =
                encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
            let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
                .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
            if !req_rest.is_empty() {
                return Err(GrpcStatus::internal(
                    "unexpected trailing bytes after request LPM frame",
                ));
            }
            Ok(req_frame.payload)
        })();
        let req_payload = match req_res {
            Ok(payload) => payload,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_path(&mut meta, path, req_payload, &mut resp_writer)
        };
        self.metadata.request.clear();
        res?;
        resp_writer.finish().map_err(GrpcStatus::from)
    }

    fn call_client_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let mut req_writer = BackwardWriter::new(self.req_buf);
            encode_reqs(&mut req_writer)?;
            req_writer.finish().map_err(GrpcStatus::from)
        })();
        let req_frames = match req_res {
            Ok(frames) => frames,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_stream_path(&mut meta, path, req_frames, &mut resp_writer)
        };
        self.metadata.request.clear();
        res?;
        let resp_framed = resp_writer.finish().map_err(GrpcStatus::from)?;
        let (resp_frame, resp_rest) = decode_grpc_frame(resp_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated response LPM frame"))?;
        if !resp_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after response LPM frame",
            ));
        }
        Ok(resp_frame.payload)
    }

    fn call_bidi_streaming<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>>,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let mut req_writer = BackwardWriter::new(self.req_buf);
            encode_reqs(&mut req_writer)?;
            req_writer.finish().map_err(GrpcStatus::from)
        })();
        let req_frames = match req_res {
            Ok(frames) => frames,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_stream_path(&mut meta, path, req_frames, &mut resp_writer)
        };
        self.metadata.request.clear();
        res?;
        resp_writer.finish().map_err(GrpcStatus::from)
    }
}

impl<S: AsyncGrpcService, M: MetadataMap> AsyncUnaryChannel for LoopbackChannel<'_, S, M> {
    async fn call_unary_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let req_framed =
                encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
            let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
                .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
            if !req_rest.is_empty() {
                return Err(GrpcStatus::internal(
                    "unexpected trailing bytes after request LPM frame",
                ));
            }
            Ok(req_frame.payload)
        })();
        let req_payload = match req_res {
            Ok(payload) => payload,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_path_async(&mut meta, path, req_payload, &mut resp_writer)
                .await
        };
        self.metadata.request.clear();
        res?;
        let resp_framed = resp_writer.finish().map_err(GrpcStatus::from)?;
        let (resp_frame, resp_rest) = decode_grpc_frame(resp_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated response LPM frame"))?;
        if !resp_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after response LPM frame",
            ));
        }
        Ok(resp_frame.payload)
    }

    async fn call_server_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let req_framed =
                encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
            let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
                .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
            if !req_rest.is_empty() {
                return Err(GrpcStatus::internal(
                    "unexpected trailing bytes after request LPM frame",
                ));
            }
            Ok(req_frame.payload)
        })();
        let req_payload = match req_res {
            Ok(payload) => payload,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_path_async(&mut meta, path, req_payload, &mut resp_writer)
                .await
        };
        self.metadata.request.clear();
        res?;
        resp_writer.finish().map_err(GrpcStatus::from)
    }

    async fn call_client_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let mut req_writer = BackwardWriter::new(self.req_buf);
            encode_reqs(&mut req_writer)?;
            req_writer.finish().map_err(GrpcStatus::from)
        })();
        let req_frames = match req_res {
            Ok(frames) => frames,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_stream_path_async(&mut meta, path, req_frames, &mut resp_writer)
                .await
        };
        self.metadata.request.clear();
        res?;
        let resp_framed = resp_writer.finish().map_err(GrpcStatus::from)?;
        let (resp_frame, resp_rest) = decode_grpc_frame(resp_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated response LPM frame"))?;
        if !resp_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after response LPM frame",
            ));
        }
        Ok(resp_frame.payload)
    }

    async fn call_bidi_streaming_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_reqs: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) -> Result<(), GrpcStatus<'static>> + 'a,
    {
        self.metadata.response_headers.clear();
        self.metadata.response_trailers.clear();
        let req_res = (|| {
            let mut req_writer = BackwardWriter::new(self.req_buf);
            encode_reqs(&mut req_writer)?;
            req_writer.finish().map_err(GrpcStatus::from)
        })();
        let req_frames = match req_res {
            Ok(frames) => frames,
            Err(err) => {
                self.metadata.request.clear();
                return Err(err);
            }
        };

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        let res = {
            let mut meta = self.metadata.metadata();
            self.service
                .dispatch_stream_path_async(&mut meta, path, req_frames, &mut resp_writer)
                .await
        };
        self.metadata.request.clear();
        res?;
        resp_writer.finish().map_err(GrpcStatus::from)
    }
}
