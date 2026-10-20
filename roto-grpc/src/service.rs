use core::future::Future;

use roto_runtime::BackwardWriter;

use crate::framing::{decode_grpc_frame, encode_grpc_message, DEFAULT_MAX_MESSAGE_SIZE};
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

    /// Dispatches a unary RPC call for `method_name` (the part after `"/{SERVICE_NAME}/"`).
    ///
    /// `req_payload` is the raw protobuf message slice (with the 5-byte gRPC LPM header already stripped).
    /// The implementation decodes the request view, invokes the service handler to populate the response
    /// builder on `writer`, and prepends the 5-byte gRPC LPM header `[0, len_be..]` onto `writer`.
    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>>;

    /// Dispatches a unary RPC call using the full HTTP/2 `:path` pseudo-header
    /// (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_path(
        &self,
        path: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
            return Err(GrpcStatus::unimplemented("unimplemented gRPC service or path"));
        };
        self.call_unary(method_name, req_payload, writer)
    }
}

/// Asynchronous `#![no_std]` zero-allocation gRPC service dispatch trait implemented by
/// generated `<Service>Server<T>` wrappers.
pub trait AsyncGrpcService {
    /// Full protobuf service name: `"<package>.<Service>"` (or `"<Service>"` if package is empty).
    const SERVICE_NAME: &'static str;

    /// Asynchronously dispatches a unary RPC call for `method_name` (the part after `"/{SERVICE_NAME}/"`).
    ///
    /// `req_payload` is the raw protobuf message slice (with the 5-byte gRPC LPM header already stripped).
    /// The implementation decodes the request view, awaits the service handler to populate the response
    /// builder on `writer`, and prepends the 5-byte gRPC LPM header `[0, len_be..]` onto `writer`.
    fn call_unary_async<'a, 'b>(
        &'a self,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a;

    /// Asynchronously dispatches a unary RPC call using the full HTTP/2 `:path` pseudo-header
    /// (`"/{SERVICE_NAME}/{Method}"`).
    #[inline]
    fn dispatch_path_async<'a, 'b>(
        &'a self,
        path: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> impl Future<Output = Result<(), GrpcStatus<'static>>> + 'a {
        async move {
            let Some(method_name) = extract_method_name(path, Self::SERVICE_NAME) else {
                return Err(GrpcStatus::unimplemented("unimplemented gRPC service or path"));
            };
            self.call_unary_async(method_name, req_payload, writer).await
        }
    }
}

/// Synchronous `#![no_std]` zero-allocation client channel trait used by generated
/// `<Service>Client<C>` stubs.
pub trait UnaryChannel {
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
}

/// Asynchronous `#![no_std]` zero-allocation client channel trait used by generated
/// `<Service>Client<C>` stubs.
pub trait AsyncUnaryChannel {
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
}

/// Zero-allocation in-memory loopback channel connecting a client stub directly to a
/// [`GrpcService`] or [`AsyncGrpcService`] using caller-provided request and response buffers.
#[derive(Debug)]
pub struct LoopbackChannel<'buf, S> {
    pub service: S,
    pub req_buf: &'buf mut [u8],
    pub resp_buf: &'buf mut [u8],
    pub max_message_size: usize,
}

impl<'buf, S> LoopbackChannel<'buf, S> {
    /// Creates a new [`LoopbackChannel`] with [`DEFAULT_MAX_MESSAGE_SIZE`].
    #[inline]
    pub fn new(service: S, req_buf: &'buf mut [u8], resp_buf: &'buf mut [u8]) -> Self {
        Self {
            service,
            req_buf,
            resp_buf,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
        }
    }

    /// Configures the maximum inbound/outbound message payload size enforced during LPM deframing.
    #[inline]
    pub const fn with_max_message_size(mut self, max_message_size: usize) -> Self {
        self.max_message_size = max_message_size;
        self
    }
}

impl<S: GrpcService> UnaryChannel for LoopbackChannel<'_, S> {
    fn call_unary<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let req_framed =
            encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
        let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
        if !req_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after request LPM frame",
            ));
        }

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        self.service
            .dispatch_path(path, req_frame.payload, &mut resp_writer)?;
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
}

impl<S: AsyncGrpcService> AsyncUnaryChannel for LoopbackChannel<'_, S> {
    async fn call_unary_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        let req_framed =
            encode_grpc_message(self.req_buf, false, encode_req).map_err(GrpcStatus::from)?;
        let (req_frame, req_rest) = decode_grpc_frame(req_framed, self.max_message_size)?
            .ok_or_else(|| GrpcStatus::internal("truncated request LPM frame"))?;
        if !req_rest.is_empty() {
            return Err(GrpcStatus::internal(
                "unexpected trailing bytes after request LPM frame",
            ));
        }

        let mut resp_writer = BackwardWriter::new(self.resp_buf);
        self.service
            .dispatch_path_async(path, req_frame.payload, &mut resp_writer)
            .await?;
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
}
