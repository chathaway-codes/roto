//! Asynchronous `#![no_std]` (embedded / RP2040) and optional `tokio` transport drivers
//! for [`GrpcClient`], [`GrpcServer`], and [`AsyncGrpcServer`].

use core::cell::RefCell;
use core::fmt;
use core::future::{poll_fn, Future};
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use roto_runtime::BackwardWriter;

use crate::client::{reply_result, GrpcClient, Reply};
use crate::h2::{H2Error, H2ErrorCode, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP};
use crate::server::{AsyncGrpcServer, AsyncRouter, GrpcServer};
use crate::service::AsyncUnaryChannel;
use crate::status::{GrpcCode, GrpcStatus};

/// Default per-driver I/O scratch buffer size in bytes (`4,096`).
pub const DEFAULT_IO_BUF_LEN: usize = 4096;

/// Error returned by the asynchronous client and server transport drivers.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportError<E> {
    /// Underlying byte-stream I/O error.
    Io(E),
    /// HTTP/2 protocol or framing error.
    H2(H2Error),
    /// Peer closed the byte stream while an HTTP/2 frame or RPC call was still in flight.
    UnexpectedEof,
}

impl<E: fmt::Debug> fmt::Display for TransportError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "transport I/O error: {err:?}"),
            Self::H2(err) => write!(f, "{err}"),
            Self::UnexpectedEof => write!(f, "unexpected EOF on HTTP/2 connection"),
        }
    }
}

impl<E: fmt::Debug> core::error::Error for TransportError<E> {}

impl<E> From<H2Error> for TransportError<E> {
    #[inline]
    fn from(err: H2Error) -> Self {
        Self::H2(err)
    }
}

/// Minimal `#![no_std]` zero-allocation asynchronous byte-stream abstraction.
///
/// Mirrors the `embedded-io-async` read/write model so bare-metal UART, USB-CDC,
/// RP2040 PIO/SPI, or `embassy-net` sockets can implement it directly without
/// heap allocation or `std`. When the `tokio` feature is enabled, [`TokioIo`]
/// adapts any `tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin` stream.
pub trait AsyncIo {
    /// Transport-specific I/O error type.
    type Error: fmt::Debug;

    /// Reads up to `buf.len()` bytes from the stream, returning `Ok(0)` on clean EOF.
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + 'a;

    /// Writes the entire `buf` slice to the stream.
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a;

    /// Flushes any buffered outbound bytes to the underlying transport.
    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + '_ {
        async { Ok(()) }
    }
}

impl<T: AsyncIo + ?Sized> AsyncIo for &mut T {
    type Error = T::Error;

    #[inline]
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + 'a {
        (**self).read(buf)
    }

    #[inline]
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        (**self).write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + '_ {
        (**self).flush()
    }
}

// ============================================================================
// Client Transport Driver (`AsyncClientTransport`)
// ============================================================================

/// `#![no_std]` zero-allocation asynchronous gRPC client transport driving a [`GrpcClient`]
/// over any [`AsyncIo`] byte stream.
///
/// Implements [`AsyncUnaryChannel`] so generated `<Service>Client<C>` stubs can invoke
/// `<method>_async` directly, and also exposes [`Self::start_call`] + [`Self::drive_until_reply`]
/// for concurrent multiplexed unary RPCs over a single connection.
#[must_use]
pub struct AsyncClientTransport<
    'b,
    T,
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
    const IO_BUF_LEN: usize = DEFAULT_IO_BUF_LEN,
> {
    client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
    io: T,
    call_buf: &'b mut [u8],
    io_buf: [u8; IO_BUF_LEN],
}

impl<
        T: fmt::Debug,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
        const IO_BUF_LEN: usize,
    > fmt::Debug for AsyncClientTransport<'_, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncClientTransport")
            .field("client", &self.client)
            .field("io", &self.io)
            .field("call_buf_cap", &self.call_buf.len())
            .field("io_buf_cap", &IO_BUF_LEN)
            .finish()
    }
}

impl<'b, T, const MAX_STREAMS: usize, const REASM_CAP: usize>
    AsyncClientTransport<'b, T, MAX_STREAMS, REASM_CAP, DEFAULT_IO_BUF_LEN>
{
    /// Wraps `io` and `client`, using `call_buf` for [`AsyncUnaryChannel`] calls
    /// and the default internal scratch buffer length ([`DEFAULT_IO_BUF_LEN`]).
    pub const fn new(
        io: T,
        client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
        call_buf: &'b mut [u8],
    ) -> Self {
        Self::new_with_io_buf(io, client, call_buf)
    }
}

impl<'b, T, const MAX_STREAMS: usize, const REASM_CAP: usize, const IO_BUF_LEN: usize>
    AsyncClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
{
    /// Wraps `io` and `client`, using `call_buf` for [`AsyncUnaryChannel`] calls
    /// and a caller-chosen internal scratch buffer length `IO_BUF_LEN`.
    pub const fn new_with_io_buf(
        io: T,
        client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
        call_buf: &'b mut [u8],
    ) -> Self {
        Self {
            client,
            io,
            call_buf,
            io_buf: [0u8; IO_BUF_LEN],
        }
    }

    /// Returns a shared reference to the underlying [`GrpcClient`].
    #[must_use]
    #[inline]
    pub const fn client(&self) -> &GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
        &self.client
    }

    /// Returns a mutable reference to the underlying [`GrpcClient`].
    #[inline]
    pub fn client_mut(&mut self) -> &mut GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
        &mut self.client
    }

    /// Returns a shared reference to the underlying [`AsyncIo`] stream.
    #[must_use]
    #[inline]
    pub const fn io(&self) -> &T {
        &self.io
    }

    /// Returns a mutable reference to the underlying [`AsyncIo`] stream.
    #[inline]
    pub fn io_mut(&mut self) -> &mut T {
        &mut self.io
    }

    /// Consumes this transport driver and returns the underlying [`AsyncIo`] stream.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> T {
        self.io
    }

    /// Starts a unary call on the underlying [`GrpcClient`] using caller-provided `buf`.
    ///
    /// Use [`Self::drive_until_reply`] to drive the connection until `stream_id` finishes,
    /// or [`Self::take_reply`] if an earlier `drive_until_reply` already received its response.
    pub fn start_call<F>(
        &mut self,
        path: &str,
        buf: &'b mut [u8],
        encode_req: F,
    ) -> Result<u32, Reply<'b>>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        self.client.start_call(path, buf, encode_req)
    }

    /// Removes and returns the [`Reply`] of call `stream_id` if it has already finished.
    #[must_use]
    #[inline]
    pub fn take_reply(&mut self, stream_id: u32) -> Option<Reply<'b>> {
        self.client.take_reply(stream_id)
    }

    /// Takes the 8-byte opaque payload of the most recently received `PING` ACK frame, if any.
    #[must_use]
    #[inline]
    pub fn take_ping_ack(&mut self) -> Option<[u8; 8]> {
        self.client.take_ping_ack()
    }

    /// Returns `(last_stream_id, error_code)` if a `GOAWAY` frame has been received from the server.
    #[must_use]
    #[inline]
    pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
        self.client.goaway_received()
    }

    /// Returns the byte capacity of the internal [`AsyncUnaryChannel`] call buffer.
    #[must_use]
    #[inline]
    pub const fn call_buf_cap(&self) -> usize {
        self.call_buf.len()
    }
}

async fn flush_with<T: AsyncIo>(
    io: &mut T,
    buf: &mut [u8],
    mut poll_tx: impl FnMut(&mut [u8]) -> usize,
) -> Result<(), TransportError<T::Error>> {
    let mut wrote_any = false;
    loop {
        let n = poll_tx(buf);
        if n == 0 {
            break;
        }
        io.write_all(&buf[..n]).await.map_err(TransportError::Io)?;
        wrote_any = true;
    }
    if wrote_any {
        io.flush().await.map_err(TransportError::Io)?;
    }
    Ok(())
}

impl<'b, T: AsyncIo, const MAX_STREAMS: usize, const REASM_CAP: usize, const IO_BUF_LEN: usize>
    AsyncClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
{
    /// Drains all queued outbound bytes from [`GrpcClient::poll_transmit`] and flushes `io`.
    pub async fn flush_outbound(&mut self) -> Result<(), TransportError<T::Error>> {
        flush_with(&mut self.io, &mut self.io_buf, |buf| {
            self.client.poll_transmit(buf)
        })
        .await
    }

    /// Flushes any pending outbound frames, reads one chunk of inbound bytes from `io`,
    /// feeds it to [`GrpcClient::recv`], and flushes any resulting control/data frames.
    pub async fn step(&mut self) -> Result<usize, TransportError<T::Error>> {
        self.flush_outbound().await?;
        let n = self
            .io
            .read(&mut self.io_buf)
            .await
            .map_err(TransportError::Io)?;
        if n == 0 {
            return Ok(0);
        }
        if let Err(err) = self.client.recv(&self.io_buf[..n]) {
            let _ = self.flush_outbound().await;
            return Err(TransportError::H2(err));
        }
        self.flush_outbound().await?;
        Ok(n)
    }

    /// Drives outbound and inbound I/O until call `stream_id` finishes and returns its [`Reply`].
    ///
    /// Other multiplexed calls in flight on the same connection also advance as frames arrive;
    /// if a sibling call finishes during this loop, its [`Reply`] remains queued for retrieval
    /// via [`Self::take_reply`] or a subsequent `drive_until_reply`.
    ///
    /// If the connection encounters `UnexpectedEof` or a fatal [`H2Error`], all in-flight calls
    /// are marked failed in [`GrpcClient`] (so their buffers remain recoverable via [`Self::take_reply`])
    /// and `Err(TransportError::UnexpectedEof)` or `Err(TransportError::H2(err))` is returned.
    pub async fn drive_until_reply(
        &mut self,
        stream_id: u32,
    ) -> Result<Reply<'b>, TransportError<T::Error>> {
        loop {
            self.flush_outbound().await?;
            if let Some(reply) = self.client.take_reply(stream_id) {
                return Ok(reply);
            }
            let n = self
                .io
                .read(&mut self.io_buf)
                .await
                .map_err(TransportError::Io)?;
            if n == 0 {
                self.client
                    .fail_all_in_flight(GrpcCode::Unavailable, b"unexpected EOF from server");
                return Err(TransportError::UnexpectedEof);
            }
            if let Err(err) = self.client.recv(&self.io_buf[..n]) {
                let _ = self.flush_outbound().await;
                return Err(TransportError::H2(err));
            }
        }
    }

    /// Queues and flushes an outbound HTTP/2 `PING` frame (`ack = false`).
    pub async fn send_ping(&mut self, data: [u8; 8]) -> Result<(), TransportError<T::Error>> {
        self.client.send_ping(data).map_err(TransportError::H2)?;
        self.flush_outbound().await
    }

    /// Drives the connection until a `PING` ACK frame is received from the peer.
    pub async fn drive_until_ping_ack(&mut self) -> Result<[u8; 8], TransportError<T::Error>> {
        loop {
            if let Some(ack) = self.client.take_ping_ack() {
                self.flush_outbound().await?;
                return Ok(ack);
            }
            let n = self.step().await?;
            if let Some(ack) = self.client.take_ping_ack() {
                return Ok(ack);
            }
            if n == 0 {
                return Err(TransportError::UnexpectedEof);
            }
        }
    }

    /// Queues and flushes an outbound HTTP/2 `GOAWAY` frame on stream `0`.
    pub async fn send_goaway(
        &mut self,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), TransportError<T::Error>> {
        self.client
            .send_goaway(error_code, debug_data)
            .map_err(TransportError::H2)?;
        self.flush_outbound().await
    }
}

impl<'b, T: AsyncIo, const MAX_STREAMS: usize, const REASM_CAP: usize, const IO_BUF_LEN: usize>
    AsyncUnaryChannel for AsyncClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
{
    async fn call_unary_async<'a, F>(
        &'a mut self,
        path: &'static str,
        encode_req: F,
    ) -> Result<&'a [u8], GrpcStatus<'a>>
    where
        Self: 'a,
        F: FnOnce(&mut BackwardWriter<'_>) + 'a,
    {
        let buf = core::mem::take(&mut self.call_buf);
        let reply = match self.client.start_call(path, buf, encode_req) {
            Ok(stream_id) => match self.drive_until_reply(stream_id).await {
                Ok(reply) => reply,
                Err(err) => {
                    let status = match err {
                        TransportError::UnexpectedEof => {
                            GrpcStatus::unavailable("unexpected EOF from server")
                        }
                        TransportError::H2(h2_err) => GrpcStatus::from(h2_err),
                        TransportError::Io(_) => GrpcStatus::unavailable("transport I/O error"),
                    };
                    self.client
                        .fail_call(stream_id, status.code, status.message.as_bytes());
                    if let Some(reply) = self.client.take_reply(stream_id) {
                        self.call_buf = reply.into_buf();
                    }
                    return Err(status);
                }
            },
            Err(reply) => reply,
        };
        let code = reply.code;
        let len = reply.len;
        self.call_buf = reply.into_buf();
        reply_result(code, &self.call_buf[..len])
    }
}

// ============================================================================
// Server Transport Drivers (`serve_connection` & `serve_async_connection`)
// ============================================================================

/// Drains all queued outbound frames from `server` into `io` and flushes the stream.
pub async fn flush_server_outbound<
    T: AsyncIo,
    const MAX_STREAMS: usize,
    const REASM_CAP: usize,
>(
    server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
    buf: &mut [u8],
) -> Result<(), TransportError<T::Error>> {
    flush_with(io, buf, |b| server.poll_transmit(b)).await
}

/// Flushes pending outbound frames, reads one chunk from `io`, feeds [`GrpcServer::recv`],
/// and flushes any resulting response or control frames (including `GOAWAY` on protocol error).
pub async fn step_server<T: AsyncIo, const MAX_STREAMS: usize, const REASM_CAP: usize>(
    server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
    buf: &mut [u8],
) -> Result<usize, TransportError<T::Error>> {
    flush_server_outbound(server, io, buf).await?;
    let n = io.read(buf).await.map_err(TransportError::Io)?;
    if n == 0 {
        return Ok(0);
    }
    if let Err(err) = server.recv(&buf[..n]) {
        let _ = flush_server_outbound(server, io, buf).await;
        return Err(TransportError::H2(err));
    }
    flush_server_outbound(server, io, buf).await?;
    Ok(n)
}

/// Drives a synchronous [`GrpcServer`] over `io` until clean client disconnect (`EOF` with no
/// pending calls or partial frames) or a fatal transport/HTTP/2 error.
///
/// If `server` has already served (or started serving) a previous connection and has not yet
/// been explicitly [`GrpcServer::reset`], this function resets it automatically before sending
/// the initial `SETTINGS` frame.
pub async fn serve_connection<T: AsyncIo, const MAX_STREAMS: usize, const REASM_CAP: usize>(
    server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
) -> Result<(), TransportError<T::Error>> {
    server.reset_if_active();
    let mut buf = [0u8; DEFAULT_IO_BUF_LEN];
    loop {
        let n = step_server(server, io, &mut buf).await?;
        if n == 0 {
            if server.has_pending_work() {
                return Err(TransportError::UnexpectedEof);
            }
            return Ok(());
        }
    }
}

/// Drains all queued outbound frames from an [`AsyncGrpcServer`] into `io` and flushes the stream.
pub async fn flush_async_server_outbound<
    T: AsyncIo,
    R: AsyncRouter,
    const MAX_STREAMS: usize,
    const REASM_CAP: usize,
>(
    server: &mut AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
    buf: &mut [u8],
) -> Result<(), TransportError<T::Error>> {
    flush_with(io, buf, |b| server.poll_transmit(b)).await
}

/// Flushes pending outbound frames, reads one chunk from `io`, awaits [`AsyncGrpcServer::recv`],
/// and flushes any resulting response or control frames (including `GOAWAY` on protocol error).
pub async fn step_async_server<
    T: AsyncIo,
    R: AsyncRouter,
    const MAX_STREAMS: usize,
    const REASM_CAP: usize,
>(
    server: &mut AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
    buf: &mut [u8],
) -> Result<usize, TransportError<T::Error>> {
    flush_async_server_outbound(server, io, buf).await?;
    let n = io.read(buf).await.map_err(TransportError::Io)?;
    if n == 0 {
        return Ok(0);
    }
    let recv_res = server.recv(&buf[..n]).await;
    if let Err(err) = recv_res {
        let _ = flush_async_server_outbound(server, io, buf).await;
        return Err(TransportError::H2(err));
    }
    flush_async_server_outbound(server, io, buf).await?;
    Ok(n)
}

/// Drives an [`AsyncGrpcServer`] over `io` until clean client disconnect (`EOF` with no
/// pending calls or partial frames) or a fatal transport/HTTP/2 error.
///
/// If `server` has already served (or started serving) a previous connection and has not yet
/// been explicitly [`AsyncGrpcServer::reset`], this function resets it automatically before
/// sending the initial `SETTINGS` frame.
pub async fn serve_async_connection<
    T: AsyncIo,
    R: AsyncRouter,
    const MAX_STREAMS: usize,
    const REASM_CAP: usize,
>(
    server: &mut AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>,
    io: &mut T,
) -> Result<(), TransportError<T::Error>> {
    server.reset_if_active();
    let mut buf = [0u8; DEFAULT_IO_BUF_LEN];
    loop {
        let n = step_async_server(server, io, &mut buf).await?;
        if n == 0 {
            if server.has_pending_work() {
                return Err(TransportError::UnexpectedEof);
            }
            return Ok(());
        }
    }
}

// ============================================================================
// `#![no_std]` Zero-Allocation In-Memory Duplex Transport & Cooperative Runner
// ============================================================================

/// Error returned by [`MemoryEndpoint`].
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryIoError {
    /// Attempted to write to a direction that has already been closed.
    BrokenPipe,
}

impl fmt::Display for MemoryIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrokenPipe => write!(f, "broken pipe on in-memory transport"),
        }
    }
}

impl core::error::Error for MemoryIoError {}

#[derive(Debug)]
struct RingBuf<const CAP: usize> {
    buf: [u8; CAP],
    head: usize,
    len: usize,
    closed: bool,
    reader_dropped: bool,
}

impl<const CAP: usize> RingBuf<CAP> {
    const fn new() -> Self {
        Self {
            buf: [0u8; CAP],
            head: 0,
            len: 0,
            closed: false,
            reader_dropped: false,
        }
    }

    fn dequeue(&mut self, dst: &mut [u8]) -> usize {
        let n = dst.len().min(self.len);
        for item in dst.iter_mut().take(n) {
            *item = self.buf[self.head];
            self.head = (self.head + 1) % CAP;
        }
        self.len -= n;
        n
    }

    fn enqueue(&mut self, src: &[u8]) -> usize {
        let free = CAP - self.len;
        let n = src.len().min(free);
        let mut tail = (self.head + self.len) % CAP;
        for &byte in src.iter().take(n) {
            self.buf[tail] = byte;
            tail = (tail + 1) % CAP;
        }
        self.len += n;
        n
    }
}

/// `#![no_std]` zero-allocation full-duplex in-memory byte stream for bare-metal
/// and deterministic multi-poll testing.
#[must_use]
#[derive(Debug)]
pub struct MemoryDuplex<const CAP: usize = DEFAULT_IO_BUF_LEN> {
    a_to_b: RefCell<RingBuf<CAP>>,
    b_to_a: RefCell<RingBuf<CAP>>,
}

impl<const CAP: usize> Default for MemoryDuplex<CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> MemoryDuplex<CAP> {
    /// Creates a new full-duplex ring buffer pair with `CAP` bytes in each direction.
    #[inline]
    pub const fn new() -> Self {
        Self {
            a_to_b: RefCell::new(RingBuf::new()),
            b_to_a: RefCell::new(RingBuf::new()),
        }
    }

    /// Splits this duplex pipe into its two endpoints `(client_io, server_io)`.
    ///
    /// Dropping or calling [`MemoryEndpoint::close`] on either endpoint signals clean EOF
    /// (`Ok(0)`) to the peer's `read` once all queued bytes in that direction are drained.
    #[inline]
    pub const fn split(&self) -> (MemoryEndpoint<'_, CAP>, MemoryEndpoint<'_, CAP>) {
        (
            MemoryEndpoint::new(&self.b_to_a, &self.a_to_b),
            MemoryEndpoint::new(&self.a_to_b, &self.b_to_a),
        )
    }
}

/// One half of a [`MemoryDuplex`] full-duplex byte pipe implementing [`AsyncIo`].
#[must_use]
#[derive(Debug)]
pub struct MemoryEndpoint<'a, const CAP: usize = DEFAULT_IO_BUF_LEN> {
    rx: &'a RefCell<RingBuf<CAP>>,
    tx: &'a RefCell<RingBuf<CAP>>,
    max_read_chunk: usize,
    max_write_chunk: usize,
    yield_pending: bool,
    pending_toggle: bool,
}

impl<'a, const CAP: usize> MemoryEndpoint<'a, CAP> {
    const fn new(rx: &'a RefCell<RingBuf<CAP>>, tx: &'a RefCell<RingBuf<CAP>>) -> Self {
        Self {
            rx,
            tx,
            max_read_chunk: usize::MAX,
            max_write_chunk: usize::MAX,
            yield_pending: false,
            pending_toggle: false,
        }
    }

    /// Limits both `read` and `write_all` steps to at most `chunk` bytes per poll.
    #[inline]
    pub const fn with_max_chunk(mut self, chunk: usize) -> Self {
        let c = if chunk == 0 { 1 } else { chunk };
        self.max_read_chunk = c;
        self.max_write_chunk = c;
        self
    }

    /// Forces each I/O operation to yield `Poll::Pending` before transferring bytes,
    /// exercising multi-poll suspension points in `#![no_std]` futures.
    #[inline]
    pub const fn with_pending_yields(mut self, enabled: bool) -> Self {
        self.yield_pending = enabled;
        self
    }

    /// Closes the outbound direction of this endpoint, signaling clean EOF to the peer once
    /// all queued outbound bytes have been read.
    pub fn close(&self) {
        self.tx.borrow_mut().closed = true;
    }
}

impl<const CAP: usize> Drop for MemoryEndpoint<'_, CAP> {
    fn drop(&mut self) {
        self.tx.borrow_mut().closed = true;
        self.rx.borrow_mut().reader_dropped = true;
    }
}

impl<const CAP: usize> AsyncIo for MemoryEndpoint<'_, CAP> {
    type Error = MemoryIoError;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        poll_fn(|_cx| {
            if self.yield_pending && !self.pending_toggle {
                self.pending_toggle = true;
                return Poll::Pending;
            }
            self.pending_toggle = false;
            let mut rx = self.rx.borrow_mut();
            if rx.len > 0 {
                let limit = buf.len().min(self.max_read_chunk);
                let n = rx.dequeue(&mut buf[..limit]);
                Poll::Ready(Ok(n))
            } else if rx.closed {
                Poll::Ready(Ok(0))
            } else {
                Poll::Pending
            }
        })
        .await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let mut written = 0usize;
        poll_fn(|_cx| {
            if written == buf.len() {
                return Poll::Ready(Ok(()));
            }
            if self.yield_pending && !self.pending_toggle {
                self.pending_toggle = true;
                return Poll::Pending;
            }
            self.pending_toggle = false;
            let mut tx = self.tx.borrow_mut();
            if tx.closed {
                return Poll::Ready(Err(MemoryIoError::BrokenPipe));
            }
            let rem = &buf[written..];
            let limit = rem.len().min(self.max_write_chunk);
            let n = tx.enqueue(&rem[..limit]);
            written += n;
            if written == buf.len() {
                Poll::Ready(Ok(()))
            } else if n == 0 && tx.reader_dropped {
                Poll::Ready(Err(MemoryIoError::BrokenPipe))
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

/// Polls two futures (`fut_a` and `fut_b`) cooperatively to completion in `#![no_std]` with
/// zero heap allocation, returning `(fut_a_output, fut_b_output)`.
pub fn block_on_pair<A: Future, B: Future>(fut_a: A, fut_b: B) -> (A::Output, B::Output) {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut pin_a = pin!(fut_a);
    let mut pin_b = pin!(fut_b);
    let mut out_a: Option<A::Output> = None;
    let mut out_b: Option<B::Output> = None;
    loop {
        if out_a.is_none() {
            if let Poll::Ready(val) = pin_a.as_mut().poll(&mut cx) {
                out_a = Some(val);
            }
        }
        if out_b.is_none() {
            if let Poll::Ready(val) = pin_b.as_mut().poll(&mut cx) {
                out_b = Some(val);
            }
        }
        if out_a.is_some() && out_b.is_some() {
            // Both `out_a` and `out_b` are guaranteed to be `Some`.
            if let (Some(a), Some(b)) = (out_a.take(), out_b.take()) {
                return (a, b);
            }
        }
    }
}

// ============================================================================
// `#![no_std]` `embedded-io-async` / `embassy-net` Adapter Macro
// ============================================================================

/// Generates a `#![no_std]` zero-allocation [`AsyncIo`] newtype wrapper around an
/// `embassy_net::tcp::TcpSocket` or any stream exposing `async fn read`, `write` / `write_all`,
/// and `flush`.
///
/// # Examples
///
/// Wrapping an `embassy_net::tcp::TcpSocket<'a>` (or any socket with inherent
/// `async fn read(&mut self, &mut [u8]) -> Result<usize, E>`,
/// `async fn write(&mut self, &[u8]) -> Result<usize, E>`, and
/// `async fn flush(&mut self) -> Result<(), E>`):
///
/// ```ignore
/// roto_grpc::impl_async_io!(pub struct EmbassyTcpIo<'a>(embassy_net::tcp::TcpSocket<'a>) -> embassy_net::tcp::Error);
/// ```
///
/// Wrapping any generic `embedded_io_async::Read + embedded_io_async::Write` stream:
///
/// ```ignore
/// roto_grpc::impl_async_io!(embedded_io_async: pub struct EmbeddedIo<T>(T));
/// ```
#[macro_export]
macro_rules! impl_async_io {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident $(<$($lt:lifetime),+ $(,)?>)? ($inner:ty) -> $err:ty
    ) => {
        $(#[$meta])*
        #[must_use]
        $vis struct $name $(<$($lt),+>)? (pub $inner);

        impl $(<$($lt),+>)? $name $(<$($lt),+>)? {
            /// Wraps the inner socket/stream in this [`AsyncIo`]($crate::AsyncIo) adapter.
            #[inline]
            pub const fn new(inner: $inner) -> Self {
                Self(inner)
            }

            /// Returns a shared reference to the underlying socket/stream.
            #[must_use]
            #[inline]
            pub const fn inner(&self) -> &$inner {
                &self.0
            }

            /// Returns a mutable reference to the underlying socket/stream.
            #[inline]
            pub fn inner_mut(&mut self) -> &mut $inner {
                &mut self.0
            }

            /// Consumes the adapter and returns the underlying socket/stream.
            #[must_use]
            #[inline]
            pub fn into_inner(self) -> $inner {
                self.0
            }
        }

        impl $(<$($lt),+>)? $crate::AsyncIo for $name $(<$($lt),+>)? {
            type Error = $err;

            #[inline]
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                self.0.read(buf).await
            }

            #[inline]
            async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
                let mut written = 0usize;
                while written < buf.len() {
                    let n = self.0.write(&buf[written..]).await?;
                    written += n;
                }
                Ok(())
            }

            #[inline]
            async fn flush(&mut self) -> Result<(), Self::Error> {
                self.0.flush().await
            }
        }
    };
    (
        embedded_io_async:
        $(#[$meta:meta])*
        $vis:vis struct $name:ident <$t:ident> ($inner:ty)
    ) => {
        $(#[$meta])*
        #[must_use]
        $vis struct $name<$t>(pub $inner);

        impl<$t> $name<$t> {
            /// Wraps an `embedded_io_async` stream in this [`AsyncIo`]($crate::AsyncIo) adapter.
            #[inline]
            pub const fn new(inner: $inner) -> Self {
                Self(inner)
            }

            /// Returns a shared reference to the underlying stream.
            #[must_use]
            #[inline]
            pub const fn inner(&self) -> &$inner {
                &self.0
            }

            /// Returns a mutable reference to the underlying stream.
            #[inline]
            pub fn inner_mut(&mut self) -> &mut $inner {
                &mut self.0
            }

            /// Consumes the adapter and returns the underlying stream.
            #[must_use]
            #[inline]
            pub fn into_inner(self) -> $inner {
                self.0
            }
        }

        impl<$t: embedded_io_async::Read + embedded_io_async::Write> $crate::AsyncIo for $name<$t> {
            type Error = <$t as embedded_io_async::ErrorType>::Error;

            #[inline]
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                embedded_io_async::Read::read(&mut self.0, buf).await
            }

            #[inline]
            async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
                embedded_io_async::Write::write_all(&mut self.0, buf).await
            }

            #[inline]
            async fn flush(&mut self) -> Result<(), Self::Error> {
                embedded_io_async::Write::flush(&mut self.0).await
            }
        }
    };
}

pub use crate::impl_async_io;

// ============================================================================
// Optional `std` Blocking TCP / Read+Write Transport Drivers (`feature = "std"`)
// ============================================================================

#[cfg(feature = "std")]
pub use std_support::{
    accept_and_serve_std, connect_std_tcp, flush_std_server_outbound, serve_std_io,
    step_std_server, StdClientTransport, StdIo, StdTcpClientTransport,
};

#[cfg(feature = "std")]
mod std_support {
    use core::fmt;
    use std::io::{self, Read, Write};
    use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

    use roto_runtime::BackwardWriter;

    use super::{AsyncIo, TransportError, DEFAULT_IO_BUF_LEN};
    use crate::client::{reply_result, GrpcClient, Reply};
    use crate::h2::{H2ErrorCode, DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP};
    use crate::server::GrpcServer;
    use crate::service::{AsyncUnaryChannel, UnaryChannel};
    use crate::status::{GrpcCode, GrpcStatus};

    #[inline]
    fn read_interrupted<R: Read>(io: &mut R, buf: &mut [u8]) -> Result<usize, io::Error> {
        loop {
            match io.read(buf) {
                Ok(n) => return Ok(n),
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            }
        }
    }

    fn flush_std_with<W: Write>(
        io: &mut W,
        buf: &mut [u8],
        mut poll_tx: impl FnMut(&mut [u8]) -> usize,
    ) -> Result<(), TransportError<io::Error>> {
        let mut wrote_any = false;
        loop {
            let n = poll_tx(buf);
            if n == 0 {
                break;
            }
            io.write_all(&buf[..n]).map_err(TransportError::Io)?;
            wrote_any = true;
        }
        if wrote_any {
            io.flush().map_err(TransportError::Io)?;
        }
        Ok(())
    }

    /// Adapter implementing [`AsyncIo`] and blocking I/O helpers for any `std::io::Read + std::io::Write`
    /// stream (such as [`std::net::TcpStream`]).
    #[must_use]
    #[derive(Debug)]
    pub struct StdIo<T>(pub T);

    impl<T> StdIo<T> {
        /// Wraps a `std::io::Read + std::io::Write` stream in [`StdIo`].
        #[inline]
        pub const fn new(inner: T) -> Self {
            Self(inner)
        }

        /// Returns a shared reference to the inner stream.
        #[must_use]
        #[inline]
        pub const fn inner(&self) -> &T {
            &self.0
        }

        /// Returns a mutable reference to the inner stream.
        #[inline]
        pub fn inner_mut(&mut self) -> &mut T {
            &mut self.0
        }

        /// Unwraps this adapter, returning the inner stream.
        #[must_use]
        #[inline]
        pub fn into_inner(self) -> T {
            self.0
        }
    }

    impl StdIo<TcpStream> {
        /// Shuts down the write half of the underlying [`TcpStream`], sending EOF (`FIN`) to the peer.
        pub fn shutdown(&self) -> Result<(), io::Error> {
            self.0.shutdown(Shutdown::Write)
        }
    }

    impl<T: Read + Write> AsyncIo for StdIo<T> {
        type Error = io::Error;

        #[inline]
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            read_interrupted(&mut self.0, buf)
        }

        #[inline]
        async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(buf)
        }

        #[inline]
        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.0.flush()
        }
    }

    /// Synchronous blocking gRPC client transport driving a [`GrpcClient`] over any
    /// `std::io::Read + std::io::Write` stream (such as [`std::net::TcpStream`]).
    ///
    /// Implements [`UnaryChannel`] (and [`AsyncUnaryChannel`]) so generated `<Service>Client<C>`
    /// stubs can invoke synchronous `<method>` calls directly, and also exposes
    /// [`Self::start_call`] + [`Self::drive_until_reply`] for concurrent multiplexed unary RPCs
    /// over a single TCP connection.
    #[must_use]
    pub struct StdClientTransport<
        'b,
        T,
        const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
        const REASM_CAP: usize = DEFAULT_REASM_CAP,
        const IO_BUF_LEN: usize = DEFAULT_IO_BUF_LEN,
    > {
        client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
        io: T,
        call_buf: &'b mut [u8],
        io_buf: [u8; IO_BUF_LEN],
    }

    impl<
            T: fmt::Debug,
            const MAX_STREAMS: usize,
            const REASM_CAP: usize,
            const IO_BUF_LEN: usize,
        > fmt::Debug for StdClientTransport<'_, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("StdClientTransport")
                .field("client", &self.client)
                .field("io", &self.io)
                .field("call_buf_cap", &self.call_buf.len())
                .field("io_buf_cap", &IO_BUF_LEN)
                .finish()
        }
    }

    impl<'b, T, const MAX_STREAMS: usize, const REASM_CAP: usize>
        StdClientTransport<'b, T, MAX_STREAMS, REASM_CAP, DEFAULT_IO_BUF_LEN>
    {
        /// Wraps `io` and `client`, using `call_buf` for [`UnaryChannel`] calls
        /// and the default internal scratch buffer length ([`DEFAULT_IO_BUF_LEN`]).
        pub const fn new(
            io: T,
            client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
            call_buf: &'b mut [u8],
        ) -> Self {
            Self::new_with_io_buf(io, client, call_buf)
        }
    }

    impl<'b, T, const MAX_STREAMS: usize, const REASM_CAP: usize, const IO_BUF_LEN: usize>
        StdClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
    {
        /// Wraps `io` and `client`, using `call_buf` for [`UnaryChannel`] calls
        /// and a caller-chosen internal scratch buffer length `IO_BUF_LEN`.
        pub const fn new_with_io_buf(
            io: T,
            client: GrpcClient<'b, MAX_STREAMS, REASM_CAP>,
            call_buf: &'b mut [u8],
        ) -> Self {
            Self {
                client,
                io,
                call_buf,
                io_buf: [0u8; IO_BUF_LEN],
            }
        }

        /// Returns a shared reference to the underlying [`GrpcClient`].
        #[must_use]
        #[inline]
        pub const fn client(&self) -> &GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
            &self.client
        }

        /// Returns a mutable reference to the underlying [`GrpcClient`].
        #[inline]
        pub fn client_mut(&mut self) -> &mut GrpcClient<'b, MAX_STREAMS, REASM_CAP> {
            &mut self.client
        }

        /// Returns a shared reference to the underlying I/O stream.
        #[must_use]
        #[inline]
        pub const fn io(&self) -> &T {
            &self.io
        }

        /// Returns a mutable reference to the underlying I/O stream.
        #[inline]
        pub fn io_mut(&mut self) -> &mut T {
            &mut self.io
        }

        /// Consumes this transport driver and returns the underlying I/O stream.
        #[must_use]
        #[inline]
        pub fn into_inner(self) -> T {
            self.io
        }

        /// Starts a unary call on the underlying [`GrpcClient`] using caller-provided `buf`.
        pub fn start_call<F>(
            &mut self,
            path: &str,
            buf: &'b mut [u8],
            encode_req: F,
        ) -> Result<u32, Reply<'b>>
        where
            F: FnOnce(&mut BackwardWriter<'_>),
        {
            self.client.start_call(path, buf, encode_req)
        }

        /// Removes and returns the [`Reply`] of call `stream_id` if it has already finished.
        #[must_use]
        #[inline]
        pub fn take_reply(&mut self, stream_id: u32) -> Option<Reply<'b>> {
            self.client.take_reply(stream_id)
        }

        /// Takes the 8-byte opaque payload of the most recently received `PING` ACK frame, if any.
        #[must_use]
        #[inline]
        pub fn take_ping_ack(&mut self) -> Option<[u8; 8]> {
            self.client.take_ping_ack()
        }

        /// Returns `(last_stream_id, error_code)` if a `GOAWAY` frame has been received from the server.
        #[must_use]
        #[inline]
        pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
            self.client.goaway_received()
        }

        /// Returns the byte capacity of the internal [`UnaryChannel`] call buffer.
        #[must_use]
        #[inline]
        pub const fn call_buf_cap(&self) -> usize {
            self.call_buf.len()
        }
    }

    impl<
            'b,
            T: Read + Write,
            const MAX_STREAMS: usize,
            const REASM_CAP: usize,
            const IO_BUF_LEN: usize,
        > StdClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
    {
        /// Drains all queued outbound bytes from [`GrpcClient::poll_transmit`] and flushes `io`.
        pub fn flush_outbound(&mut self) -> Result<(), TransportError<io::Error>> {
            flush_std_with(&mut self.io, &mut self.io_buf, |buf| {
                self.client.poll_transmit(buf)
            })
        }

        /// Flushes any pending outbound frames, reads one chunk of inbound bytes from `io`,
        /// feeds it to [`GrpcClient::recv`], and flushes any resulting control/data frames.
        pub fn step(&mut self) -> Result<usize, TransportError<io::Error>> {
            self.flush_outbound()?;
            let n = read_interrupted(&mut self.io, &mut self.io_buf).map_err(TransportError::Io)?;
            if n == 0 {
                return Ok(0);
            }
            if let Err(err) = self.client.recv(&self.io_buf[..n]) {
                let _ = self.flush_outbound();
                return Err(TransportError::H2(err));
            }
            self.flush_outbound()?;
            Ok(n)
        }

        /// Drives outbound and inbound I/O until call `stream_id` finishes and returns its [`Reply`].
        pub fn drive_until_reply(
            &mut self,
            stream_id: u32,
        ) -> Result<Reply<'b>, TransportError<io::Error>> {
            loop {
                self.flush_outbound()?;
                if let Some(reply) = self.client.take_reply(stream_id) {
                    return Ok(reply);
                }
                let n =
                    read_interrupted(&mut self.io, &mut self.io_buf).map_err(TransportError::Io)?;
                if n == 0 {
                    self.client
                        .fail_all_in_flight(GrpcCode::Unavailable, b"unexpected EOF from server");
                    return Err(TransportError::UnexpectedEof);
                }
                if let Err(err) = self.client.recv(&self.io_buf[..n]) {
                    let _ = self.flush_outbound();
                    return Err(TransportError::H2(err));
                }
            }
        }

        /// Queues and flushes an outbound HTTP/2 `PING` frame (`ack = false`).
        pub fn send_ping(&mut self, data: [u8; 8]) -> Result<(), TransportError<io::Error>> {
            self.client.send_ping(data).map_err(TransportError::H2)?;
            self.flush_outbound()
        }

        /// Drives the connection until a `PING` ACK frame is received from the peer.
        pub fn drive_until_ping_ack(&mut self) -> Result<[u8; 8], TransportError<io::Error>> {
            loop {
                if let Some(ack) = self.client.take_ping_ack() {
                    self.flush_outbound()?;
                    return Ok(ack);
                }
                let n = self.step()?;
                if let Some(ack) = self.client.take_ping_ack() {
                    return Ok(ack);
                }
                if n == 0 {
                    return Err(TransportError::UnexpectedEof);
                }
            }
        }

        /// Queues and flushes an outbound HTTP/2 `GOAWAY` frame on stream `0`.
        pub fn send_goaway(
            &mut self,
            error_code: H2ErrorCode,
            debug_data: &[u8],
        ) -> Result<(), TransportError<io::Error>> {
            self.client
                .send_goaway(error_code, debug_data)
                .map_err(TransportError::H2)?;
            self.flush_outbound()
        }
    }

    impl<
            'b,
            T: Read + Write,
            const MAX_STREAMS: usize,
            const REASM_CAP: usize,
            const IO_BUF_LEN: usize,
        > UnaryChannel for StdClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
    {
        fn call_unary<'a, F>(
            &'a mut self,
            path: &'static str,
            encode_req: F,
        ) -> Result<&'a [u8], GrpcStatus<'a>>
        where
            F: FnOnce(&mut BackwardWriter<'_>),
        {
            let buf = core::mem::take(&mut self.call_buf);
            let reply = match self.client.start_call(path, buf, encode_req) {
                Ok(stream_id) => match self.drive_until_reply(stream_id) {
                    Ok(reply) => reply,
                    Err(err) => {
                        let status = match err {
                            TransportError::UnexpectedEof => {
                                GrpcStatus::unavailable("unexpected EOF from server")
                            }
                            TransportError::H2(h2_err) => GrpcStatus::from(h2_err),
                            TransportError::Io(_) => GrpcStatus::unavailable("transport I/O error"),
                        };
                        self.client
                            .fail_call(stream_id, status.code, status.message.as_bytes());
                        if let Some(reply) = self.client.take_reply(stream_id) {
                            self.call_buf = reply.into_buf();
                        }
                        return Err(status);
                    }
                },
                Err(reply) => reply,
            };
            let code = reply.code;
            let len = reply.len;
            self.call_buf = reply.into_buf();
            reply_result(code, &self.call_buf[..len])
        }
    }

    impl<
            'b,
            T: Read + Write,
            const MAX_STREAMS: usize,
            const REASM_CAP: usize,
            const IO_BUF_LEN: usize,
        > AsyncUnaryChannel for StdClientTransport<'b, T, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>
    {
        async fn call_unary_async<'a, F>(
            &'a mut self,
            path: &'static str,
            encode_req: F,
        ) -> Result<&'a [u8], GrpcStatus<'a>>
        where
            Self: 'a,
            F: FnOnce(&mut BackwardWriter<'_>) + 'a,
        {
            self.call_unary(path, encode_req)
        }
    }

    /// Type alias for a [`StdClientTransport`] backed by a [`std::net::TcpStream`].
    pub type StdTcpClientTransport<
        'b,
        const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
        const REASM_CAP: usize = DEFAULT_REASM_CAP,
        const IO_BUF_LEN: usize = DEFAULT_IO_BUF_LEN,
    > = StdClientTransport<'b, TcpStream, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>;

    /// Connects a [`std::net::TcpStream`] to `addr`, enables `TCP_NODELAY`, and returns a
    /// [`StdTcpClientTransport`] ready for generated `<Service>Client` calls.
    pub fn connect_std_tcp<
        'b,
        A: ToSocketAddrs,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        addr: A,
        scheme: &'b str,
        authority: &'b str,
        tx_buf: &'b mut [u8],
        call_buf: &'b mut [u8],
    ) -> Result<StdTcpClientTransport<'b, MAX_STREAMS, REASM_CAP>, io::Error> {
        let stream = TcpStream::connect(addr)?;
        let _ = stream.set_nodelay(true);
        let client = GrpcClient::new(scheme, authority, tx_buf);
        Ok(StdClientTransport::new(stream, client, call_buf))
    }

    /// Drains all queued outbound frames from `server` into blocking `io` and flushes the stream.
    pub fn flush_std_server_outbound<
        I: Write,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
        io: &mut I,
        buf: &mut [u8],
    ) -> Result<(), TransportError<io::Error>> {
        flush_std_with(io, buf, |b| server.poll_transmit(b))
    }

    /// Flushes pending outbound frames, reads one chunk from blocking `io`, feeds [`GrpcServer::recv`],
    /// and flushes any resulting response or control frames (including `GOAWAY` on protocol error).
    pub fn step_std_server<
        I: Read + Write,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
        io: &mut I,
        buf: &mut [u8],
    ) -> Result<usize, TransportError<io::Error>> {
        flush_std_server_outbound(server, io, buf)?;
        let n = read_interrupted(io, buf).map_err(TransportError::Io)?;
        if n == 0 {
            return Ok(0);
        }
        if let Err(err) = server.recv(&buf[..n]) {
            let _ = flush_std_server_outbound(server, io, buf);
            return Err(TransportError::H2(err));
        }
        flush_std_server_outbound(server, io, buf)?;
        Ok(n)
    }

    /// Drives a synchronous [`GrpcServer`] over any blocking `std::io::Read + std::io::Write` stream
    /// (such as [`std::net::TcpStream`]) until clean client disconnect (`EOF` with no pending calls
    /// or partial frames) or a fatal transport/HTTP/2 error.
    ///
    /// If `server` has already served (or started serving) a previous connection and has not yet
    /// been explicitly [`GrpcServer::reset`], this function resets it automatically before sending
    /// the initial `SETTINGS` frame.
    pub fn serve_std_io<
        I: Read + Write,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
        mut io: I,
    ) -> Result<(), TransportError<io::Error>> {
        server.reset_if_active();
        let mut buf = [0u8; DEFAULT_IO_BUF_LEN];
        loop {
            let n = step_std_server(server, &mut io, &mut buf)?;
            if n == 0 {
                if server.has_pending_work() {
                    return Err(TransportError::UnexpectedEof);
                }
                return Ok(());
            }
        }
    }

    /// Accepts one connection from `listener`, sets `TCP_NODELAY`, and drives `server` until completion.
    pub fn accept_and_serve_std<const MAX_STREAMS: usize, const REASM_CAP: usize>(
        listener: &TcpListener,
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    ) -> Result<SocketAddr, TransportError<io::Error>> {
        let (stream, peer_addr) = listener.accept().map_err(TransportError::Io)?;
        let _ = stream.set_nodelay(true);
        serve_std_io(server, stream)?;
        Ok(peer_addr)
    }
}

// ============================================================================
// Optional `tokio` Datacenter Transport Adapter & Helpers (`feature = "tokio"`)
// ============================================================================

#[cfg(feature = "tokio")]
pub use tokio_support::{
    accept_and_serve_tokio, accept_and_serve_tokio_async, connect_tokio_tcp,
    serve_tokio_async_io, serve_tokio_io, TokioIo, TokioTcpClientTransport,
};

#[cfg(feature = "tokio")]
mod tokio_support {
    use std::io;
    use std::net::SocketAddr;

    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

    use super::{
        serve_async_connection, serve_connection, AsyncClientTransport, AsyncIo, TransportError,
        DEFAULT_IO_BUF_LEN,
    };
    use crate::client::GrpcClient;
    use crate::h2::{DEFAULT_MAX_STREAMS, DEFAULT_REASM_CAP};
    use crate::server::{AsyncGrpcServer, AsyncRouter, GrpcServer};

    /// Adapter implementing [`AsyncIo`] for any `tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin`
    /// stream (such as [`tokio::net::TcpStream`] or [`tokio::io::DuplexStream`]).
    #[must_use]
    #[derive(Debug)]
    pub struct TokioIo<T>(pub T);

    impl<T> TokioIo<T> {
        /// Wraps a Tokio async read/write stream in [`TokioIo`].
        #[inline]
        pub const fn new(inner: T) -> Self {
            Self(inner)
        }

        /// Returns a shared reference to the inner Tokio stream.
        #[must_use]
        #[inline]
        pub const fn inner(&self) -> &T {
            &self.0
        }

        /// Returns a mutable reference to the inner Tokio stream.
        #[inline]
        pub fn inner_mut(&mut self) -> &mut T {
            &mut self.0
        }

        /// Unwraps this adapter, returning the inner Tokio stream.
        #[must_use]
        #[inline]
        pub fn into_inner(self) -> T {
            self.0
        }
    }

    impl<T: AsyncWrite + Unpin> TokioIo<T> {
        /// Shuts down the write half of the underlying Tokio stream, sending EOF (`FIN`) to the peer.
        pub async fn shutdown(&mut self) -> Result<(), io::Error> {
            self.0.shutdown().await
        }
    }

    impl<T: AsyncRead + AsyncWrite + Unpin> AsyncIo for TokioIo<T> {
        type Error = io::Error;

        #[inline]
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.0.read(buf).await
        }

        #[inline]
        async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(buf).await
        }

        #[inline]
        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.0.flush().await
        }
    }

    /// Type alias for an [`AsyncClientTransport`] backed by a [`tokio::net::TcpStream`].
    pub type TokioTcpClientTransport<
        'b,
        const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
        const REASM_CAP: usize = DEFAULT_REASM_CAP,
        const IO_BUF_LEN: usize = DEFAULT_IO_BUF_LEN,
    > = AsyncClientTransport<'b, TokioIo<TcpStream>, MAX_STREAMS, REASM_CAP, IO_BUF_LEN>;

    /// Connects a [`tokio::net::TcpStream`] to `addr`, enables `TCP_NODELAY`, and returns a
    /// [`TokioTcpClientTransport`] ready for generated `<Service>Client` calls.
    pub async fn connect_tokio_tcp<
        'b,
        A: ToSocketAddrs,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        addr: A,
        scheme: &'b str,
        authority: &'b str,
        tx_buf: &'b mut [u8],
        call_buf: &'b mut [u8],
    ) -> Result<TokioTcpClientTransport<'b, MAX_STREAMS, REASM_CAP>, io::Error> {
        let stream = TcpStream::connect(addr).await?;
        let _ = stream.set_nodelay(true);
        let client = GrpcClient::new(scheme, authority, tx_buf);
        Ok(AsyncClientTransport::new(
            TokioIo::new(stream),
            client,
            call_buf,
        ))
    }

    /// Drives a synchronous [`GrpcServer`] over any Tokio `AsyncRead + AsyncWrite + Unpin` stream.
    pub async fn serve_tokio_io<
        I: AsyncRead + AsyncWrite + Unpin,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
        io: I,
    ) -> Result<(), TransportError<io::Error>> {
        let mut wrapped = TokioIo::new(io);
        serve_connection(server, &mut wrapped).await
    }

    /// Drives an [`AsyncGrpcServer`] over any Tokio `AsyncRead + AsyncWrite + Unpin` stream.
    pub async fn serve_tokio_async_io<
        I: AsyncRead + AsyncWrite + Unpin,
        R: AsyncRouter,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        server: &mut AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>,
        io: I,
    ) -> Result<(), TransportError<io::Error>> {
        let mut wrapped = TokioIo::new(io);
        serve_async_connection(server, &mut wrapped).await
    }

    /// Accepts one connection from `listener`, sets `TCP_NODELAY`, and drives `server` until completion.
    pub async fn accept_and_serve_tokio<const MAX_STREAMS: usize, const REASM_CAP: usize>(
        listener: &TcpListener,
        server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    ) -> Result<SocketAddr, TransportError<io::Error>> {
        let (stream, peer_addr) = listener.accept().await.map_err(TransportError::Io)?;
        let _ = stream.set_nodelay(true);
        serve_tokio_io(server, stream).await?;
        Ok(peer_addr)
    }

    /// Accepts one connection from `listener`, sets `TCP_NODELAY`, and drives an [`AsyncGrpcServer`] until completion.
    pub async fn accept_and_serve_tokio_async<
        R: AsyncRouter,
        const MAX_STREAMS: usize,
        const REASM_CAP: usize,
    >(
        listener: &TcpListener,
        server: &mut AsyncGrpcServer<'_, R, MAX_STREAMS, REASM_CAP>,
    ) -> Result<SocketAddr, TransportError<io::Error>> {
        let (stream, peer_addr) = listener.accept().await.map_err(TransportError::Io)?;
        let _ = stream.set_nodelay(true);
        serve_tokio_async_io(server, stream).await?;
        Ok(peer_addr)
    }
}
