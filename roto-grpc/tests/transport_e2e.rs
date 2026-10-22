//! End-to-end integration tests for `roto_grpc::transport` covering both the `#![no_std]`
//! `AsyncIo` transport driver (`MemoryDuplex`) and the optional `tokio` datacenter transport
//! driver (`TcpListener` / `TcpStream` and `tokio::io::duplex`).

use roto::runtime::{
    decode_varint_at, BackwardWriter, DecodeError, FieldIter, FieldMarker, MessageView, WireType,
};
use roto_grpc::framing::{prepend_grpc_header, take_writer, write_grpc_frame_bytes};
use roto_grpc::{
    block_on_pair, serve_async_connection, serve_connection, step_async_server, step_server,
    AsyncClientTransport, AsyncGrpcServer, AsyncGrpcService, AsyncUnaryChannel, GrpcClient,
    GrpcCode, GrpcServer, GrpcService, GrpcStatus, H2ErrorCode, MemoryDuplex, MemoryIoError, Route,
    TransportError,
};
#[cfg(feature = "std")]
use roto_grpc::UnaryChannel;

// ============================================================================
// Protobuf Message Views/Builders & Generated-Style Service Stubs
// ============================================================================

const UNICODE_FAIL_MSG: &str = "rejected: ünïcode 100% \n\t☃";

#[derive(Clone, Copy, Debug)]
struct EchoReqView<'a> {
    buf: &'a [u8],
    id: FieldMarker,
    payload: FieldMarker,
}

impl<'a> EchoReqView<'a> {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut id = FieldMarker::none();
        let mut payload = FieldMarker::none();
        for item in FieldIter::new(buf) {
            let f = item?;
            match (f.tag, f.wire_type) {
                (1, WireType::Varint) => id = f.marker,
                (2, WireType::LengthDelimited) => payload = f.marker,
                _ => {}
            }
        }
        Ok(Self { buf, id, payload })
    }

    fn id(&self) -> Result<u64, DecodeError> {
        if !self.id.is_present() {
            return Ok(0);
        }
        decode_varint_at(self.buf, self.id)
    }

    fn payload(&self) -> &'a [u8] {
        if !self.payload.is_present() {
            return &[];
        }
        self.payload.slice(self.buf)
    }
}

impl<'a> MessageView<'a> for EchoReqView<'a> {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

struct EchoReqBuilder<'b> {
    writer: BackwardWriter<'b>,
}

impl<'b> EchoReqBuilder<'b> {
    const fn from_writer(writer: BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    fn write_id(&mut self, val: u64) -> &mut Self {
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(1, WireType::Varint);
        self
    }

    fn write_payload(&mut self, val: &[u8]) -> &mut Self {
        let _ = self.writer.write_length_delimited(val);
        let _ = self.writer.write_tag(2, WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct EchoRespView<'a> {
    buf: &'a [u8],
    echoed_id: FieldMarker,
    payload: FieldMarker,
}

impl<'a> EchoRespView<'a> {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut echoed_id = FieldMarker::none();
        let mut payload = FieldMarker::none();
        for item in FieldIter::new(buf) {
            let f = item?;
            match (f.tag, f.wire_type) {
                (1, WireType::Varint) => echoed_id = f.marker,
                (2, WireType::LengthDelimited) => payload = f.marker,
                _ => {}
            }
        }
        Ok(Self {
            buf,
            echoed_id,
            payload,
        })
    }

    fn echoed_id(&self) -> Result<u64, DecodeError> {
        if !self.echoed_id.is_present() {
            return Ok(0);
        }
        decode_varint_at(self.buf, self.echoed_id)
    }

    fn payload(&self) -> &'a [u8] {
        if !self.payload.is_present() {
            return &[];
        }
        self.payload.slice(self.buf)
    }
}

struct EchoRespBuilder<'b> {
    writer: BackwardWriter<'b>,
}

impl<'b> EchoRespBuilder<'b> {
    const fn from_writer(writer: BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    fn write_echoed_id(&mut self, val: u64) -> &mut Self {
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(1, WireType::Varint);
        self
    }

    fn write_payload(&mut self, val: &[u8]) -> &mut Self {
        let _ = self.writer.write_length_delimited(val);
        let _ = self.writer.write_tag(2, WireType::LengthDelimited);
        self
    }
}

// Generated-equivalent service & client definitions (`test.v1.EchoService` & `test.v1.MathService`)

const ECHO_SERVICE_NAME: &str = "test.v1.EchoService";
const PATH_ECHO: &str = "/test.v1.EchoService/Echo";
const PATH_FAIL: &str = "/test.v1.EchoService/Fail";
const PATH_BIG: &str = "/test.v1.EchoService/Big";

struct EchoServiceServer;

impl GrpcService for EchoServiceServer {
    const SERVICE_NAME: &'static str = ECHO_SERVICE_NAME;

    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        match method_name {
            "Echo" => {
                let req = EchoReqView::from_bytes(req_payload)
                    .map_err(|_| GrpcStatus::internal("invalid EchoReq"))?;
                let id = req.id().map_err(|_| GrpcStatus::internal("invalid id"))?;
                if id == 999 {
                    return Err(GrpcStatus::invalid_argument("id 999 is reserved"));
                }
                let end_pos = writer.cursor();
                let mut resp = EchoRespBuilder::from_writer(take_writer(writer));
                resp.write_payload(req.payload()).write_echoed_id(id + 1);
                *writer = resp.writer;
                prepend_grpc_header(writer, false, end_pos).map_err(GrpcStatus::from)
            }
            "Fail" => Err(GrpcStatus::permission_denied(UNICODE_FAIL_MSG)),
            "Big" => {
                let len = u32::from_le_bytes(
                    req_payload
                        .try_into()
                        .map_err(|_| GrpcStatus::invalid_argument("expected 4-byte length"))?,
                ) as usize;
                let data: Vec<u8> = (0..len).map(|i| (i * 13 + 7) as u8).collect();
                write_grpc_frame_bytes(writer, false, &data).map_err(GrpcStatus::from)
            }
            _ => Err(GrpcStatus::unimplemented("unimplemented gRPC method")),
        }
    }
}

impl AsyncGrpcService for EchoServiceServer {
    const SERVICE_NAME: &'static str = ECHO_SERVICE_NAME;

    async fn call_unary_async<'a, 'b>(
        &'a self,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as GrpcService>::call_unary(self, method_name, req_payload, writer)
    }
}

const MATH_SERVICE_NAME: &str = "test.v1.MathService";
const PATH_DOUBLE: &str = "/test.v1.MathService/Double";

struct MathServiceServer;

impl GrpcService for MathServiceServer {
    const SERVICE_NAME: &'static str = MATH_SERVICE_NAME;

    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        match method_name {
            "Double" => {
                let req = EchoReqView::from_bytes(req_payload)
                    .map_err(|_| GrpcStatus::internal("invalid EchoReq"))?;
                let id = req.id().map_err(|_| GrpcStatus::internal("invalid id"))?;
                let end_pos = writer.cursor();
                let mut resp = EchoRespBuilder::from_writer(take_writer(writer));
                resp.write_payload(req.payload()).write_echoed_id(id * 2);
                *writer = resp.writer;
                prepend_grpc_header(writer, false, end_pos).map_err(GrpcStatus::from)
            }
            _ => Err(GrpcStatus::unimplemented("unimplemented MathService method")),
        }
    }
}

impl AsyncGrpcService for MathServiceServer {
    const SERVICE_NAME: &'static str = MATH_SERVICE_NAME;

    async fn call_unary_async<'a, 'b>(
        &'a self,
        method_name: &'a str,
        req_payload: &'a [u8],
        writer: &'a mut BackwardWriter<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        <Self as GrpcService>::call_unary(self, method_name, req_payload, writer)
    }
}

struct EchoServiceClient<C> {
    channel: C,
}

impl<C> EchoServiceClient<C> {
    const fn new(channel: C) -> Self {
        Self { channel }
    }
}

#[cfg(feature = "std")]
impl<C: UnaryChannel> EchoServiceClient<C> {
    fn echo<'a, F>(&'a mut self, build_req: F) -> Result<EchoRespView<'a>, GrpcStatus<'a>>
    where
        F: FnOnce(&mut EchoReqBuilder<'_>),
    {
        let resp_bytes = self.channel.call_unary(PATH_ECHO, |writer| {
            let mut req = EchoReqBuilder::from_writer(take_writer(writer));
            build_req(&mut req);
            *writer = req.writer;
        })?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }

    fn fail(&mut self) -> Result<EchoRespView<'_>, GrpcStatus<'_>> {
        let resp_bytes = self.channel.call_unary(PATH_FAIL, |_| {})?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }
}

impl<C: AsyncUnaryChannel> EchoServiceClient<C> {
    async fn echo_async<'a, F>(&'a mut self, build_req: F) -> Result<EchoRespView<'a>, GrpcStatus<'a>>
    where
        F: FnOnce(&mut EchoReqBuilder<'_>) + 'a,
    {
        let resp_bytes = self
            .channel
            .call_unary_async(PATH_ECHO, |writer| {
                let mut req = EchoReqBuilder::from_writer(take_writer(writer));
                build_req(&mut req);
                *writer = req.writer;
            })
            .await?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }

    async fn fail_async(&mut self) -> Result<EchoRespView<'_>, GrpcStatus<'_>> {
        let resp_bytes = self.channel.call_unary_async(PATH_FAIL, |_| {}).await?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }
}

struct MathServiceClient<C> {
    channel: C,
}

impl<C> MathServiceClient<C> {
    const fn new(channel: C) -> Self {
        Self { channel }
    }
}

#[cfg(feature = "std")]
impl<C: UnaryChannel> MathServiceClient<C> {
    fn double<'a, F>(&'a mut self, build_req: F) -> Result<EchoRespView<'a>, GrpcStatus<'a>>
    where
        F: FnOnce(&mut EchoReqBuilder<'_>),
    {
        let resp_bytes = self.channel.call_unary(PATH_DOUBLE, |writer| {
            let mut req = EchoReqBuilder::from_writer(take_writer(writer));
            build_req(&mut req);
            *writer = req.writer;
        })?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }
}

impl<C: AsyncUnaryChannel> MathServiceClient<C> {
    async fn double_async<'a, F>(
        &'a mut self,
        build_req: F,
    ) -> Result<EchoRespView<'a>, GrpcStatus<'a>>
    where
        F: FnOnce(&mut EchoReqBuilder<'_>) + 'a,
    {
        let resp_bytes = self
            .channel
            .call_unary_async(PATH_DOUBLE, |writer| {
                let mut req = EchoReqBuilder::from_writer(take_writer(writer));
                build_req(&mut req);
                *writer = req.writer;
            })
            .await?;
        EchoRespView::from_bytes(resp_bytes)
            .map_err(|_| GrpcStatus::internal("invalid EchoResp payload"))
    }
}

fn sync_routes() -> [&'static dyn Route; 2] {
    [&EchoServiceServer, &MathServiceServer]
}

fn run_with_large_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

// ============================================================================
// 1. `#![no_std]` `AsyncIo` Transport Tests (`MemoryDuplex`)
// ============================================================================

#[test]
fn nostd_transport_sync_and_async_server_unary_and_status_propagation() {
    let routes = sync_routes();

    // 1a. Sync GrpcServer over chunked MemoryDuplex with Poll::Pending yields.
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_io, server_io) = duplex.split();
        let client_io = client_io.with_max_chunk(13).with_pending_yields(true);
        let mut server_io = server_io.with_max_chunk(17).with_pending_yields(true);

        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);

        let ((), srv_res) = block_on_pair(
            async move {
                let mut transport = AsyncClientTransport::new(client_io, client, &mut call);

                // Call EchoService
                {
                    let mut echo = EchoServiceClient::new(&mut transport);
                    let resp = echo
                        .echo_async(|req| {
                            req.write_payload(b"rp2040-payload").write_id(41);
                        })
                        .await
                        .unwrap();
                    assert_eq!(resp.echoed_id().unwrap(), 42);
                    assert_eq!(resp.payload(), b"rp2040-payload");

                    // InvalidArgument error
                    let err_arg = echo
                        .echo_async(|req| {
                            req.write_id(999);
                        })
                        .await
                        .unwrap_err();
                    assert_eq!(err_arg.code, GrpcCode::InvalidArgument);
                    assert_eq!(err_arg.message, "id 999 is reserved");

                    // Unicode percent-encoded PermissionDenied error
                    let err_perm = echo.fail_async().await.unwrap_err();
                    assert_eq!(err_perm.code, GrpcCode::PermissionDenied);
                    assert_eq!(err_perm.message, UNICODE_FAIL_MSG);
                }

                // Call MathService over the same connection
                {
                    let mut math = MathServiceClient::new(&mut transport);
                    let resp = math
                        .double_async(|req| {
                            req.write_payload(b"math").write_id(21);
                        })
                        .await
                        .unwrap();
                    assert_eq!(resp.echoed_id().unwrap(), 42);
                    assert_eq!(resp.payload(), b"math");
                }

                // Unknown service -> Unimplemented
                let err_unimpl = transport
                    .call_unary_async("/unknown.v1.Svc/Method", |_| {})
                    .await
                    .unwrap_err();
                assert_eq!(err_unimpl.code, GrpcCode::Unimplemented);

                // Corrupted protobuf wire payload -> Internal
                let err_internal = transport
                    .call_unary_async(PATH_ECHO, |w| {
                        let _ = w.write_bytes(&[0xFF, 0xFF, 0xFF]);
                    })
                    .await
                    .unwrap_err();
                assert_eq!(err_internal.code, GrpcCode::Internal);

                // Server msg_buf overflow (600 > 512) -> ResourceExhausted
                let err_srv_exhausted = transport
                    .call_unary_async(PATH_BIG, |w| {
                        let _ = w.write_bytes(&600u32.to_le_bytes());
                    })
                    .await
                    .unwrap_err();
                assert_eq!(err_srv_exhausted.code, GrpcCode::ResourceExhausted);

                // Client call_buf overflow (300 > 128) -> ResourceExhausted
                let mut tiny_buf = [0u8; 128];
                let sid = transport
                    .start_call(PATH_BIG, &mut tiny_buf, |w| {
                        let _ = w.write_bytes(&300u32.to_le_bytes());
                    })
                    .unwrap();
                let r_exhausted = transport.drive_until_reply(sid).await.unwrap();
                assert_eq!(
                    r_exhausted.result().unwrap_err().code,
                    GrpcCode::ResourceExhausted
                );
                assert_eq!(r_exhausted.into_buf().len(), 128);
            },
            serve_connection(&mut server, &mut server_io),
        );
        assert!(srv_res.is_ok());
    }

    // 1b. AsyncGrpcServer with 2-service tuple `(EchoServiceServer, MathServiceServer)`
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_io, server_io) = duplex.split();
        let client_io = client_io.with_max_chunk(29).with_pending_yields(true);
        let mut server_io = server_io.with_max_chunk(31).with_pending_yields(true);

        let async_router = (EchoServiceServer, MathServiceServer);
        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
        let mut server: AsyncGrpcServer<_> =
            AsyncGrpcServer::new(&async_router, &mut msg, &mut stx);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);

        let ((), srv_res) = block_on_pair(
            async move {
                let mut transport = AsyncClientTransport::new(client_io, client, &mut call);
                {
                    let mut echo = EchoServiceClient::new(&mut transport);
                    let r = echo
                        .echo_async(|req| {
                            req.write_payload(b"async-svc").write_id(9);
                        })
                        .await
                        .unwrap();
                    assert_eq!(r.echoed_id().unwrap(), 10);
                    assert_eq!(r.payload(), b"async-svc");
                }
                {
                    let mut math = MathServiceClient::new(&mut transport);
                    let r = math
                        .double_async(|req| {
                            req.write_id(15);
                        })
                        .await
                        .unwrap();
                    assert_eq!(r.echoed_id().unwrap(), 30);
                }
                let err = transport
                    .call_unary_async("/unknown.Svc/Call", |_| {})
                    .await
                    .unwrap_err();
                assert_eq!(err.code, GrpcCode::Unimplemented);

                // Corrupted protobuf wire payload on AsyncGrpcServer -> Internal
                let err_internal = transport
                    .call_unary_async(PATH_ECHO, |w| {
                        let _ = w.write_bytes(&[0xFF, 0xFF, 0xFF]);
                    })
                    .await
                    .unwrap_err();
                assert_eq!(err_internal.code, GrpcCode::Internal);

                // AsyncGrpcServer msg_buf overflow (600 > 512) -> ResourceExhausted
                let err_exh = transport
                    .call_unary_async(PATH_BIG, |w| {
                        let _ = w.write_bytes(&600u32.to_le_bytes());
                    })
                    .await
                    .unwrap_err();
                assert_eq!(err_exh.code, GrpcCode::ResourceExhausted);
            },
            serve_async_connection(&mut server, &mut server_io),
        );
        assert!(srv_res.is_ok());
    }
}

#[test]
fn nostd_transport_multiplexed_calls_ping_goaway_and_large_messages() {
    run_with_large_stack(|| {
        const CAP: usize = 128 * 1024;
        const PAYLOAD_LEN: usize = 80 * 1024; // 80 KiB > 65,535-byte initial HTTP/2 window
        let routes = sync_routes();

        let duplex: MemoryDuplex<8192> = MemoryDuplex::new();
        let (client_io, mut server_io) = duplex.split();
        let mut msg = vec![0u8; CAP];
        let mut stx = vec![0u8; CAP];
        let mut ctx = vec![0u8; CAP];
        let mut call = vec![0u8; CAP];
        let mut server: GrpcServer<8, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let client: GrpcClient<8, CAP> = GrpcClient::new("http", "embedded", &mut ctx);

        let big_payload: Vec<u8> = (0..PAYLOAD_LEN).map(|i| (i * 19 + 3) as u8).collect();
        let expected_big_srv: Vec<u8> = (0..PAYLOAD_LEN).map(|i| (i * 13 + 7) as u8).collect();

        let ((), srv_res) = block_on_pair(
            async move {
                let mut transport: AsyncClientTransport<_, 8, CAP, 4096> =
                    AsyncClientTransport::new(client_io, client, &mut call);

                // 1. 6 concurrent multiplexed unary calls across services, collected out of order.
                let mut bufs = [[0u8; 256]; 6];
                let [b0, b1, b2, b3, b4, b5] = &mut bufs;
                let s1 = transport
                    .start_call(PATH_ECHO, b0, |w| {
                        let mut b = EchoReqBuilder::from_writer(take_writer(w));
                        b.write_payload(b"m1").write_id(1);
                        *w = b.writer;
                    })
                    .unwrap();
                let s3 = transport
                    .start_call(PATH_DOUBLE, b1, |w| {
                        let mut b = EchoReqBuilder::from_writer(take_writer(w));
                        b.write_payload(b"m2").write_id(20);
                        *w = b.writer;
                    })
                    .unwrap();
                let s5 = transport.start_call(PATH_FAIL, b2, |_| {}).unwrap();
                let s7 = transport
                    .start_call("/test.v1.Missing/No", b3, |_| {})
                    .unwrap();
                let s9 = transport
                    .start_call("/test.v1.EchoService/NoMethod", b4, |_| {})
                    .unwrap();
                let s11 = transport
                    .start_call(PATH_ECHO, b5, |w| {
                        let mut b = EchoReqBuilder::from_writer(take_writer(w));
                        b.write_id(999);
                        *w = b.writer;
                    })
                    .unwrap();
                assert_eq!([s1, s3, s5, s7, s9, s11], [1, 3, 5, 7, 9, 11]);

                // Send PING while all 6 calls are in flight!
                transport.send_ping(*b"keepaliv").await.unwrap();
                assert_eq!(transport.drive_until_ping_ack().await.unwrap(), *b"keepaliv");

                // Collect out of order: s11, s9, s7, s5, s3, s1
                let r11 = transport.drive_until_reply(s11).await.unwrap();
                assert_eq!(r11.result().unwrap_err().code, GrpcCode::InvalidArgument);

                let r9 = transport.drive_until_reply(s9).await.unwrap();
                assert_eq!(r9.result().unwrap_err().code, GrpcCode::Unimplemented);

                let r7 = transport.drive_until_reply(s7).await.unwrap();
                assert_eq!(r7.result().unwrap_err().code, GrpcCode::Unimplemented);

                let r5 = transport.drive_until_reply(s5).await.unwrap();
                assert_eq!(
                    r5.result().unwrap_err(),
                    GrpcStatus::permission_denied(UNICODE_FAIL_MSG)
                );

                let r3 = transport.drive_until_reply(s3).await.unwrap();
                let v3 = EchoRespView::from_bytes(r3.result().unwrap()).unwrap();
                assert_eq!(v3.echoed_id().unwrap(), 40);
                assert_eq!(v3.payload(), b"m2");

                let r1 = transport.drive_until_reply(s1).await.unwrap();
                let v1 = EchoRespView::from_bytes(r1.result().unwrap()).unwrap();
                assert_eq!(v1.echoed_id().unwrap(), 2);
                assert_eq!(v1.payload(), b"m1");

                // 2. Large > 65,535-byte request AND response over 8 KiB MemoryDuplex ring buffer.
                {
                    let mut echo = EchoServiceClient::new(&mut transport);
                    let resp = echo
                        .echo_async(|req| {
                            req.write_payload(&big_payload).write_id(100);
                        })
                        .await
                        .unwrap();
                    assert_eq!(resp.echoed_id().unwrap(), 101);
                    assert_eq!(resp.payload(), big_payload.as_slice());
                }
                let big_srv = transport
                    .call_unary_async(PATH_BIG, |w| {
                        let _ = w.write_bytes(&(PAYLOAD_LEN as u32).to_le_bytes());
                    })
                    .await
                    .unwrap();
                assert_eq!(big_srv, expected_big_srv.as_slice());

                // 3. Client sends GOAWAY before closing.
                transport
                    .send_goaway(H2ErrorCode::NoError, b"client closing")
                    .await
                    .unwrap();
            },
            serve_connection(&mut server, &mut server_io),
        );
        assert!(srv_res.is_ok());
        assert_eq!(server.goaway_received(), Some((0, H2ErrorCode::NoError)));
    });
}

#[test]
fn nostd_transport_server_goaway_drains_in_flight_and_rejects_higher_streams() {
    let routes = sync_routes();
    let duplex: MemoryDuplex<4096> = MemoryDuplex::new();
    let (client_io, mut server_io) = duplex.split();

    let (mut msg, mut stx, mut ctx, mut call) =
        ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 256]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);

    // Server pre-configures graceful GOAWAY at last_stream_id = 1 and sends a server PING.
    server
        .send_goaway_at(1, H2ErrorCode::NoError, b"draining")
        .unwrap();
    server.send_ping(*b"srv-ping").unwrap();

    let ((), srv_res) = block_on_pair(
        async move {
            let (mut b1, mut b3, mut b5) = ([0u8; 256], [0u8; 256], [0u8; 256]);
            let mut transport = AsyncClientTransport::new(client_io, client, &mut call);

            // Start stream 1 and stream 3 concurrently before receiving server's GOAWAY(1).
            let s1 = transport
                .start_call(PATH_ECHO, &mut b1, |w| {
                    let mut b = EchoReqBuilder::from_writer(take_writer(w));
                    b.write_payload(b"ok-1").write_id(10);
                    *w = b.writer;
                })
                .unwrap();
            let s3 = transport
                .start_call(PATH_ECHO, &mut b3, |w| {
                    let mut b = EchoReqBuilder::from_writer(take_writer(w));
                    b.write_payload(b"rejected-3").write_id(20);
                    *w = b.writer;
                })
                .unwrap();
            assert_eq!([s1, s3], [1, 3]);

            // Stream 1 (<= last_stream_id) completes with Ok!
            let r1 = transport.drive_until_reply(s1).await.unwrap();
            let v1 = EchoRespView::from_bytes(r1.result().unwrap()).unwrap();
            assert_eq!(v1.echoed_id().unwrap(), 11);
            assert_eq!(v1.payload(), b"ok-1");

            // Stream 3 (> last_stream_id) fails with Unavailable!
            let r3 = transport.drive_until_reply(s3).await.unwrap();
            assert_eq!(r3.result().unwrap_err().code, GrpcCode::Unavailable);
            assert_eq!(
                transport.goaway_received(),
                Some((1, H2ErrorCode::NoError))
            );

            // Subsequent start_call after GOAWAY is immediately rejected with Unavailable.
            let after = transport
                .start_call(PATH_ECHO, &mut b5, |_| {})
                .unwrap_err();
            assert_eq!(after.result().unwrap_err().code, GrpcCode::Unavailable);
        },
        serve_connection(&mut server, &mut server_io),
    );
    assert!(srv_res.is_ok());
    assert_eq!(server.take_ping_ack(), Some(*b"srv-ping"));
}

#[test]
fn nostd_transport_error_edge_cases_unexpected_eof_and_protocol_errors() {
    let routes = sync_routes();

    // 1a. Server drops connection before replying -> call_unary_async recovers call_buf and returns Unavailable.
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_io, server_io) = duplex.split();
        drop(server_io); // Immediately closed!

        let (mut ctx, mut call) = ([0u8; 1024], [0u8; 256]);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);
        let mut transport = AsyncClientTransport::new(client_io, client, &mut call);

        let (err_code, ()) = block_on_pair(
            async {
                transport
                    .call_unary_async(PATH_ECHO, |w| {
                        let _ = w.write_bytes(b"hello");
                    })
                    .await
                    .unwrap_err()
                    .code
            },
            async {},
        );
        assert_eq!(err_code, GrpcCode::Unavailable);
        assert_eq!(transport.call_buf_cap(), 256);
        assert!(format!("{transport:?}").contains("call_buf_cap: 256"));
    }

    // 1b. Server drops connection mid-call -> drive_until_reply returns Err(TransportError::UnexpectedEof)
    // and caller recovers the buffer via take_reply.
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_io, mut server_io) = duplex.split();
        let (mut ctx, mut call, mut slot_buf) = ([0u8; 1024], [0u8; 64], [0u8; 256]);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);
        let mut transport = AsyncClientTransport::new(client_io, client, &mut call);

        let ((drive_res, recovered_len), ()) = block_on_pair(
            async {
                let sid = transport
                    .start_call(PATH_ECHO, &mut slot_buf, |w| {
                        let _ = w.write_bytes(b"mid-call");
                    })
                    .unwrap();
                let res = transport.drive_until_reply(sid).await;
                let reply = transport.take_reply(sid).expect("reply must be recoverable");
                assert_eq!(reply.result().unwrap_err().code, GrpcCode::Unavailable);
                (res.map(|_| ()), reply.into_buf().len())
            },
            async move {
                use roto_grpc::AsyncIo;
                let mut scratch = [0u8; 64];
                let _ = server_io.read(&mut scratch).await;
                server_io.close();
            },
        );
        assert_eq!(drive_res, Err(TransportError::UnexpectedEof));
        assert_eq!(recovered_len, 256);
    }

    // 1c. Outbound flush fails after reply arrives -> call_unary_async still recovers call_buf!
    {
        #[derive(Debug)]
        struct FailAfterReplyIo<'a> {
            inner: roto_grpc::MemoryEndpoint<'a, 2048>,
            reads: usize,
        }
        impl roto_grpc::AsyncIo for FailAfterReplyIo<'_> {
            type Error = MemoryIoError;
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                let n = self.inner.read(buf).await?;
                self.reads += 1;
                Ok(n)
            }
            async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
                self.inner.write_all(buf).await
            }
            async fn flush(&mut self) -> Result<(), Self::Error> {
                if self.reads >= 1 {
                    // Simulate I/O error when flushing SETTINGS ACK after receiving server response
                    return Err(MemoryIoError::BrokenPipe);
                }
                self.inner.flush().await
            }
        }

        let duplex: MemoryDuplex<2048> = MemoryDuplex::new();
        let (client_io, mut server_io) = duplex.split();
        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 256], [0u8; 2048], [0u8; 2048], [0u8; 256]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);
        let mut transport = AsyncClientTransport::new(
            FailAfterReplyIo {
                inner: client_io,
                reads: 0,
            },
            client,
            &mut call,
        );

        let ((err, cap_after), ()) = block_on_pair(
            async move {
                let err = transport
                    .call_unary_async(PATH_ECHO, |w| {
                        let mut b = EchoReqBuilder::from_writer(take_writer(w));
                        b.write_payload(b"hi").write_id(1);
                        *w = b.writer;
                    })
                    .await
                    .unwrap_err();
                (err.code, transport.call_buf_cap())
            },
            async move {
                let mut buf = [0u8; 1024];
                let _ = step_server(&mut server, &mut server_io, &mut buf).await;
                drop(server_io); // Close so client's subsequent write fails with BrokenPipe
            },
        );
        assert_eq!(err, GrpcCode::Unavailable);
        assert_eq!(cap_after, 256);
    }

    // 2. Client sends partial preface and closes -> both sync GrpcServer and AsyncGrpcServer return UnexpectedEof.
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (mut client_io, mut server_io) = duplex.split();
        let (mut msg, mut stx) = ([0u8; 256], [0u8; 1024]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);

        let ((), srv_res) = block_on_pair(
            async move {
                use roto_grpc::AsyncIo;
                client_io.write_all(b"PRI * HTTP/2.0").await.unwrap();
                client_io.close();
            },
            serve_connection(&mut server, &mut server_io),
        );
        assert_eq!(srv_res, Err(TransportError::UnexpectedEof));

        let duplex_async: MemoryDuplex<1024> = MemoryDuplex::new();
        let (mut client_io_a, mut server_io_a) = duplex_async.split();
        let (mut msg_a, mut stx_a) = ([0u8; 256], [0u8; 1024]);
        let mut async_server: AsyncGrpcServer<_> =
            AsyncGrpcServer::new(&EchoServiceServer, &mut msg_a, &mut stx_a);

        let ((), async_srv_res) = block_on_pair(
            async move {
                use roto_grpc::AsyncIo;
                client_io_a.write_all(b"PRI * HTTP/2.0").await.unwrap();
                client_io_a.close();
            },
            serve_async_connection(&mut async_server, &mut server_io_a),
        );
        assert_eq!(async_srv_res, Err(TransportError::UnexpectedEof));
    }

    // 3. Client sends invalid preface -> both sync and async servers flush GOAWAY and return TransportError::H2.
    {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (mut client_io, mut server_io) = duplex.split();
        let (mut msg, mut stx) = ([0u8; 256], [0u8; 1024]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);

        let (read_bytes, srv_res) = block_on_pair(
            async move {
                use roto_grpc::AsyncIo;
                client_io
                    .write_all(b"BAD_PREFACE_24_BYTES!!!!")
                    .await
                    .unwrap();
                let mut rx = [0u8; 256];
                let mut total = 0;
                loop {
                    let n = client_io.read(&mut rx[total..]).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    total += n;
                }
                total
            },
            async move {
                let res = serve_connection(&mut server, &mut server_io).await;
                server_io.close();
                res
            },
        );
        assert!(matches!(srv_res, Err(TransportError::H2(_))));
        assert!(
            read_bytes >= 17,
            "server must flush SETTINGS + GOAWAY before returning error"
        );

        let duplex_async: MemoryDuplex<1024> = MemoryDuplex::new();
        let (mut client_io_a, mut server_io_a) = duplex_async.split();
        let (mut msg_a, mut stx_a) = ([0u8; 256], [0u8; 1024]);
        let mut async_server: AsyncGrpcServer<_> =
            AsyncGrpcServer::new(&EchoServiceServer, &mut msg_a, &mut stx_a);

        let (read_bytes_a, async_srv_res) = block_on_pair(
            async move {
                use roto_grpc::AsyncIo;
                client_io_a
                    .write_all(b"BAD_PREFACE_24_BYTES!!!!")
                    .await
                    .unwrap();
                let mut rx = [0u8; 256];
                let mut total = 0;
                loop {
                    let n = client_io_a.read(&mut rx[total..]).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    total += n;
                }
                total
            },
            async move {
                let res = serve_async_connection(&mut async_server, &mut server_io_a).await;
                server_io_a.close();
                res
            },
        );
        assert!(matches!(async_srv_res, Err(TransportError::H2(_))));
        assert!(
            read_bytes_a >= 17,
            "async server must flush SETTINGS + GOAWAY before returning error"
        );
    }

    // 4. AsyncGrpcServer step_async_server & multi-tuple routers (3-tuple & 4-tuple) + BrokenPipe
    {
        struct Svc3;
        impl AsyncGrpcService for Svc3 {
            const SERVICE_NAME: &'static str = "test.v1.Svc3";
            async fn call_unary_async<'a, 'b>(
                &'a self,
                _method: &'a str,
                req: &'a [u8],
                writer: &'a mut BackwardWriter<'b>,
            ) -> Result<(), GrpcStatus<'static>> {
                write_grpc_frame_bytes(writer, false, req).map_err(GrpcStatus::from)
            }
        }
        struct Svc4;
        impl AsyncGrpcService for Svc4 {
            const SERVICE_NAME: &'static str = "test.v1.Svc4";
            async fn call_unary_async<'a, 'b>(
                &'a self,
                _method: &'a str,
                req: &'a [u8],
                writer: &'a mut BackwardWriter<'b>,
            ) -> Result<(), GrpcStatus<'static>> {
                write_grpc_frame_bytes(writer, false, req).map_err(GrpcStatus::from)
            }
        }

        let router4 = (EchoServiceServer, MathServiceServer, Svc3, Svc4);
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_io, mut server_io) = duplex.split();
        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 256], [0u8; 2048], [0u8; 2048], [0u8; 256]);
        let mut server: AsyncGrpcServer<_> = AsyncGrpcServer::new(&router4, &mut msg, &mut stx);
        let client: GrpcClient = GrpcClient::new("http", "embedded", &mut ctx);

        let ((), srv_res) = block_on_pair(
            async move {
                let mut transport = AsyncClientTransport::new(client_io, client, &mut call);
                let _ = (transport.client(), transport.io());
                let r3 = transport
                    .call_unary_async("/test.v1.Svc3/Ping", |w| {
                        let _ = w.write_bytes(b"s3");
                    })
                    .await
                    .unwrap();
                assert_eq!(r3, b"s3");
                let r4 = transport
                    .call_unary_async("/test.v1.Svc4/Ping", |w| {
                        let _ = w.write_bytes(b"s4");
                    })
                    .await
                    .unwrap();
                assert_eq!(r4, b"s4");
                let err = transport
                    .call_unary_async("/test.v1.Svc5/Ping", |_| {})
                    .await
                    .unwrap_err();
                assert_eq!(err.code, GrpcCode::Unimplemented);
            },
            serve_async_connection(&mut server, &mut server_io),
        );
        assert!(srv_res.is_ok());

        // BrokenPipe when peer drops while ring buffer is full during a large write_all
        let duplex_full: MemoryDuplex<16> = MemoryDuplex::new();
        let (mut ep_a, ep_b) = duplex_full.split();
        let (write_res, ()) = block_on_pair(
            async {
                use roto_grpc::AsyncIo;
                let w = ep_a.write_all(&[0xAA; 64]).await;
                let mut empty = [];
                let r = ep_a.read(&mut empty).await;
                assert_eq!(r, Ok(0));
                w
            },
            async move {
                drop(ep_b);
            },
        );
        assert_eq!(write_res, Err(MemoryIoError::BrokenPipe));
        let _ = format!(
            "{} {} {}",
            MemoryIoError::BrokenPipe,
            TransportError::<MemoryIoError>::UnexpectedEof,
            TransportError::Io(MemoryIoError::BrokenPipe),
        );
        let _ = (
            step_server::<MemoryDuplexEndpoint, 8, 512>,
            step_async_server::<MemoryDuplexEndpoint, Svc3, 8, 512>,
        );
    }
}

type MemoryDuplexEndpoint<'a> = roto_grpc::MemoryEndpoint<'a, 1024>;

#[test]
fn direct_sans_io_tcp_listener_read_recv_poll_transmit_write_loop() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    // Server thread: pure Sans-I/O loop over std::net::TcpListener / TcpStream with zero adapters,
    // constructing GrpcServer once outside the accept loop and calling `server.reset()` on each
    // accepted connection (matching `docs/grpc.md`).
    let server_thread = std::thread::spawn(move || {
        let routes = sync_routes();
        let (mut msg_buf, mut tx_buf, mut io_buf) = ([0u8; 1024], [0u8; 4096], [0u8; 4096]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);

        for stream in listener.incoming().take(2) {
            let mut stream = stream.unwrap();
            let _ = stream.set_nodelay(true);
            server.reset();
            assert_eq!(server.last_stream_id(), 0);
            assert!(server.goaway_received().is_none());

            loop {
                // 1. Flush all queued outbound HTTP/2 frames (SETTINGS, ACKs, RPC responses)
                loop {
                    let tx_n = server.poll_transmit(&mut io_buf);
                    if tx_n == 0 {
                        break;
                    }
                    stream.write_all(&io_buf[..tx_n]).unwrap();
                }
                stream.flush().unwrap();

                // 2. Read incoming bytes from the TCP socket
                let rx_n = stream.read(&mut io_buf).unwrap();
                if rx_n == 0 {
                    assert!(!server.has_pending_work());
                    break;
                }

                // 3. Feed bytes into GrpcServer -> triggers handler when a complete request arrives
                if server.recv(&io_buf[..rx_n]).is_err() {
                    let tx_n = server.poll_transmit(&mut io_buf);
                    let _ = stream.write_all(&io_buf[..tx_n]);
                    break;
                }
            }
            assert_eq!(server.last_stream_id(), 1);
        }
    });

    let (mut ctx, mut call_buf, mut io_buf) = ([0u8; 4096], [0u8; 512], [0u8; 4096]);
    let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);

    // Connection 1: drive GrpcClient directly over std::net::TcpStream with raw poll_transmit / recv.
    let call_buf = {
        let mut stream = TcpStream::connect(addr).unwrap();
        let _ = stream.set_nodelay(true);

        let sid = client
            .start_call(PATH_ECHO, &mut call_buf, |w| {
                let mut req = EchoReqBuilder::from_writer(take_writer(w));
                req.write_payload(b"direct-sans-io").write_id(41);
                *w = req.writer;
            })
            .unwrap();
        assert_eq!(sid, 1);

        let reply = loop {
            loop {
                let tx_n = client.poll_transmit(&mut io_buf);
                if tx_n == 0 {
                    break;
                }
                stream.write_all(&io_buf[..tx_n]).unwrap();
            }
            stream.flush().unwrap();

            if let Some(reply) = client.take_reply(sid) {
                break reply;
            }

            let rx_n = stream.read(&mut io_buf).unwrap();
            assert!(rx_n > 0, "unexpected EOF waiting for reply");
            client.recv(&io_buf[..rx_n]).unwrap();
        };

        let view = EchoRespView::from_bytes(reply.result().unwrap()).unwrap();
        assert_eq!(view.echoed_id().unwrap(), 42);
        assert_eq!(view.payload(), b"direct-sans-io");
        stream.shutdown(Shutdown::Write).unwrap();
        reply.into_buf()
    };

    // Connection 2: call `client.reset()` and reuse the same GrpcClient & GrpcServer instances.
    {
        client.reset();
        let mut stream = TcpStream::connect(addr).unwrap();
        let _ = stream.set_nodelay(true);

        let sid = client
            .start_call(PATH_DOUBLE, call_buf, |w| {
                let mut req = EchoReqBuilder::from_writer(take_writer(w));
                req.write_payload(b"conn-2").write_id(25);
                *w = req.writer;
            })
            .unwrap();
        assert_eq!(sid, 1);

        let reply = loop {
            loop {
                let tx_n = client.poll_transmit(&mut io_buf);
                if tx_n == 0 {
                    break;
                }
                stream.write_all(&io_buf[..tx_n]).unwrap();
            }
            stream.flush().unwrap();

            if let Some(reply) = client.take_reply(sid) {
                break reply;
            }

            let rx_n = stream.read(&mut io_buf).unwrap();
            assert!(rx_n > 0, "unexpected EOF waiting for reply");
            client.recv(&io_buf[..rx_n]).unwrap();
        };

        let view = EchoRespView::from_bytes(reply.result().unwrap()).unwrap();
        assert_eq!(view.echoed_id().unwrap(), 50);
        assert_eq!(view.payload(), b"conn-2");
        let _ = reply.into_buf();
        stream.shutdown(Shutdown::Write).unwrap();
    }

    server_thread.join().unwrap();
}

#[test]
fn nostd_embassy_tcp_socket_macro_adapter_e2e() {
    /// Simulated `embassy_net::tcp::TcpSocket<'a>` exposing inherent `read`, partial `write`,
    /// and `flush` async methods (without implementing `AsyncIo` directly).
    struct MockEmbassyTcpSocket<'a> {
        ep: roto_grpc::MemoryEndpoint<'a, 1024>,
    }

    impl<'a> MockEmbassyTcpSocket<'a> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, MemoryIoError> {
            use roto_grpc::AsyncIo;
            self.ep.read(buf).await
        }

        async fn write(&mut self, buf: &[u8]) -> Result<usize, MemoryIoError> {
            use roto_grpc::AsyncIo;
            // Write at most 11 bytes per call to verify `impl_async_io!` loops partial writes.
            let n = buf.len().min(11);
            self.ep.write_all(&buf[..n]).await?;
            Ok(n)
        }

        async fn flush(&mut self) -> Result<(), MemoryIoError> {
            use roto_grpc::AsyncIo;
            self.ep.flush().await
        }
    }

    roto_grpc::impl_async_io!(
        struct EmbassySocketIo<'a>(MockEmbassyTcpSocket<'a>) -> MemoryIoError
    );

    let routes = sync_routes();
    let (mut msg, mut stx, mut ctx, mut call) =
        ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);

    // Serve two sequential RP2040W socket connections on the same GrpcServer via `server.reset()`
    // (matching the `rp2040w_grpc_server_task` loop in `docs/grpc.md`).
    for idx in 0..2u64 {
        let duplex: MemoryDuplex<1024> = MemoryDuplex::new();
        let (client_ep, server_ep) = duplex.split();

        let client_sock = EmbassySocketIo::new(MockEmbassyTcpSocket {
            ep: client_ep.with_max_chunk(13).with_pending_yields(true),
        });
        let mut server_sock = EmbassySocketIo::new(MockEmbassyTcpSocket {
            ep: server_ep.with_max_chunk(17).with_pending_yields(true),
        });
        let _ = server_sock.inner();
        let _ = server_sock.inner_mut();

        server.reset();
        let client: GrpcClient = GrpcClient::new("http", "rp2040w.local", &mut ctx);
        let call_slice: &mut [u8] = &mut call;

        let ((), srv_res) = block_on_pair(
            async move {
                let mut transport = AsyncClientTransport::new(client_sock, client, call_slice);
                {
                    let mut echo = EchoServiceClient::new(&mut transport);
                    let resp = echo
                        .echo_async(|req| {
                            req.write_payload(b"cyw43-rp2040w").write_id(100 + idx);
                        })
                        .await
                        .unwrap();
                    assert_eq!(resp.echoed_id().unwrap(), 101 + idx);
                    assert_eq!(resp.payload(), b"cyw43-rp2040w");
                }
                let raw_sock = transport.into_inner().into_inner();
                raw_sock.ep.close();
            },
            serve_connection(&mut server, &mut server_sock),
        );
        assert!(srv_res.is_ok());
    }
}

// ============================================================================
// 2. Optional `std` Blocking TCP Transport Tests (`feature = "std"`)
// ============================================================================

#[cfg(feature = "std")]
mod std_tests {
    use std::net::{Shutdown, TcpListener};

    use super::*;
    use roto_grpc::{
        accept_and_serve_std, connect_std_tcp, flush_std_server_outbound, serve_std_io,
        step_std_server, StdClientTransport, StdIo, StdTcpClientTransport,
    };

    #[test]
    fn std_tcp_unary_and_multiplexed_calls_and_std_io_adapter() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_thread = std::thread::spawn(move || {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 1024], [0u8; 16384]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            let peer = accept_and_serve_std(&listener, &mut server).unwrap();
            assert!(peer.ip().is_loopback());
            assert_eq!(server.goaway_received(), Some((0, H2ErrorCode::NoError)));
        });

        let (mut ctx, mut call) = ([0u8; 16384], [0u8; 1024]);
        let mut transport: StdTcpClientTransport<'_> =
            connect_std_tcp(addr, "http", "localhost", &mut ctx, &mut call).unwrap();
        assert_eq!(transport.call_buf_cap(), 1024);
        let _ = (transport.client(), transport.io());
        let _ = transport.client_mut();
        assert!(format!("{transport:?}").contains("StdClientTransport"));

        // 1. Synchronous UnaryChannel calls via EchoServiceClient and MathServiceClient
        {
            let mut echo = EchoServiceClient::new(&mut transport);
            let resp = echo
                .echo(|req| {
                    req.write_payload(b"over-std-tcp").write_id(77);
                })
                .unwrap();
            assert_eq!(resp.echoed_id().unwrap(), 78);
            assert_eq!(resp.payload(), b"over-std-tcp");

            let err_perm = echo.fail().unwrap_err();
            assert_eq!(err_perm.code, GrpcCode::PermissionDenied);
            assert_eq!(err_perm.message, UNICODE_FAIL_MSG);

            let err_arg = echo
                .echo(|req| {
                    req.write_id(999);
                })
                .unwrap_err();
            assert_eq!(err_arg.code, GrpcCode::InvalidArgument);
        }

        {
            let mut math = MathServiceClient::new(&mut transport);
            let resp = math
                .double(|req| {
                    req.write_payload(b"std-math").write_id(21);
                })
                .unwrap();
            assert_eq!(resp.echoed_id().unwrap(), 42);
            assert_eq!(resp.payload(), b"std-math");
        }

        // 2. Concurrent multiplexed calls + PING keepalive over blocking std::net::TcpStream
        let mut bufs = [[0u8; 256]; 4];
        let [b0, b1, b2, b3] = &mut bufs;
        let s1 = transport
            .start_call(PATH_ECHO, b0, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"m1").write_id(10);
                *w = b.writer;
            })
            .unwrap();
        let s2 = transport
            .start_call(PATH_DOUBLE, b1, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"m2").write_id(15);
                *w = b.writer;
            })
            .unwrap();
        let s3 = transport.start_call(PATH_FAIL, b2, |_| {}).unwrap();
        let s4 = transport
            .start_call("/test.v1.Unknown/Rpc", b3, |_| {})
            .unwrap();

        transport.send_ping(*b"std-ping").unwrap();
        assert_eq!(transport.drive_until_ping_ack().unwrap(), *b"std-ping");

        // Collect out of order: s4, s3, s2, s1
        let r4 = transport.drive_until_reply(s4).unwrap();
        assert_eq!(r4.result().unwrap_err().code, GrpcCode::Unimplemented);

        let r3 = transport.drive_until_reply(s3).unwrap();
        assert_eq!(
            r3.result().unwrap_err(),
            GrpcStatus::permission_denied(UNICODE_FAIL_MSG)
        );

        let r2 = transport.drive_until_reply(s2).unwrap();
        let v2 = EchoRespView::from_bytes(r2.result().unwrap()).unwrap();
        assert_eq!(v2.echoed_id().unwrap(), 30);
        assert_eq!(v2.payload(), b"m2");

        let r1 = transport.drive_until_reply(s1).unwrap();
        let v1 = EchoRespView::from_bytes(r1.result().unwrap()).unwrap();
        assert_eq!(v1.echoed_id().unwrap(), 11);
        assert_eq!(v1.payload(), b"m1");

        // 3. Graceful GOAWAY and clean socket close
        transport
            .send_goaway(H2ErrorCode::NoError, b"done")
            .unwrap();
        transport.io_mut().shutdown(Shutdown::Write).unwrap();
        let _ = transport.into_inner();
        server_thread.join().unwrap();

        // 4. Test StdIo<TcpStream> with AsyncGrpcServer (reused across 2 sequential connections)
        //    + step_std_server / flush_std_server_outbound
        let listener2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr2 = listener2.local_addr().unwrap();

        let server_thread2 = std::thread::spawn(move || {
            let router = (EchoServiceServer, MathServiceServer);
            let (mut msg, mut stx) = ([0u8; 512], [0u8; 4096]);
            let mut async_server: AsyncGrpcServer<_> =
                AsyncGrpcServer::new(&router, &mut msg, &mut stx);

            for i in 0..2 {
                let (stream, _) = listener2.accept().unwrap();
                let _ = stream.set_nodelay(true);
                let mut std_io = StdIo::new(stream);
                let _ = std_io.inner();
                let _ = std_io.inner_mut();
                if i == 1 {
                    async_server.reset();
                }
                let (srv_res, ()) = block_on_pair(
                    serve_async_connection(&mut async_server, &mut std_io),
                    async {},
                );
                assert!(srv_res.is_ok());
                let _ = std_io.into_inner();
            }
        });

        for i in 0..2u64 {
            let (mut ctx2, mut call2) = ([0u8; 4096], [0u8; 512]);
            let mut transport2 =
                connect_std_tcp::<_, 4, 2048>(addr2, "http", "localhost", &mut ctx2, &mut call2)
                    .unwrap();
            {
                let mut echo = EchoServiceClient::new(&mut transport2);
                let resp = echo
                    .echo(|req| {
                        req.write_payload(b"std-io-async-srv").write_id(5 + i);
                    })
                    .unwrap();
                assert_eq!(resp.echoed_id().unwrap(), 6 + i);
                assert_eq!(resp.payload(), b"std-io-async-srv");
            }
            let std_io_client = StdIo::new(transport2.into_inner());
            std_io_client.shutdown().unwrap();
            drop(std_io_client);
        }
        server_thread2.join().unwrap();

        // 5. Test `accept_and_serve_std` across 2 sequential connections on the same GrpcServer
        let listener3 = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr3 = listener3.local_addr().unwrap();
        let server_thread3 = std::thread::spawn(move || {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 512], [0u8; 4096]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            for _ in 0..2 {
                accept_and_serve_std(&listener3, &mut server).unwrap();
            }
        });
        for i in 0..2u64 {
            let (mut ctx3, mut call3) = ([0u8; 4096], [0u8; 512]);
            let mut t3: StdTcpClientTransport<'_> =
                connect_std_tcp(addr3, "http", "localhost", &mut ctx3, &mut call3).unwrap();
            {
                let mut echo = EchoServiceClient::new(&mut t3);
                let r = echo.echo(|req| { req.write_id(10 + i); }).unwrap();
                assert_eq!(r.echoed_id().unwrap(), 11 + i);
            }
            t3.io_mut().shutdown(Shutdown::Write).unwrap();
        }
        server_thread3.join().unwrap();

        let _ = (
            serve_std_io::<std::net::TcpStream, 4, 1024>,
            step_std_server::<std::net::TcpStream, 4, 1024>,
            flush_std_server_outbound::<std::net::TcpStream, 4, 1024>,
            StdClientTransport::<std::net::TcpStream>::new,
        );
    }
}

// ============================================================================
// 3. Optional `tokio` Datacenter Transport Tests (`TcpListener` / `TcpStream` & `duplex`)
// ============================================================================

#[cfg(feature = "tokio")]
mod tokio_tests {
    use tokio::net::TcpListener;

    use super::*;
    use roto_grpc::{
        accept_and_serve_tokio, accept_and_serve_tokio_async, connect_tokio_tcp,
        serve_tokio_async_io, serve_tokio_io, TokioIo,
    };

    #[tokio::test]
    async fn tokio_tcp_unary_calls_and_error_propagation_sync_and_async_servers() {
        // 1. Real TCP loopback with synchronous GrpcServer (`accept_and_serve_tokio` + `connect_tokio_tcp`)
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 1024], [0u8; 8192]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            accept_and_serve_tokio(&listener, &mut server).await.unwrap();
        });

        let (mut ctx, mut call) = ([0u8; 8192], [0u8; 1024]);
        let mut transport: roto_grpc::TokioTcpClientTransport<'_> =
            connect_tokio_tcp(addr, "http", "localhost", &mut ctx, &mut call)
                .await
                .unwrap();

        {
            let mut echo = EchoServiceClient::new(&mut transport);
            let resp = echo
                .echo_async(|req| {
                    req.write_payload(b"over-tokio-tcp").write_id(77);
                })
                .await
                .unwrap();
            assert_eq!(resp.echoed_id().unwrap(), 78);
            assert_eq!(resp.payload(), b"over-tokio-tcp");

            let err_perm = echo.fail_async().await.unwrap_err();
            assert_eq!(err_perm.code, GrpcCode::PermissionDenied);
            assert_eq!(err_perm.message, UNICODE_FAIL_MSG);

            let err_arg = echo
                .echo_async(|req| {
                    req.write_id(999);
                })
                .await
                .unwrap_err();
            assert_eq!(err_arg.code, GrpcCode::InvalidArgument);
        }

        {
            let mut math = MathServiceClient::new(&mut transport);
            let resp = math
                .double_async(|req| {
                    req.write_payload(b"tcp-math").write_id(50);
                })
                .await
                .unwrap();
            assert_eq!(resp.echoed_id().unwrap(), 100);
            assert_eq!(resp.payload(), b"tcp-math");
        }

        let err_unimpl = transport
            .call_unary_async("/missing.Service/Rpc", |_| {})
            .await
            .unwrap_err();
        assert_eq!(err_unimpl.code, GrpcCode::Unimplemented);

        // Corrupted protobuf wire payload over Tokio TCP -> Internal
        let err_internal = transport
            .call_unary_async(PATH_ECHO, |w| {
                let _ = w.write_bytes(&[0xFF, 0xFF, 0xFF]);
            })
            .await
            .unwrap_err();
        assert_eq!(err_internal.code, GrpcCode::Internal);

        // Response exceeding server's 1024-byte msg_buf over Tokio TCP -> ResourceExhausted
        let err_exhausted = transport
            .call_unary_async(PATH_BIG, |w| {
                let _ = w.write_bytes(&2000u32.to_le_bytes());
            })
            .await
            .unwrap_err();
        assert_eq!(err_exhausted.code, GrpcCode::ResourceExhausted);

        transport.io_mut().shutdown().await.unwrap();
        drop(transport);
        server_task.await.unwrap();

        // 2. Real TCP loopback with `AsyncGrpcServer` (`accept_and_serve_tokio_async`) across 2 sequential connections
        let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr2 = listener2.local_addr().unwrap();

        let async_server_task = tokio::spawn(async move {
            let router = (EchoServiceServer, MathServiceServer);
            let (mut msg, mut stx) = ([0u8; 1024], [0u8; 8192]);
            let mut server: AsyncGrpcServer<_> =
                AsyncGrpcServer::new(&router, &mut msg, &mut stx);
            for _ in 0..2 {
                accept_and_serve_tokio_async(&listener2, &mut server)
                    .await
                    .unwrap();
            }
        });

        for i in 0..2u64 {
            let (mut ctx2, mut call2) = ([0u8; 8192], [0u8; 1024]);
            let mut transport2: roto_grpc::TokioTcpClientTransport<'_> =
                connect_tokio_tcp(addr2, "http", "localhost", &mut ctx2, &mut call2)
                    .await
                    .unwrap();
            {
                let mut echo = EchoServiceClient::new(&mut transport2);
                let resp = echo
                    .echo_async(|req| {
                        req.write_payload(b"async-tcp").write_id(123 + i);
                    })
                    .await
                    .unwrap();
                assert_eq!(resp.echoed_id().unwrap(), 124 + i);
                assert_eq!(resp.payload(), b"async-tcp");
            }
            transport2.io_mut().shutdown().await.unwrap();
            drop(transport2);
        }
        async_server_task.await.unwrap();
    }

    #[tokio::test]
    async fn tokio_tcp_multiplexed_concurrent_calls_and_ping_keepalive() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 1024], [0u8; 16384]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            accept_and_serve_tokio(&listener, &mut server).await.unwrap();
        });

        let (mut ctx, mut call) = ([0u8; 16384], [0u8; 512]);
        let mut transport: roto_grpc::TokioTcpClientTransport<'_> =
            connect_tokio_tcp(addr, "http", "localhost", &mut ctx, &mut call)
                .await
                .unwrap();

        let mut bufs = [[0u8; 256]; 6];
        let [b0, b1, b2, b3, b4, b5] = &mut bufs;

        let s1 = transport
            .start_call(PATH_ECHO, b0, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"c1").write_id(10);
                *w = b.writer;
            })
            .unwrap();
        let s3 = transport
            .start_call(PATH_DOUBLE, b1, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"c3").write_id(11);
                *w = b.writer;
            })
            .unwrap();
        let s5 = transport.start_call(PATH_FAIL, b2, |_| {}).unwrap();
        let s7 = transport
            .start_call("/test.v1.Unknown/Method", b3, |_| {})
            .unwrap();
        let s9 = transport
            .start_call(PATH_ECHO, b4, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"c9").write_id(90);
                *w = b.writer;
            })
            .unwrap();
        let s11 = transport
            .start_call(PATH_DOUBLE, b5, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"c11").write_id(33);
                *w = b.writer;
            })
            .unwrap();

        // Send PING keepalive while all 6 multiplexed calls are in flight over TCP!
        transport.send_ping(*b"tcp-ping").await.unwrap();
        assert_eq!(transport.drive_until_ping_ack().await.unwrap(), *b"tcp-ping");

        // Collect out of order: s11, s9, s7, s5, s3, s1
        let r11 = transport.drive_until_reply(s11).await.unwrap();
        let v11 = EchoRespView::from_bytes(r11.result().unwrap()).unwrap();
        assert_eq!(v11.echoed_id().unwrap(), 66);
        assert_eq!(v11.payload(), b"c11");

        let r9 = transport.drive_until_reply(s9).await.unwrap();
        let v9 = EchoRespView::from_bytes(r9.result().unwrap()).unwrap();
        assert_eq!(v9.echoed_id().unwrap(), 91);
        assert_eq!(v9.payload(), b"c9");

        let r7 = transport.drive_until_reply(s7).await.unwrap();
        assert_eq!(r7.result().unwrap_err().code, GrpcCode::Unimplemented);

        let r5 = transport.drive_until_reply(s5).await.unwrap();
        assert_eq!(
            r5.result().unwrap_err(),
            GrpcStatus::permission_denied(UNICODE_FAIL_MSG)
        );

        let r3 = transport.drive_until_reply(s3).await.unwrap();
        let v3 = EchoRespView::from_bytes(r3.result().unwrap()).unwrap();
        assert_eq!(v3.echoed_id().unwrap(), 22);
        assert_eq!(v3.payload(), b"c3");

        let r1 = transport.drive_until_reply(s1).await.unwrap();
        let v1 = EchoRespView::from_bytes(r1.result().unwrap()).unwrap();
        assert_eq!(v1.echoed_id().unwrap(), 11);
        assert_eq!(v1.payload(), b"c1");

        transport.io_mut().shutdown().await.unwrap();
        drop(transport);
        server_task.await.unwrap();
    }

    #[test]
    fn tokio_tcp_large_multi_frame_messages_exceeding_initial_window() {
        super::run_with_large_stack(|| {
            const CAP: usize = 128 * 1024;
            const PAYLOAD_LEN: usize = 96 * 1024; // 96 KiB > 65,535-byte initial window

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();

                let server_task = tokio::spawn(async move {
                    let routes = sync_routes();
                    let mut msg = vec![0u8; CAP];
                    let mut stx = vec![0u8; CAP];
                    let mut server: GrpcServer<4, CAP> =
                        GrpcServer::new(&routes, &mut msg, &mut stx);
                    accept_and_serve_tokio(&listener, &mut server).await.unwrap();
                });

                let mut ctx = vec![0u8; CAP];
                let mut call = vec![0u8; CAP];
                let mut transport: roto_grpc::TokioTcpClientTransport<'_, 4, CAP> =
                    connect_tokio_tcp(addr, "http", "localhost", &mut ctx, &mut call)
                        .await
                        .unwrap();

                let payload: Vec<u8> = (0..PAYLOAD_LEN).map(|i| (i * 29 + 5) as u8).collect();
                let expected_big: Vec<u8> =
                    (0..PAYLOAD_LEN).map(|i| (i * 13 + 7) as u8).collect();

                // 1. 96 KiB Echo request and 96 KiB Echo response over real TCP socket
                {
                    let mut echo = EchoServiceClient::new(&mut transport);
                    let resp = echo
                        .echo_async(|req| {
                            req.write_payload(&payload).write_id(500);
                        })
                        .await
                        .unwrap();
                    assert_eq!(resp.echoed_id().unwrap(), 501);
                    assert_eq!(resp.payload(), payload.as_slice());
                }

                // 2. 96 KiB Big response over the same connection (requiring further WINDOW_UPDATE replenishment)
                let big_resp = transport
                    .call_unary_async(PATH_BIG, |w| {
                        let _ = w.write_bytes(&(PAYLOAD_LEN as u32).to_le_bytes());
                    })
                    .await
                    .unwrap();
                assert_eq!(big_resp, expected_big.as_slice());

                transport.io_mut().shutdown().await.unwrap();
                drop(transport);
                server_task.await.unwrap();
            });
        });
    }

    #[tokio::test]
    async fn tokio_tcp_and_duplex_graceful_goaway_shutdown() {
        // 1. Server sends GOAWAY(last_stream_id = 1) over TCP: stream 1 completes, stream 3 fails with Unavailable
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 512], [0u8; 4096]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            server
                .send_goaway_at(1, H2ErrorCode::NoError, b"server shutting down")
                .unwrap();
            accept_and_serve_tokio(&listener, &mut server).await.unwrap();
        });

        let (mut ctx, mut call) = ([0u8; 4096], [0u8; 256]);
        let mut transport: roto_grpc::TokioTcpClientTransport<'_> =
            connect_tokio_tcp(addr, "http", "localhost", &mut ctx, &mut call)
                .await
                .unwrap();

        let (mut b1, mut b3) = ([0u8; 256], [0u8; 256]);
        let s1 = transport
            .start_call(PATH_ECHO, &mut b1, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"graceful-1").write_id(1);
                *w = b.writer;
            })
            .unwrap();
        let s3 = transport
            .start_call(PATH_ECHO, &mut b3, |w| {
                let mut b = EchoReqBuilder::from_writer(take_writer(w));
                b.write_payload(b"rejected-3").write_id(3);
                *w = b.writer;
            })
            .unwrap();

        let r1 = transport.drive_until_reply(s1).await.unwrap();
        let v1 = EchoRespView::from_bytes(r1.result().unwrap()).unwrap();
        assert_eq!(v1.echoed_id().unwrap(), 2);
        assert_eq!(v1.payload(), b"graceful-1");

        let r3 = transport.drive_until_reply(s3).await.unwrap();
        assert_eq!(r3.result().unwrap_err().code, GrpcCode::Unavailable);
        assert_eq!(
            transport.goaway_received(),
            Some((1, H2ErrorCode::NoError))
        );

        transport.io_mut().shutdown().await.unwrap();
        drop(transport);
        server_task.await.unwrap();

        // 2. tokio::io::duplex with serve_tokio_io & serve_tokio_async_io
        let (client_dup, server_dup) = tokio::io::duplex(2048);
        let srv_dup_task = tokio::spawn(async move {
            let routes = sync_routes();
            let (mut msg, mut stx) = ([0u8; 512], [0u8; 4096]);
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            serve_tokio_io(&mut server, server_dup).await.unwrap();
            assert_eq!(server.goaway_received(), Some((0, H2ErrorCode::NoError)));
        });

        let (mut ctx2, mut call2) = ([0u8; 4096], [0u8; 256]);
        let client2: GrpcClient = GrpcClient::new("http", "duplex", &mut ctx2);
        let mut dup_transport =
            AsyncClientTransport::new(TokioIo::new(client_dup), client2, &mut call2);
        {
            let mut echo = EchoServiceClient::new(&mut dup_transport);
            let r = echo
                .echo_async(|req| {
                    req.write_payload(b"tokio-duplex").write_id(7);
                })
                .await
                .unwrap();
            assert_eq!(r.echoed_id().unwrap(), 8);
            assert_eq!(r.payload(), b"tokio-duplex");
        }
        dup_transport
            .send_goaway(H2ErrorCode::NoError, b"bye")
            .await
            .unwrap();
        let mut inner_dup = dup_transport.into_inner();
        let _ = inner_dup.inner();
        let _ = inner_dup.inner_mut();
        inner_dup.shutdown().await.unwrap();
        drop(inner_dup.into_inner());
        srv_dup_task.await.unwrap();

        // 3. tokio::io::duplex with serve_tokio_async_io
        let (client_dup2, server_dup2) = tokio::io::duplex(2048);
        let srv_dup2_task = tokio::spawn(async move {
            let (mut msg, mut stx) = ([0u8; 512], [0u8; 4096]);
            let mut server: AsyncGrpcServer<_> =
                AsyncGrpcServer::new(&EchoServiceServer, &mut msg, &mut stx);
            serve_tokio_async_io(&mut server, server_dup2).await.unwrap();
        });
        let (mut ctx3, mut call3) = ([0u8; 4096], [0u8; 256]);
        let client3: GrpcClient = GrpcClient::new("http", "duplex", &mut ctx3);
        let mut dup_transport2 =
            AsyncClientTransport::new(TokioIo::new(client_dup2), client3, &mut call3);
        {
            let mut echo = EchoServiceClient::new(&mut dup_transport2);
            let r = echo
                .echo_async(|req| {
                    req.write_id(10);
                })
                .await
                .unwrap();
            assert_eq!(r.echoed_id().unwrap(), 11);
        }
        dup_transport2.io_mut().shutdown().await.unwrap();
        drop(dup_transport2);
        srv_dup2_task.await.unwrap();
    }
}
