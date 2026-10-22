//! In-memory Sans-I/O client <-> server loopback integration tests for `roto_grpc::{server, client}`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use roto::runtime::BackwardWriter;
use roto_grpc::framing::write_grpc_frame_bytes;
use roto_grpc::h2::{
    FrameHeader, FrameType, H2Connection, H2ErrorCode, H2Event, Role, FRAME_HEADER_LEN,
};
use roto_grpc::hpack::{GrpcTrailers, HpackDecoder, HpackEncoder};
use roto_grpc::status::percent_decode;
use roto_grpc::{
    GrpcClient, GrpcCode, GrpcServer, GrpcService, GrpcStatus, H2LoopbackChannel, Route,
    UnaryChannel,
};

// ============================================================================
// Per-thread allocation counter proving the loopback path is heap-free
// ============================================================================

struct CountingAlloc;

thread_local! {
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCS.try_with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn allocs() -> usize {
    ALLOCS.with(Cell::get)
}

// ============================================================================
// Test services (hand-written equivalents of generated `<Service>Server<T>`)
// ============================================================================

const FAIL_MESSAGE: &str = "denied: ünïcode 100% \n\t☃";

struct Echo;

impl GrpcService for Echo {
    const SERVICE_NAME: &'static str = "test.v1.Echo";

    fn is_client_streaming(&self, method_name: &str) -> bool {
        matches!(method_name, "Sum" | "Chat")
    }

    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        match method_name {
            "Echo" => Ok(write_grpc_frame_bytes(writer, false, req_payload)?),
            "Fail" => Err(GrpcStatus::permission_denied(FAIL_MESSAGE)),
            "Big" => {
                let len = u32::from_le_bytes(req_payload.try_into().unwrap()) as usize;
                let body: Vec<u8> = (0..len).map(|i| i as u8).collect();
                Ok(write_grpc_frame_bytes(writer, false, &body)?)
            }
            "Expand" => {
                // Server-streaming: req_payload[0] is count, req_payload[1..] is base payload
                let count = req_payload.first().copied().unwrap_or(0);
                if count == 0xFF {
                    return Err(GrpcStatus::invalid_argument("invalid expand count"));
                }
                let base = if req_payload.len() > 1 {
                    &req_payload[1..]
                } else {
                    &[]
                };
                for idx in 0..count {
                    let end_pos = writer.cursor();
                    writer.write_bytes(base)?;
                    writer.write_bytes(&[idx])?;
                    roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)?;
                    writer.rotate_written_to_end(end_pos);
                }
                Ok(())
            }
            _ => Err(GrpcStatus::unimplemented("unimplemented gRPC method")),
        }
    }

    fn call_stream(
        &self,
        method_name: &str,
        req_frames: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        let mut iter = roto_grpc::framing::GrpcFrameIter::new(req_frames, 4 * 1024 * 1024)
            .with_require_complete(true);
        match method_name {
            "Sum" => {
                let mut sum: u32 = 0;
                for frame_res in &mut iter {
                    let frame = frame_res?;
                    if frame.compressed {
                        return Err(GrpcStatus::internal("compressed frame"));
                    }
                    for &b in frame.payload {
                        if b == 0xFF {
                            return Err(GrpcStatus::invalid_argument("0xFF not allowed in Sum"));
                        }
                        sum = sum.wrapping_add(u32::from(b));
                    }
                }
                Ok(write_grpc_frame_bytes(writer, false, &sum.to_le_bytes())?)
            }
            "Chat" => {
                for frame_res in &mut iter {
                    let frame = frame_res?;
                    if frame.compressed {
                        return Err(GrpcStatus::internal("compressed frame"));
                    }
                    if frame.payload == b"abort" {
                        return Err(GrpcStatus::aborted("chat aborted"));
                    }
                    let end_pos = writer.cursor();
                    writer.write_bytes(frame.payload)?;
                    writer.write_bytes(b"pong:")?;
                    roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)?;
                    writer.rotate_written_to_end(end_pos);
                }
                Ok(())
            }
            _ => Err(GrpcStatus::unimplemented(
                "unimplemented gRPC stream method",
            )),
        }
    }
}

struct Reverse;

impl GrpcService for Reverse {
    const SERVICE_NAME: &'static str = "test.v1.Reverse";

    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        match method_name {
            "Reverse" => {
                let mut tmp = [0u8; 256];
                let out = &mut tmp[..req_payload.len()];
                out.copy_from_slice(req_payload);
                out.reverse();
                Ok(write_grpc_frame_bytes(writer, false, out)?)
            }
            _ => Err(GrpcStatus::unimplemented("unimplemented gRPC method")),
        }
    }
}

fn routes() -> [&'static dyn Route; 2] {
    [&Echo, &Reverse]
}

// ============================================================================
// Byte shuttling helpers
// ============================================================================

/// Shuttles bytes in both directions, delivering each direction in chunks of `chunk()` bytes,
/// until neither side has anything left to send.
fn shuttle<const M: usize, const R: usize>(
    client: &mut GrpcClient<'_, M, R>,
    server: &mut GrpcServer<'_, M, R>,
    mut chunk: impl FnMut() -> usize,
) {
    let mut wire = vec![0u8; 1 << 18];
    loop {
        let n = client.poll_transmit(&mut wire);
        let mut rest = &wire[..n];
        while !rest.is_empty() {
            let take = chunk().clamp(1, rest.len());
            server.recv(&rest[..take]).expect("server recv");
            rest = &rest[take..];
        }
        let m = server.poll_transmit(&mut wire);
        let mut rest = &wire[..m];
        while !rest.is_empty() {
            let take = chunk().clamp(1, rest.len());
            client.recv(&rest[..take]).expect("client recv");
            rest = &rest[take..];
        }
        if n == 0 && m == 0 {
            return;
        }
    }
}

fn whole() -> usize {
    usize::MAX
}

fn reply_of<const M: usize, const R: usize>(
    client: &mut GrpcClient<'_, M, R>,
    stream_id: u32,
) -> Result<Vec<u8>, (GrpcCode, String)> {
    let reply = client.take_reply(stream_id).expect("call finished");
    reply
        .result()
        .map(<[u8]>::to_vec)
        .map_err(|s| (s.code, s.message.to_owned()))
}

/// Runs `f` on a thread with a large stack (for connections with large reassembly buffers).
fn with_big_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

#[derive(Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

// ============================================================================
// Routing, status codes, and multiplexing
// ============================================================================

fn run_mixed_calls(chunk: impl FnMut() -> usize) {
    let routes = routes();
    let (mut msg, mut stx, mut ctx) = ([0u8; 1024], [0u8; 8192], [0u8; 8192]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
    let mut bufs = [[0u8; 256]; 6];
    let [b0, b1, b2, b3, b4, b5] = &mut bufs;

    let echo = client
        .start_call("/test.v1.Echo/Echo", b0, |w| {
            let _ = w.write_bytes(b"hello");
        })
        .unwrap();
    let rev = client
        .start_call("/test.v1.Reverse/Reverse", b1, |w| {
            let _ = w.write_bytes(b"abc");
        })
        .unwrap();
    let fail = client.start_call("/test.v1.Echo/Fail", b2, |_| {}).unwrap();
    let no_svc = client.start_call("/test.v1.Nope/Echo", b3, |_| {}).unwrap();
    let no_method = client.start_call("/test.v1.Echo/Nope", b4, |_| {}).unwrap();
    let bad_path = client.start_call("/no-method", b5, |_| {}).unwrap();
    assert_eq!(
        [echo, rev, fail, no_svc, no_method, bad_path],
        [1, 3, 5, 7, 9, 11]
    );

    shuttle(&mut client, &mut server, chunk);

    // Collect out of order.
    let mut unimplemented = |id| reply_of(&mut client, id).unwrap_err().0;
    assert_eq!(unimplemented(bad_path), GrpcCode::Unimplemented);
    assert_eq!(unimplemented(no_method), GrpcCode::Unimplemented);
    assert_eq!(unimplemented(no_svc), GrpcCode::Unimplemented);
    assert_eq!(
        reply_of(&mut client, fail),
        Err((GrpcCode::PermissionDenied, FAIL_MESSAGE.to_owned()))
    );
    assert_eq!(reply_of(&mut client, rev).unwrap(), b"cba");
    assert_eq!(reply_of(&mut client, echo).unwrap(), b"hello");
    assert!(client.take_reply(echo).is_none());
}

#[test]
fn routes_multiplexed_calls_across_services_with_status_codes() {
    run_mixed_calls(whole);
}

#[test]
fn survives_one_byte_chunked_delivery() {
    run_mixed_calls(|| 1);
}

#[test]
fn survives_random_small_chunked_delivery() {
    for seed in 0..32 {
        let mut rng = SplitMix64(seed);
        run_mixed_calls(move || 1 + (rng.next_u64() % 23) as usize);
    }
}

#[test]
fn reuses_stream_slots_across_many_sequential_calls() {
    let routes = routes();
    let (mut msg, mut stx, mut ctx) = ([0u8; 256], [0u8; 4096], [0u8; 4096]);
    let mut server: GrpcServer<2> = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient<2> = GrpcClient::new("https", "example.com", &mut ctx);
    let mut storage = [0u8; 64];
    let mut buf: &mut [u8] = &mut storage;
    for i in 0..100u8 {
        let id = client
            .start_call("/test.v1.Echo/Echo", buf, |w| {
                let _ = w.write_bytes(&[i; 3]);
            })
            .unwrap();
        shuttle(&mut client, &mut server, whole);
        let reply = client.take_reply(id).unwrap();
        assert_eq!(reply.result().unwrap(), [i; 3]);
        buf = reply.into_buf();
    }
}

#[test]
fn rejects_calls_beyond_slot_capacity_and_tiny_buffers() {
    let routes = routes();
    let (mut msg, mut stx, mut ctx) = ([0u8; 256], [0u8; 4096], [0u8; 4096]);
    let mut server: GrpcServer<2> = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient<2> = GrpcClient::new("http", "x", &mut ctx);
    let (mut a, mut b, mut c, mut tiny) = ([0u8; 32], [0u8; 32], [0u8; 32], [0u8; 4]);

    let err = client
        .start_call("/test.v1.Echo/Echo", &mut tiny, |w| {
            let _ = w.write_bytes(b"too long");
        })
        .unwrap_err();
    assert_eq!(err.result().unwrap_err().code, GrpcCode::ResourceExhausted);

    let id_a = client
        .start_call("/test.v1.Echo/Echo", &mut a, |_| {})
        .unwrap();
    let id_b = client
        .start_call("/test.v1.Echo/Echo", &mut b, |_| {})
        .unwrap();
    let err = client
        .start_call("/test.v1.Echo/Echo", &mut c, |_| {})
        .unwrap_err();
    assert_eq!(err.result().unwrap_err().code, GrpcCode::ResourceExhausted);
    shuttle(&mut client, &mut server, whole);
    assert_eq!(reply_of(&mut client, id_a).unwrap(), b"");
    assert_eq!(reply_of(&mut client, id_b).unwrap(), b"");
}

#[test]
fn reports_resource_exhausted_for_oversized_responses() {
    let routes = routes();
    let (mut msg, mut stx, mut ctx) = ([0u8; 64], [0u8; 4096], [0u8; 4096]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient = GrpcClient::new("http", "x", &mut ctx);
    let (mut a, mut b) = ([0u8; 256], [0u8; 16]);

    // Server-side message buffer overflow.
    let big = client
        .start_call("/test.v1.Echo/Big", &mut a, |w| {
            let _ = w.write_bytes(&100u32.to_le_bytes());
        })
        .unwrap();
    // Client-side call buffer overflow.
    let small = client
        .start_call("/test.v1.Echo/Big", &mut b, |w| {
            let _ = w.write_bytes(&40u32.to_le_bytes());
        })
        .unwrap();
    shuttle(&mut client, &mut server, whole);
    assert_eq!(
        reply_of(&mut client, big).unwrap_err().0,
        GrpcCode::ResourceExhausted
    );
    assert_eq!(
        reply_of(&mut client, small).unwrap_err().0,
        GrpcCode::ResourceExhausted
    );
}

#[test]
fn reports_resource_exhausted_when_server_transmit_buffer_is_too_small() {
    let routes = routes();
    let (mut msg, mut stx, mut ctx) = ([0u8; 512], [0u8; 256], [0u8; 4096]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient = GrpcClient::new("http", "x", &mut ctx);
    let mut buf = [0u8; 512];
    let id = client
        .start_call("/test.v1.Echo/Big", &mut buf, |w| {
            let _ = w.write_bytes(&300u32.to_le_bytes());
        })
        .unwrap();
    shuttle(&mut client, &mut server, whole);
    assert_eq!(
        reply_of(&mut client, id).unwrap_err().0,
        GrpcCode::ResourceExhausted
    );
}

// ============================================================================
// Multi-frame payloads and flow-control replenishment
// ============================================================================

#[test]
fn transfers_multi_frame_payloads_in_both_directions() {
    with_big_stack(|| {
        const CAP: usize = 64 * 1024;
        let routes = routes();
        let mut msg = vec![0u8; CAP];
        let mut stx = vec![0u8; 2 * CAP];
        let mut ctx = vec![0u8; 2 * CAP];
        let mut server: GrpcServer<4, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient<4, CAP> = GrpcClient::new("http", "x", &mut ctx);
        let mut storage = vec![0u8; CAP];
        let mut buf: &mut [u8] = &mut storage;
        let payload: Vec<u8> = (0..40_000u32).map(|i| (i * 7) as u8).collect();
        let expected: Vec<u8> = (0..50_000usize).map(|i| i as u8).collect();

        // Three rounds exceed the 65,535-byte initial windows, proving WINDOW_UPDATE replenishment.
        for round in 0..3 {
            let id = client
                .start_call("/test.v1.Echo/Echo", buf, |w| {
                    let _ = w.write_bytes(&payload);
                })
                .unwrap();
            let mut rng = SplitMix64(round);
            shuttle(&mut client, &mut server, || {
                1 + (rng.next_u64() % 4096) as usize
            });
            let reply = client.take_reply(id).unwrap();
            assert_eq!(reply.result().unwrap(), payload);

            let id = client
                .start_call("/test.v1.Echo/Big", reply.into_buf(), |w| {
                    let _ = w.write_bytes(&50_000u32.to_le_bytes());
                })
                .unwrap();
            shuttle(&mut client, &mut server, whole);
            let reply = client.take_reply(id).unwrap();
            assert_eq!(reply.result().unwrap(), expected);
            buf = reply.into_buf();
        }
    });
}

// ============================================================================
// Invalid request headers (raw HTTP/2 client)
// ============================================================================

/// Sends one request with the literal `headers` (and an optional LPM `body`) over a raw client
/// connection and returns the server's decoded `(grpc-status, grpc-message)`.
fn raw_call(headers: &[(&[u8], &[u8])], body: Option<&[u8]>) -> (GrpcCode, String) {
    let routes = routes();
    let (mut msg, mut stx) = ([0u8; 256], [0u8; 4096]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut raw: H2Connection = H2Connection::new(Role::Client);

    let mut block = [0u8; 1024];
    let mut len = 0;
    for (name, value) in headers {
        len += HpackEncoder::new()
            .encode_literal(name, value, false, &mut block[len..])
            .unwrap();
    }
    let mut wire = vec![0u8; 8192];
    let mut pos = raw.poll_transmit(&mut wire);
    let id = raw.open_stream().unwrap();
    pos += raw
        .send_headers_frame(id, &block[..len], body.is_none(), &mut wire[pos..])
        .unwrap();
    if let Some(body) = body {
        pos += raw
            .send_data_frame(id, body, true, &mut wire[pos..])
            .unwrap();
    }
    server.recv(&wire[..pos]).unwrap();

    let n = server.poll_transmit(&mut wire);
    let mut input = &wire[..n];
    while !input.is_empty() {
        let (used, event) = raw.recv(input).unwrap();
        input = &input[used..];
        if let Some(H2Event::Headers {
            header_block,
            end_stream: true,
            ..
        }) = event
        {
            let mut scratch = [0u8; 512];
            let mut decoder = HpackDecoder::<0>::stateless();
            let t = GrpcTrailers::decode(&mut decoder, header_block, &mut scratch).unwrap();
            if t.grpc_status != Some(GrpcCode::Ok) {
                assert_eq!(
                    t.status,
                    Some(&b"200"[..]),
                    "Trailers-Only carries :status 200"
                );
            }
            let mut out = [0u8; 256];
            let message = percent_decode(t.grpc_message.unwrap_or_default(), &mut out).unwrap();
            return (t.grpc_status.unwrap(), message.to_owned());
        }
    }
    panic!("no Trailers-Only response");
}

const LPM_EMPTY: &[u8] = &[0, 0, 0, 0, 0];

type Headers = Vec<(&'static [u8], &'static [u8])>;

fn request(method: &'static [u8], te: &'static [u8], ct: &'static [u8]) -> Headers {
    vec![
        (b":method".as_slice(), method),
        (b":scheme", b"http"),
        (b":path", b"/test.v1.Echo/Echo"),
        (b":authority", b"x"),
        (b"content-type", ct),
        (b"te", te),
    ]
}

#[test]
fn rejects_invalid_request_headers_with_trailers_only() {
    let ct = b"application/grpc+roto";
    let cases: [(Headers, GrpcCode); 5] = [
        (request(b"GET", b"trailers", ct), GrpcCode::InvalidArgument),
        (request(b"POST", b"gzip", ct), GrpcCode::InvalidArgument),
        (
            request(b"POST", b"trailers", b"application/json"),
            GrpcCode::InvalidArgument,
        ),
        (
            request(b"POST", b"trailers", b"application/grpcx"),
            GrpcCode::InvalidArgument,
        ),
        (
            request(b"POST", b"trailers", ct)[..5].to_vec(),
            GrpcCode::InvalidArgument,
        ),
    ];
    for (headers, code) in cases {
        assert_eq!(raw_call(&headers, Some(LPM_EMPTY)).0, code);
    }
    let no_path: Vec<_> = request(b"POST", b"trailers", ct)
        .into_iter()
        .filter(|(n, _)| *n != b":path")
        .collect();
    assert_eq!(
        raw_call(&no_path, Some(LPM_EMPTY)).0,
        GrpcCode::InvalidArgument
    );

    // Headers-only request (END_STREAM on HEADERS) has no unary message.
    assert_eq!(
        raw_call(&request(b"POST", b"trailers", ct), None).0,
        GrpcCode::Internal
    );
    // Compressed-flag messages are not supported (RFC gRPC -> Internal).
    let compressed = [1, 0, 0, 0, 0];
    assert_eq!(
        raw_call(&request(b"POST", b"trailers", ct), Some(&compressed)).0,
        GrpcCode::Internal
    );
    // Plain `application/grpc` is accepted.
    let echo = raw_call(
        &request(b"POST", b"trailers", b"application/grpc"),
        Some(LPM_EMPTY),
    );
    assert_eq!(echo.0, GrpcCode::Ok);
}

#[test]
fn server_edge_cases_and_stream_error_recovery() {
    let ct = b"application/grpc+roto";

    // 1. :path exceeding MAX_PATH_LEN (256) -> Unimplemented
    let long_path = vec![b'a'; roto_grpc::server::MAX_PATH_LEN + 1];
    let mut hdrs_long = request(b"POST", b"trailers", ct);
    hdrs_long[2] = (b":path", &long_path);
    assert_eq!(
        raw_call(&hdrs_long, Some(LPM_EMPTY)).0,
        GrpcCode::Unimplemented
    );

    // 2. Non-UTF-8 :path -> InvalidArgument
    let mut hdrs_bad_utf8 = request(b"POST", b"trailers", ct);
    hdrs_bad_utf8[2] = (b":path", b"/test.v1.Echo/\xFF\xFE");
    assert_eq!(
        raw_call(&hdrs_bad_utf8, Some(LPM_EMPTY)).0,
        GrpcCode::InvalidArgument
    );

    // 3. Server handles duplicate HEADERS before message (Internal), empty DATA with END_STREAM
    // (Internal), client RST_STREAM freeing the call slot, and extra trailers/DATA on an answered call.
    let routes = routes();
    let (mut msg, mut stx) = ([0u8; 256], [0u8; 4096]);
    let mut server: GrpcServer<2, 64> = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut raw: H2Connection<4, 256> = H2Connection::new(Role::Client);
    let mut wire = [0u8; 4096];

    // Complete handshake
    let n = raw.poll_transmit(&mut wire);
    server.recv(&wire[..n]).unwrap();
    let m = server.poll_transmit(&mut wire);
    let mut pos = 0;
    while pos < m {
        let (c, _) = raw.recv(&wire[pos..m]).unwrap();
        pos += c;
    }
    let n = raw.poll_transmit(&mut wire);
    server.recv(&wire[..n]).unwrap();
    let _ = server.poll_transmit(&mut wire);

    // 3a. Stream 1: send request HEADERS (no END_STREAM), then duplicate HEADERS (END_STREAM) -> Internal
    let (s1, n) = raw
        .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
        .unwrap();
    assert_eq!(s1, 1);
    server.recv(&wire[..n]).unwrap();
    let mut tr_block = [0u8; 64];
    let tr_len = HpackEncoder::new()
        .encode_literal(b"x-trailer", b"1", false, &mut tr_block)
        .unwrap();
    let n = raw
        .send_headers_frame(s1, &tr_block[..tr_len], true, &mut wire)
        .unwrap();
    server.recv(&wire[..n]).unwrap();
    let m = server.poll_transmit(&mut wire);
    let (_, ev) = raw.recv(&wire[..m]).unwrap();
    if let Some(H2Event::Headers { header_block, .. }) = ev {
        let mut scratch = [0u8; 128];
        let mut dec = HpackDecoder::<0>::stateless();
        let t = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
        assert_eq!(t.grpc_status, Some(GrpcCode::Internal));
    } else {
        panic!("expected Trailers-Only Internal on duplicate HEADERS, got {ev:?}");
    }

    // 3b. Stream 3: send request HEADERS (no END_STREAM), then empty DATA with END_STREAM -> Internal
    let (s3, n) = raw
        .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
        .unwrap();
    assert_eq!(s3, 3);
    server.recv(&wire[..n]).unwrap();
    let n = raw.send_data_frame(s3, &[], true, &mut wire).unwrap();
    server.recv(&wire[..n]).unwrap();
    let m = server.poll_transmit(&mut wire);
    let (_, ev) = raw.recv(&wire[..m]).unwrap();
    if let Some(H2Event::Headers { header_block, .. }) = ev {
        let mut scratch = [0u8; 128];
        let mut dec = HpackDecoder::<0>::stateless();
        let t = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
        assert_eq!(t.grpc_status, Some(GrpcCode::Internal));
    } else {
        panic!("expected Trailers-Only Internal on empty EndStream, got {ev:?}");
    }

    // 3c. Stream 5 & 7: open both slots (MAX_STREAMS = 2), then cancel both via RST_STREAM and StreamError
    // to verify PendingCall slots are freed for stream 9.
    let (s5, n) = raw
        .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
        .unwrap();
    assert_eq!(s5, 5);
    server.recv(&wire[..n]).unwrap();
    raw.queue_rst_stream(s5, roto_grpc::h2::H2ErrorCode::Cancel)
        .unwrap();
    let n = raw.poll_transmit(&mut wire);
    server.recv(&wire[..n]).unwrap();

    let (s7, n) = raw
        .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
        .unwrap();
    assert_eq!(s7, 7);
    server.recv(&wire[..n]).unwrap();
    // Send oversized LPM header (declared length 100 > REASM_CAP 64) on stream 7 -> StreamError
    let mut bad_lpm = [0u8; 5];
    bad_lpm[1..5].copy_from_slice(&100u32.to_be_bytes());
    let n = raw.send_data_frame(s7, &bad_lpm, true, &mut wire).unwrap();
    server.recv(&wire[..n]).unwrap();
    let m = server.poll_transmit(&mut wire);
    let _ = raw.recv(&wire[..m]).unwrap();

    // Stream 9 now succeeds and extra message + trailers after response are ignored cleanly.
    let (s9, n) = raw
        .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
        .unwrap();
    assert_eq!(s9, 9);
    server.recv(&wire[..n]).unwrap();
    let n = raw
        .send_data_frame(s9, LPM_EMPTY, false, &mut wire)
        .unwrap();
    server.recv(&wire[..n]).unwrap();
    // Send extra message and trailers on already-answered stream 9
    let n = raw
        .send_data_frame(s9, LPM_EMPTY, false, &mut wire)
        .unwrap();
    server.recv(&wire[..n]).unwrap();
    let n = raw
        .send_headers_frame(s9, &tr_block[..tr_len], true, &mut wire)
        .unwrap();
    server.recv(&wire[..n]).unwrap();
    let m = server.poll_transmit(&mut wire);
    assert!(m > 0);
}

#[test]
fn server_max_streams_refusal_reject_fallback_wire_percent_encoding_and_pipelined_burst() {
    let ct = b"application/grpc+roto";
    let routes = routes();

    // 1 (Item 5): MAX_STREAMS + 1 concurrent streams on GrpcServer<2, 64>:
    // Open s1 and s3 without closing either, then open s5 (3rd concurrent stream).
    // Server must refuse s5 with RST_STREAM(RefusedStream) while s1 and s3 complete Ok,
    // and subsequent stream s7 succeeds once slots are freed.
    {
        let (mut msg, mut stx) = ([0u8; 256], [0u8; 4096]);
        let mut server: GrpcServer<2, 64> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut raw: H2Connection<4, 256> = H2Connection::new(Role::Client);
        let mut wire = [0u8; 4096];

        let n = raw.poll_transmit(&mut wire);
        server.recv(&wire[..n]).unwrap();
        let m = server.poll_transmit(&mut wire);
        let mut pos = 0;
        while pos < m {
            let (c, _) = raw.recv(&wire[pos..m]).unwrap();
            pos += c;
        }
        let n = raw.poll_transmit(&mut wire);
        server.recv(&wire[..n]).unwrap();
        let _ = server.poll_transmit(&mut wire);

        let (s1, n1) = raw
            .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
            .unwrap();
        server.recv(&wire[..n1]).unwrap();
        let (s3, n3) = raw
            .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
            .unwrap();
        server.recv(&wire[..n3]).unwrap();

        // 3rd concurrent stream s5 exceeds server's MAX_STREAMS = 2 (craft raw HEADERS frame
        // bypassing client-side peer_settings.max_concurrent_streams check).
        assert_eq!([s1, s3], [1, 3]);
        let mut hpack = [0u8; 128];
        let hlen = HpackEncoder::with_huffman(true)
            .encode_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, &mut hpack)
            .unwrap();
        let mut s5_hdr = Vec::with_capacity(FRAME_HEADER_LEN + hlen);
        s5_hdr.extend_from_slice(
            &FrameHeader::new(hlen as u32, FrameType::HEADERS, 0x04, 5).encode(),
        );
        s5_hdr.extend_from_slice(&hpack[..hlen]);
        server.recv(&s5_hdr).unwrap();
        let m = server.poll_transmit(&mut wire);
        assert_eq!(m, FRAME_HEADER_LEN + 4);
        let rst_hdr = FrameHeader::decode_slice(&wire[..m]).unwrap();
        assert_eq!(rst_hdr.frame_type, FrameType::RST_STREAM);
        assert_eq!(rst_hdr.stream_id, 5);
        assert_eq!(
            u32::from_be_bytes(wire[FRAME_HEADER_LEN..m].try_into().unwrap()),
            H2ErrorCode::RefusedStream as u32
        );

        // Now send request bodies for s1 and s3; both complete with Ok
        let n = raw.send_data_frame(s1, LPM_EMPTY, true, &mut wire).unwrap();
        server.recv(&wire[..n]).unwrap();
        let n = raw.send_data_frame(s3, LPM_EMPTY, true, &mut wire).unwrap();
        server.recv(&wire[..n]).unwrap();
        let m = server.poll_transmit(&mut wire);
        let mut pos = 0;
        let mut completed_streams = Vec::new();
        while pos < m {
            let (c, ev) = raw.recv(&wire[pos..m]).unwrap();
            pos += c;
            if let Some(H2Event::Headers {
                stream_id,
                end_stream: true,
                header_block,
            }) = ev
            {
                let mut scratch = [0u8; 128];
                let mut dec = HpackDecoder::<0>::stateless();
                let t = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
                assert_eq!(t.grpc_status, Some(GrpcCode::Ok));
                completed_streams.push(stream_id);
            }
        }
        assert_eq!(completed_streams, vec![1, 3]);

        // Subsequent stream s7 succeeds now that slots are freed (advance raw past stream 5 first)
        assert_eq!(raw.open_stream().unwrap(), 5);
        let (s7, n7) = raw
            .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, false, &mut wire)
            .unwrap();
        assert_eq!(s7, 7);
        server.recv(&wire[..n7]).unwrap();
        let n = raw.send_data_frame(s7, LPM_EMPTY, true, &mut wire).unwrap();
        server.recv(&wire[..n]).unwrap();
        let m = server.poll_transmit(&mut wire);
        assert!(m > 0);
    }

    // 2 (Item 6a): reject -> RST_STREAM(InternalError) fallback when tx_buf cannot fit Trailers-Only
    {
        // 36 bytes fits the initial server SETTINGS (33 bytes) and SETTINGS ACK (9 bytes),
        // but once drained, cannot fit a ~55-byte Trailers-Only error response; fallback queues
        // a 13-byte RST_STREAM(InternalError).
        let (mut msg, mut tiny_stx) = ([0u8; 64], [0u8; 36]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut tiny_stx);
        let mut raw: H2Connection = H2Connection::new(Role::Client);
        let mut wire = [0u8; 512];

        let n = raw.poll_transmit(&mut wire);
        server.recv(&wire[..n]).unwrap();
        let m = server.poll_transmit(&mut wire);
        let mut pos = 0;
        while pos < m {
            let (c, _) = raw.recv(&wire[pos..m]).unwrap();
            pos += c;
        }
        let n = raw.poll_transmit(&mut wire);
        server.recv(&wire[..n]).unwrap();
        let _ = server.poll_transmit(&mut wire);

        // Send headers-only request (END_STREAM = true -> missing request message -> reject)
        let (sid, n) = raw
            .send_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, true, &mut wire)
            .unwrap();
        assert_eq!(sid, 1);
        server.recv(&wire[..n]).unwrap();
        let m = server.poll_transmit(&mut wire);
        assert_eq!(m, FRAME_HEADER_LEN + 4);
        let hdr = FrameHeader::decode_slice(&wire[..m]).unwrap();
        assert_eq!(hdr.frame_type, FrameType::RST_STREAM);
        assert_eq!(hdr.stream_id, 1);
        assert_eq!(
            u32::from_be_bytes(wire[FRAME_HEADER_LEN..m].try_into().unwrap()),
            H2ErrorCode::InternalError as u32
        );
    }

    // 3 (Item 6b): Raw wire percent-encoding check on /test.v1.Echo/Fail
    {
        let (mut msg, mut stx) = ([0u8; 256], [0u8; 4096]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut raw: H2Connection = H2Connection::new(Role::Client);
        let mut wire = [0u8; 4096];

        let mut pos = raw.poll_transmit(&mut wire);
        let (sid, n) = raw
            .send_grpc_request_headers(
                b"http",
                b"x",
                b"/test.v1.Echo/Fail",
                ct,
                false,
                &mut wire[pos..],
            )
            .unwrap();
        pos += n;
        pos += raw
            .send_data_frame(sid, LPM_EMPTY, true, &mut wire[pos..])
            .unwrap();
        server.recv(&wire[..pos]).unwrap();

        let m = server.poll_transmit(&mut wire);
        let mut input = &wire[..m];
        let mut found_raw_msg = false;
        while !input.is_empty() {
            let (used, ev) = raw.recv(input).unwrap();
            input = &input[used..];
            if let Some(H2Event::Headers {
                header_block,
                end_stream: true,
                ..
            }) = ev
            {
                let mut scratch = [0u8; 512];
                let mut dec = HpackDecoder::<0>::stateless();
                let t = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
                assert_eq!(t.grpc_status, Some(GrpcCode::PermissionDenied));
                assert_eq!(
                    t.grpc_message,
                    Some(&b"denied: %C3%BCn%C3%AFcode 100%25 %0A%09%E2%98%83"[..])
                );
                found_raw_msg = true;
            }
        }
        assert!(found_raw_msg);
    }

    // 4 (Item 6c): 40 back-to-back unary calls fed into a single server.recv(&wire) call
    // (exceeding TX_QUEUE_CAP / 26 bytes per pair of WINDOW_UPDATEs without overflowing conn.tx_queue)
    {
        let (mut msg, mut stx) = ([0u8; 256], vec![0u8; 32_768]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut raw: H2Connection = H2Connection::new(Role::Client);
        let mut burst = Vec::new();
        let mut tmp = [0u8; 256];
        let n = raw.poll_transmit(&mut tmp);
        burst.extend_from_slice(&tmp[..n]);

        for i in 0..40u32 {
            let sid = 1 + i * 2;
            let mut hpack = [0u8; 128];
            let hlen = HpackEncoder::with_huffman(true)
                .encode_grpc_request_headers(b"http", b"x", b"/test.v1.Echo/Echo", ct, &mut hpack)
                .unwrap();
            burst.extend_from_slice(
                &FrameHeader::new(hlen as u32, FrameType::HEADERS, 0x04, sid).encode(),
            );
            burst.extend_from_slice(&hpack[..hlen]);
            burst.extend_from_slice(
                &FrameHeader::new(LPM_EMPTY.len() as u32, FrameType::DATA, 0x01, sid).encode(),
            );
            burst.extend_from_slice(LPM_EMPTY);
        }

        // Single recv call processes all 40 back-to-back calls cleanly
        server.recv(&burst).unwrap();
        let mut out = vec![0u8; 32_768];
        let written = server.poll_transmit(&mut out);
        assert!(written > 40 * 20);
    }
}

// ============================================================================
// Client-side error paths (driven against a raw HTTP/2 server connection)
// ============================================================================

fn encode_header_block(headers: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut block = [0u8; 512];
    let mut len = 0;
    for (name, value) in headers {
        len += HpackEncoder::new()
            .encode_literal(name, value, false, &mut block[len..])
            .unwrap();
    }
    block[..len].to_vec()
}

/// Runs a single unary call on `GrpcClient` and lets `respond` craft the raw server frames.
fn client_call_with_raw_server(
    respond: impl FnOnce(&mut H2Connection, u32, &mut [u8]) -> usize,
) -> Result<Vec<u8>, (GrpcCode, String)> {
    let (mut ctx, mut buf, mut wire) = ([0u8; 4096], [0u8; 256], [0u8; 4096]);
    let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
    let mut raw_srv: H2Connection = H2Connection::new(Role::Server);

    let sid = client
        .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
            let _ = w.write_bytes(b"req");
        })
        .unwrap();

    // Deliver client preface + SETTINGS + request to raw server.
    let n = client.poll_transmit(&mut wire);
    let mut pos = 0;
    while pos < n {
        let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
        pos += c;
    }
    let m = raw_srv.poll_transmit(&mut wire);
    client.recv(&wire[..m]).unwrap();
    let _ = client.poll_transmit(&mut wire);

    let resp_len = respond(&mut raw_srv, sid, &mut wire);
    client.recv(&wire[..resp_len]).unwrap();
    reply_of(&mut client, sid)
}

#[test]
fn client_error_paths_and_status_mappings() {
    let ct = b"application/grpc+roto".as_slice();

    // 1. Non-200 HTTP :status mappings per gRPC HTTP-to-gRPC specification
    let http_cases: [(&[u8], GrpcCode); 9] = [
        (b"400", GrpcCode::Internal),
        (b"401", GrpcCode::Unauthenticated),
        (b"403", GrpcCode::PermissionDenied),
        (b"404", GrpcCode::Unimplemented),
        (b"429", GrpcCode::Unavailable),
        (b"502", GrpcCode::Unavailable),
        (b"503", GrpcCode::Unavailable),
        (b"504", GrpcCode::Unavailable),
        (b"500", GrpcCode::Unknown),
    ];
    for (http_status, expected_code) in http_cases {
        let block = encode_header_block(&[(b":status", http_status), (b"content-type", ct)]);
        let err = client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &block, false, out).unwrap()
        })
        .unwrap_err();
        assert_eq!(err.0, expected_code);
    }

    // 2. Invalid response content-type -> Unknown
    let bad_ct = encode_header_block(&[(b":status", b"200"), (b"content-type", b"text/plain")]);
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &bad_ct, true, out).unwrap()
        })
        .unwrap_err()
        .0,
        GrpcCode::Unknown
    );

    // 3. Malformed HPACK in response headers -> Internal
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &[0xFF, 0xFF, 0xFF], true, out)
                .unwrap()
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 4. Missing grpc-status in trailers -> Internal
    let no_status = encode_header_block(&[(b":status", b"200"), (b"content-type", ct)]);
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &no_status, true, out).unwrap()
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 5. Unrecognised numeric grpc-status (e.g. 99) -> Unknown
    let unknown_status = encode_header_block(&[
        (b":status", b"200"),
        (b"content-type", ct),
        (b"grpc-status", b"99"),
    ]);
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &unknown_status, true, out)
                .unwrap()
        })
        .unwrap_err()
        .0,
        GrpcCode::Unknown
    );

    // 6. grpc-status: 0 with no response message -> Internal
    let ok_no_msg = encode_header_block(&[
        (b":status", b"200"),
        (b"content-type", ct),
        (b"grpc-status", b"0"),
    ]);
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &ok_no_msg, true, out).unwrap()
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 7. Multiple response messages on unary call -> Internal
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            let mut n = srv.send_grpc_response_headers(sid, ct, out).unwrap();
            n += srv
                .send_data_frame(sid, LPM_EMPTY, false, &mut out[n..])
                .unwrap();
            n += srv
                .send_data_frame(sid, LPM_EMPTY, false, &mut out[n..])
                .unwrap();
            n
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 8. Compressed response message -> Internal
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            let mut n = srv.send_grpc_response_headers(sid, ct, out).unwrap();
            n += srv
                .send_data_frame(sid, &[1, 0, 0, 0, 0], false, &mut out[n..])
                .unwrap();
            n
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 9. END_STREAM on response DATA with message (no trailers) -> Internal
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            let mut n = srv.send_grpc_response_headers(sid, ct, out).unwrap();
            n += srv
                .send_data_frame(sid, LPM_EMPTY, true, &mut out[n..])
                .unwrap();
            n
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 10. Empty DATA with END_STREAM after response HEADERS -> Internal
    assert_eq!(
        client_call_with_raw_server(|srv, sid, out| {
            let mut n = srv.send_grpc_response_headers(sid, ct, out).unwrap();
            n += srv.send_data_frame(sid, &[], true, &mut out[n..]).unwrap();
            n
        })
        .unwrap_err()
        .0,
        GrpcCode::Internal
    );

    // 11. Inbound RST_STREAM error code mappings
    for (h2_code, grpc_code) in [
        (H2ErrorCode::Cancel, GrpcCode::Cancelled),
        (H2ErrorCode::RefusedStream, GrpcCode::Unavailable),
        (H2ErrorCode::EnhanceYourCalm, GrpcCode::ResourceExhausted),
    ] {
        assert_eq!(
            client_call_with_raw_server(|srv, sid, out| {
                srv.queue_rst_stream(sid, h2_code).unwrap();
                srv.poll_transmit(out)
            })
            .unwrap_err()
            .0,
            grpc_code
        );
    }

    // 12 (Item 3a): Invalid percent-encoding / invalid UTF-8 in grpc-message falls back to ""
    for bad_msg in [&b"%FF"[..], &b"bad%2"[..]] {
        let block = encode_header_block(&[
            (b":status", b"200"),
            (b"content-type", ct),
            (b"grpc-status", b"7"),
            (b"grpc-message", bad_msg),
        ]);
        let (code, msg) = client_call_with_raw_server(|srv, sid, out| {
            srv.send_headers_frame(sid, &block, true, out).unwrap()
        })
        .unwrap_err();
        assert_eq!(code, GrpcCode::PermissionDenied);
        assert_eq!(msg, "");
    }
}

#[test]
fn client_concurrent_rst_cancel_slot_recovery_utf8_truncation_and_unknown_stream() {
    let ct = b"application/grpc+roto".as_slice();

    // 1 (Item 1 & Item 4): RST_STREAM on one of two concurrent calls (s1 reset, s3 succeeds),
    // and events on an already-taken stream ID are ignored cleanly.
    {
        let (mut ctx, mut wire) = ([0u8; 4096], [0u8; 4096]);
        let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
        let mut raw_srv: H2Connection = H2Connection::new(Role::Server);
        let (mut b1, mut b3) = ([0u8; 128], [0u8; 128]);

        let s1 = client
            .start_call("/test.v1.Echo/Echo", &mut b1, |_| {})
            .unwrap();
        let s3 = client
            .start_call("/test.v1.Echo/Echo", &mut b3, |_| {})
            .unwrap();
        assert_eq!([s1, s3], [1, 3]);

        let n = client.poll_transmit(&mut wire);
        let mut pos = 0;
        while pos < n {
            let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
            pos += c;
        }
        let m = raw_srv.poll_transmit(&mut wire);
        client.recv(&wire[..m]).unwrap();
        let _ = client.poll_transmit(&mut wire);

        // Server resets s1 with RefusedStream -> s1 finishes with Unavailable; take_reply(s1) removes it.
        raw_srv
            .queue_rst_stream(s1, H2ErrorCode::RefusedStream)
            .unwrap();
        let m = raw_srv.poll_transmit(&mut wire);
        client.recv(&wire[..m]).unwrap();
        assert_eq!(
            reply_of(&mut client, s1).unwrap_err().0,
            GrpcCode::Unavailable
        );

        // Feed another RST_STREAM on already-taken stream s1 (Item 4) followed by valid response on s3.
        let mut extra = Vec::new();
        extra.extend_from_slice(&FrameHeader::new(4, FrameType::RST_STREAM, 0, s1).encode());
        extra.extend_from_slice(&(H2ErrorCode::Cancel as u32).to_be_bytes());

        let mut m = raw_srv
            .send_grpc_response_headers(s3, ct, &mut wire)
            .unwrap();
        m += raw_srv
            .send_data_frame(s3, LPM_EMPTY, false, &mut wire[m..])
            .unwrap();
        m += raw_srv
            .send_grpc_trailers(s3, GrpcStatus::ok(), &mut wire[m..])
            .unwrap();
        extra.extend_from_slice(&wire[..m]);

        client.recv(&extra).unwrap();
        assert_eq!(reply_of(&mut client, s3).unwrap(), b"");
    }

    // 2 (Item 2): GrpcClient<1> (single stream slot) emits 13-byte RST_STREAM(Cancel) on early
    // response failure (end_stream = false) for both headers and data, freeing the single slot
    // so a subsequent start_call succeeds.
    {
        let (mut ctx, mut wire) = ([0u8; 4096], [0u8; 4096]);
        let mut client: GrpcClient<1> = GrpcClient::new("http", "localhost", &mut ctx);
        let mut raw_srv: H2Connection<2> = H2Connection::new(Role::Server);
        let mut buf = [0u8; 128];

        let s1 = client
            .start_call("/test.v1.Echo/Echo", &mut buf, |_| {})
            .unwrap();
        assert_eq!(s1, 1);
        let n = client.poll_transmit(&mut wire);
        let mut pos = 0;
        while pos < n {
            let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
            pos += c;
        }
        let m = raw_srv.poll_transmit(&mut wire);
        client.recv(&wire[..m]).unwrap();
        let _ = client.poll_transmit(&mut wire);

        // Server sends :status 503 with end_stream = false -> client finishes call and queues RST_STREAM(Cancel)
        let hdr_503 = encode_header_block(&[(b":status", b"503"), (b"content-type", ct)]);
        let m = raw_srv
            .send_headers_frame(s1, &hdr_503, false, &mut wire)
            .unwrap();
        client.recv(&wire[..m]).unwrap();

        let rst_len = client.poll_transmit(&mut wire);
        assert_eq!(rst_len, FRAME_HEADER_LEN + 4);
        let rst_hdr = FrameHeader::decode_slice(&wire[..rst_len]).unwrap();
        assert_eq!(rst_hdr.frame_type, FrameType::RST_STREAM);
        assert_eq!(rst_hdr.stream_id, 1);
        assert_eq!(
            u32::from_be_bytes(wire[FRAME_HEADER_LEN..rst_len].try_into().unwrap()),
            H2ErrorCode::Cancel as u32
        );
        raw_srv.recv(&wire[..rst_len]).unwrap();

        let reply1 = client.take_reply(s1).unwrap();
        assert_eq!(reply1.result().unwrap_err().code, GrpcCode::Unavailable);
        let buf = reply1.into_buf();

        // Subsequent start_call on GrpcClient<1> succeeds on stream 3; now test early DATA failure
        let s3 = client
            .start_call("/test.v1.Echo/Echo", buf, |_| {})
            .unwrap();
        assert_eq!(s3, 3);
        let n = client.poll_transmit(&mut wire);
        let mut pos = 0;
        while pos < n {
            let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
            pos += c;
        }
        // Server sends 200 OK headers + compressed DATA with end_stream = false
        let mut m = raw_srv
            .send_grpc_response_headers(s3, ct, &mut wire)
            .unwrap();
        m += raw_srv
            .send_data_frame(s3, &[1, 0, 0, 0, 0], false, &mut wire[m..])
            .unwrap();
        client.recv(&wire[..m]).unwrap();

        // poll_transmit emits 2 auto WINDOW_UPDATE frames (26 bytes) + 13-byte RST_STREAM(Cancel) = 39 bytes
        let tx_len = client.poll_transmit(&mut wire);
        assert_eq!(tx_len, (FRAME_HEADER_LEN + 4) * 3);
        let rst_slice = &wire[tx_len - (FRAME_HEADER_LEN + 4)..tx_len];
        let rst_hdr = FrameHeader::decode_slice(rst_slice).unwrap();
        assert_eq!(rst_hdr.frame_type, FrameType::RST_STREAM);
        assert_eq!(rst_hdr.stream_id, 3);
        assert_eq!(
            u32::from_be_bytes(rst_slice[FRAME_HEADER_LEN..].try_into().unwrap()),
            H2ErrorCode::Cancel as u32
        );
        let reply3 = client.take_reply(s3).unwrap();
        assert_eq!(reply3.result().unwrap_err().code, GrpcCode::Internal);
        let buf = reply3.into_buf();

        // Slot is free again for stream 5
        let s5 = client
            .start_call("/test.v1.Echo/Echo", buf, |_| {})
            .unwrap();
        assert_eq!(s5, 5);
    }

    // 3 (Item 3b): write_truncated UTF-8 codepoint boundary truncation on tiny call buffers:
    // FAIL_MESSAGE = "denied: ünïcode 100% \n\t☃"
    // - 'ü' occupies bytes 8..10, so a 9-byte buffer must truncate to 8 bytes ("denied: ")
    // - '☃' occupies bytes 24..27, so a 26-byte buffer must truncate to 24 bytes ("denied: ünïcode 100% \n\t")
    {
        let routes = routes();
        let (mut msg, mut stx, mut ctx) = ([0u8; 256], [0u8; 4096], [0u8; 4096]);
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
        let mut buf9 = [0u8; 9];
        let mut buf26 = [0u8; 26];

        let s1 = client
            .start_call("/test.v1.Echo/Fail", &mut buf9, |_| {})
            .unwrap();
        let s3 = client
            .start_call("/test.v1.Echo/Fail", &mut buf26, |_| {})
            .unwrap();
        shuttle(&mut client, &mut server, whole);

        assert_eq!(
            reply_of(&mut client, s1),
            Err((GrpcCode::PermissionDenied, "denied: ".to_owned()))
        );
        assert_eq!(
            reply_of(&mut client, s3),
            Err((
                GrpcCode::PermissionDenied,
                "denied: ünïcode 100% \n\t".to_owned()
            ))
        );
    }
}

#[test]
fn client_goaway_fatal_recv_and_utf8_truncation_recover_buffers() {
    let ct = b"application/grpc+roto".as_slice();
    let (mut ctx, mut wire) = ([0u8; 4096], [0u8; 4096]);
    let mut client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
    let mut raw_srv: H2Connection = H2Connection::new(Role::Server);
    let (mut b1, mut b3, mut b5) = ([0u8; 128], [0u8; 128], [0u8; 128]);

    let s1 = client
        .start_call("/test.v1.Echo/Echo", &mut b1, |_| {})
        .unwrap();
    let s3 = client
        .start_call("/test.v1.Echo/Echo", &mut b3, |_| {})
        .unwrap();
    let n = client.poll_transmit(&mut wire);
    let mut pos = 0;
    while pos < n {
        let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
        pos += c;
    }
    let m = raw_srv.poll_transmit(&mut wire);
    client.recv(&wire[..m]).unwrap();

    // Server sends GOAWAY(last_stream_id = 1): stream 3 (> 1) finishes with Unavailable,
    // while stream 1 completes normally. Also new start_call after GOAWAY returns Unavailable.
    raw_srv
        .queue_goaway(1, H2ErrorCode::NoError, b"draining")
        .unwrap();
    let mut m = raw_srv.poll_transmit(&mut wire);
    m += raw_srv
        .send_grpc_response_headers(s1, ct, &mut wire[m..])
        .unwrap();
    m += raw_srv
        .send_data_frame(s1, LPM_EMPTY, false, &mut wire[m..])
        .unwrap();
    m += raw_srv
        .send_grpc_trailers(s1, GrpcStatus::ok(), &mut wire[m..])
        .unwrap();
    client.recv(&wire[..m]).unwrap();

    assert_eq!(
        reply_of(&mut client, s3).unwrap_err().0,
        GrpcCode::Unavailable
    );
    assert_eq!(reply_of(&mut client, s1).unwrap(), b"");

    let after_goaway = client
        .start_call("/test.v1.Echo/Echo", &mut b5, |_| {})
        .unwrap_err();
    assert_eq!(
        after_goaway.result().unwrap_err().code,
        GrpcCode::Unavailable
    );
    let b5 = after_goaway.into_buf();

    // start_call with tiny tx_buf -> ResourceExhausted
    let mut tiny_tx = [0u8; 16];
    let mut client_tiny: GrpcClient = GrpcClient::new("http", "localhost", &mut tiny_tx);
    let err = client_tiny
        .start_call("/test.v1.Echo/Echo", b5, |_| {})
        .unwrap_err();
    assert_eq!(err.result().unwrap_err().code, GrpcCode::ResourceExhausted);
    let b5 = err.into_buf();

    // Fatal connection error during client.recv finishes all in-flight calls with Unavailable
    // so caller buffers can be recovered via take_reply.
    let mut ctx2 = [0u8; 4096];
    let mut client2: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx2);
    let sid = client2
        .start_call("/test.v1.Echo/Echo", b5, |_| {})
        .unwrap();
    // Feed invalid frame (PING with length 3 on stream 0 -> FrameSizeError) after server SETTINGS
    let bad_wire = [
        0, 0, 0, 4, 0, 0, 0, 0, 0, // valid empty SETTINGS
        0, 0, 3, 6, 0, 0, 0, 0, 0, 1, 2, 3, // invalid 3-byte PING
    ];
    assert!(client2.recv(&bad_wire).is_err());
    let recovered = client2
        .take_reply(sid)
        .expect("in-flight call finished on fatal error");
    assert_eq!(recovered.result().unwrap_err().code, GrpcCode::Unavailable);
    let _b5 = recovered.into_buf();

    // Verify H2LoopbackChannel recovers its buffer if pump encounters an error (stalled/corrupted server)
    // and can still use its buffer afterward.
    let routes = routes();
    let (mut msg, mut stx, mut ctx3, mut call_buf) =
        ([0u8; 128], [0u8; 1024], [0u8; 1024], [0u8; 128]);
    let server3: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let client3: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx3);
    let mut channel = H2LoopbackChannel::new(client3, server3, &mut call_buf);
    // Corrupt server state by feeding invalid preface directly via server_mut()
    assert!(channel
        .server_mut()
        .recv(b"INVALID_PREFACE________________________")
        .is_err());
    // Now call_unary will fail during pump, and must recover call_buf rather than leaving it empty.
    let err = channel
        .call_unary("/test.v1.Echo/Echo", |w| {
            let _ = w.write_bytes(b"hello");
        })
        .unwrap_err();
    assert_eq!(err.code, GrpcCode::Unavailable);
    // Verify call_buf was restored (a subsequent call with a 32-byte payload does not fail with "request exceeds call buffer").
    let err2 = channel
        .call_unary("/test.v1.Echo/Echo", |w| {
            let _ = w.write_bytes(&[0xAB; 32]);
        })
        .unwrap_err();
    assert_ne!(err2.message, "request exceeds call buffer");
    let _ = format!(
        "{:?} {:?} {:?}",
        channel,
        channel.client(),
        channel.server()
    );
}

// ============================================================================
// `UnaryChannel` adapter and zero-allocation proof
// ============================================================================

#[test]
fn h2_loopback_channel_round_trips_with_zero_heap_allocations() {
    let routes = routes();
    let (mut msg, mut stx, mut ctx, mut call) = ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
    let probe = allocs();
    drop(std::hint::black_box(vec![0u8; 1]));
    assert!(allocs() > probe, "allocation counter must observe heap use");

    let before = allocs();
    let server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
    let mut channel = H2LoopbackChannel::new(client, server, &mut call);

    for i in 0..10u8 {
        let resp = channel
            .call_unary("/test.v1.Reverse/Reverse", |w| {
                let _ = w.write_bytes(&[i, i + 1, i + 2]);
            })
            .unwrap();
        assert_eq!(resp, [i + 2, i + 1, i]);
    }
    let err = channel
        .call_unary("/test.v1.Echo/Fail", |_| {})
        .unwrap_err();
    assert_eq!(
        (err.code, err.message),
        (GrpcCode::PermissionDenied, FAIL_MESSAGE)
    );
    let err = channel.call_unary("/missing.Svc/Echo", |_| {}).unwrap_err();
    assert_eq!(err.code, GrpcCode::Unimplemented);
    let resp = channel
        .call_unary("/test.v1.Echo/Echo", |w| {
            let _ = w.write_bytes(b"after errors");
        })
        .unwrap();
    assert_eq!(resp, b"after errors");

    // Also verify multiplexed GrpcClient + GrpcServer calls allocate 0 bytes on the heap.
    let (mut m2, mut s2, mut c2) = ([0u8; 512], [0u8; 4096], [0u8; 4096]);
    let mut srv: GrpcServer = GrpcServer::new(&routes, &mut m2, &mut s2);
    let mut cli: GrpcClient = GrpcClient::new("http", "localhost", &mut c2);
    let mut bufs = [[0u8; 128]; 4];
    let [b0, b1, b2, b3] = &mut bufs;
    let id0 = cli
        .start_call("/test.v1.Echo/Echo", b0, |w| {
            let _ = w.write_bytes(b"mux-0");
        })
        .unwrap();
    let id1 = cli
        .start_call("/test.v1.Reverse/Reverse", b1, |w| {
            let _ = w.write_bytes(b"mux-1");
        })
        .unwrap();
    let id2 = cli.start_call("/test.v1.Echo/Fail", b2, |_| {}).unwrap();
    let id3 = cli
        .start_call("/test.v1.Echo/Echo", b3, |w| {
            let _ = w.write_bytes(b"mux-3");
        })
        .unwrap();

    let mut wire = [0u8; 1024];
    loop {
        let n = cli.poll_transmit(&mut wire);
        srv.recv(&wire[..n]).unwrap();
        let m = srv.poll_transmit(&mut wire);
        cli.recv(&wire[..m]).unwrap();
        if n == 0 && m == 0 {
            break;
        }
    }
    assert_eq!(cli.take_reply(id0).unwrap().result().unwrap(), b"mux-0");
    assert_eq!(cli.take_reply(id1).unwrap().result().unwrap(), b"1-xum");
    assert_eq!(
        cli.take_reply(id2).unwrap().result().unwrap_err().code,
        GrpcCode::PermissionDenied
    );
    assert_eq!(cli.take_reply(id3).unwrap().result().unwrap(), b"mux-3");

    assert_eq!(
        allocs(),
        before,
        "loopback and multiplexed paths must not allocate"
    );
}

// ============================================================================
// Issue #035: Outbound flow-control window queuing, small tx_buf streaming, and mid-transfer cancellation
// ============================================================================

#[test]
fn streams_single_unary_call_exceeding_initial_window_in_both_directions() {
    with_big_stack(|| {
        const CAP: usize = 128 * 1024;
        const PAYLOAD_LEN: usize = 100 * 1024; // 100 KiB > 65,535-byte default initial window
        let routes = routes();
        let mut msg = vec![0u8; CAP];
        let mut stx = vec![0u8; CAP];
        let mut ctx = vec![0u8; CAP];
        let mut server: GrpcServer<4, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient<4, CAP> = GrpcClient::new("http", "localhost", &mut ctx);
        let mut buf = vec![0u8; CAP];
        let payload: Vec<u8> = (0..PAYLOAD_LEN).map(|i| (i * 31 + 17) as u8).collect();

        // Start a single 100 KiB Echo call over default 65,535-byte initial windows.
        let id = client
            .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
                let _ = w.write_bytes(&payload);
            })
            .unwrap();

        let mut wire = vec![0u8; CAP];
        // First client poll_transmit must pause at the 65,535-byte initial send window
        // (cannot finish the 100 KiB + 5-byte LPM request in one pass before server WINDOW_UPDATE).
        let n1 = client.poll_transmit(&mut wire);
        assert!(n1 > 65_535 && n1 < PAYLOAD_LEN);
        assert_eq!(client.poll_transmit(&mut wire), 0);

        // Deliver initial 65,535 bytes of request to server; server emits SETTINGS + WINDOW_UPDATE.
        server.recv(&wire[..n1]).unwrap();
        let m1 = server.poll_transmit(&mut wire);
        assert!(m1 > 0);
        // Server has not yet received the complete 100 KiB request, so call is not finished.
        client.recv(&wire[..m1]).unwrap();
        assert!(client.take_reply(id).is_none());

        // Client now has fresh WINDOW_UPDATE credit and emits the remaining ~37 KiB of request DATA.
        let n2 = client.poll_transmit(&mut wire);
        assert!(n2 > 30_000);
        server.recv(&wire[..n2]).unwrap();

        // Server dispatches the 100 KiB Echo response, which also pauses at 65,535 bytes on its first pass!
        let m2 = server.poll_transmit(&mut wire);
        assert!(m2 > 65_535 && m2 < PAYLOAD_LEN);
        assert_eq!(server.poll_transmit(&mut wire), 0);

        // Client receives first 65,535 bytes of response DATA and emits WINDOW_UPDATE.
        client.recv(&wire[..m2]).unwrap();
        assert!(client.take_reply(id).is_none());
        let n3 = client.poll_transmit(&mut wire);
        assert!(n3 > 0);

        // Server receives client's WINDOW_UPDATE and emits the remaining response DATA + trailers.
        server.recv(&wire[..n3]).unwrap();
        let m3 = server.poll_transmit(&mut wire);
        assert!(m3 > 30_000);
        client.recv(&wire[..m3]).unwrap();

        let reply = client.take_reply(id).expect("100 KiB Echo completed");
        assert_eq!(reply.result().unwrap(), payload.as_slice());
    });
}

#[test]
fn queues_concurrent_responses_exceeding_connection_window_in_single_recv_batch() {
    with_big_stack(|| {
        const CAP: usize = 32 * 1024;
        const RESP_LEN: u32 = 16 * 1024; // 6 * 16 KiB = 96 KiB > 65,535-byte connection send window
        let routes = routes();
        let mut msg = vec![0u8; CAP];
        let mut stx = vec![0u8; 128 * 1024];
        let mut ctx = vec![0u8; 64 * 1024];
        let mut server: GrpcServer<8, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient<8, CAP> = GrpcClient::new("http", "localhost", &mut ctx);
        let mut bufs = [
            vec![0u8; CAP],
            vec![0u8; CAP],
            vec![0u8; CAP],
            vec![0u8; CAP],
            vec![0u8; CAP],
            vec![0u8; CAP],
        ];
        let [b0, b1, b2, b3, b4, b5] = &mut bufs;

        let ids = [
            client
                .start_call("/test.v1.Echo/Big", b0, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
            client
                .start_call("/test.v1.Echo/Big", b1, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
            client
                .start_call("/test.v1.Echo/Big", b2, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
            client
                .start_call("/test.v1.Echo/Big", b3, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
            client
                .start_call("/test.v1.Echo/Big", b4, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
            client
                .start_call("/test.v1.Echo/Big", b5, |w| {
                    let _ = w.write_bytes(&RESP_LEN.to_le_bytes());
                })
                .unwrap(),
        ];

        // Feed all 6 requests in a single server.recv batch so all 6 responses are dispatched before
        // any client WINDOW_UPDATE arrives.
        let mut wire = vec![0u8; 128 * 1024];
        let n = client.poll_transmit(&mut wire);
        server.recv(&wire[..n]).unwrap();

        // Shuttle remaining frames (server initial 65,535 bytes -> client WINDOW_UPDATE -> remaining ~31 KiB).
        shuttle(&mut client, &mut server, whole);

        let expected: Vec<u8> = (0..RESP_LEN as usize).map(|i| i as u8).collect();
        for id in ids {
            assert_eq!(reply_of(&mut client, id).unwrap(), expected);
        }
    });
}

#[test]
fn client_streams_large_request_through_small_tx_buf_across_poll_transmit() {
    with_big_stack(|| {
        const CAP: usize = 64 * 1024;
        const REQ_LEN: usize = 32 * 1024; // 32 KiB request through a 4 KiB client tx_buf
        let routes = routes();
        let mut msg = vec![0u8; CAP];
        let mut stx = vec![0u8; CAP];
        let mut small_ctx = [0u8; 4096];
        let mut server: GrpcServer<4, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient<4, CAP> = GrpcClient::new("http", "localhost", &mut small_ctx);
        let mut buf = vec![0u8; CAP];
        let payload: Vec<u8> = (0..REQ_LEN).map(|i| (i ^ 0x5A) as u8).collect();

        let id = client
            .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
                let _ = w.write_bytes(&payload);
            })
            .unwrap();

        // Drain client in <= 4 KiB poll_transmit steps into server, and shuttle response back.
        let mut wire = [0u8; 4096];
        let mut client_tx_calls = 0usize;
        loop {
            let n = client.poll_transmit(&mut wire);
            if n > 0 {
                client_tx_calls += 1;
                server.recv(&wire[..n]).unwrap();
            }
            let m = server.poll_transmit(&mut wire);
            if m > 0 {
                client.recv(&wire[..m]).unwrap();
            }
            if n == 0 && m == 0 {
                break;
            }
        }
        assert!(
            client_tx_calls >= 8,
            "32 KiB request over 4 KiB tx_buf must stream across >= 8 poll_transmit calls, got {client_tx_calls}"
        );
        assert_eq!(reply_of(&mut client, id).unwrap(), payload);
    });
}

#[test]
fn mid_transfer_rst_stream_cancels_window_stalled_outbound_data_cleanly() {
    with_big_stack(|| {
        const CAP: usize = 64 * 1024;
        let routes = routes();

        // 1. Server-side stalled outbound response cancelled by client RST_STREAM while sibling stream completes.
        {
            let mut msg = vec![0u8; CAP];
            let mut stx = vec![0u8; 128 * 1024];
            let mut server: GrpcServer<4, CAP> = GrpcServer::new(&routes, &mut msg, &mut stx);
            let mut raw_cli: H2Connection<4, CAP> = H2Connection::new(Role::Client);
            let mut wire = vec![0u8; 128 * 1024];
            let ct = b"application/grpc+roto";

            // Complete handshake (note: raw_cli has auto_window_update = false by default, so we control WINDOW_UPDATE manually).
            let n = raw_cli.poll_transmit(&mut wire);
            server.recv(&wire[..n]).unwrap();
            let m = server.poll_transmit(&mut wire);
            let mut pos = 0;
            while pos < m {
                let (c, _) = raw_cli.recv(&wire[pos..m]).unwrap();
                pos += c;
            }
            let n = raw_cli.poll_transmit(&mut wire);
            server.recv(&wire[..n]).unwrap();
            let _ = server.poll_transmit(&mut wire);

            // Open stream 1 and stream 3, each requesting a 48 KiB response (96 KiB total > 65,535 connection window).
            let mut req_lpm = [0u8; 9];
            req_lpm[1..5].copy_from_slice(&4u32.to_be_bytes());
            req_lpm[5..9].copy_from_slice(&(48 * 1024u32).to_le_bytes());

            let (s1, mut n) = raw_cli
                .send_grpc_request_headers(
                    b"http",
                    b"x",
                    b"/test.v1.Echo/Big",
                    ct,
                    false,
                    &mut wire,
                )
                .unwrap();
            n += raw_cli
                .send_data_frame(s1, &req_lpm, true, &mut wire[n..])
                .unwrap();
            let (s3, n3) = raw_cli
                .send_grpc_request_headers(
                    b"http",
                    b"x",
                    b"/test.v1.Echo/Big",
                    ct,
                    false,
                    &mut wire[n..],
                )
                .unwrap();
            n += n3;
            n += raw_cli
                .send_data_frame(s3, &req_lpm, true, &mut wire[n..])
                .unwrap();
            assert_eq!([s1, s3], [1, 3]);

            // Server dispatches both: s1 sends all 48 KiB + trailers; s3 sends ~16 KiB and stalls with ~32 KiB staged.
            server.recv(&wire[..n]).unwrap();
            let m = server.poll_transmit(&mut wire);
            let mut pos = 0;
            let mut s1_done = false;
            let mut s3_done = false;
            while pos < m {
                let (c, ev) = raw_cli.recv(&wire[pos..m]).unwrap();
                pos += c;
                if let Some(H2Event::Headers {
                    stream_id,
                    end_stream: true,
                    ..
                }) = ev
                {
                    if stream_id == s1 {
                        s1_done = true;
                    }
                    if stream_id == s3 {
                        s3_done = true;
                    }
                }
            }
            assert!(s1_done, "s1 should complete within initial 65,535 window");
            assert!(!s3_done, "s3 must be window-stalled after ~16 KiB");

            // Client cancels stalled s3 with RST_STREAM(Cancel), opens sibling s5 (Echo "ok"),
            // and replenishes connection receive window so s5 can complete.
            raw_cli.queue_rst_stream(s3, H2ErrorCode::Cancel).unwrap();
            raw_cli.replenish_recv_window(0, 65_535).unwrap();
            let mut n = raw_cli.poll_transmit(&mut wire);
            let (s5, n5) = raw_cli
                .send_grpc_request_headers(
                    b"http",
                    b"x",
                    b"/test.v1.Echo/Echo",
                    ct,
                    false,
                    &mut wire[n..],
                )
                .unwrap();
            n += n5;
            n += raw_cli
                .send_data_frame(s5, &[0, 0, 0, 0, 2, b'o', b'k'], true, &mut wire[n..])
                .unwrap();
            assert_eq!(s5, 5);

            server.recv(&wire[..n]).unwrap();
            let m = server.poll_transmit(&mut wire);
            let mut pos = 0;
            let mut s5_payload = Vec::new();
            let mut s5_trailers = false;
            while pos < m {
                let (c, ev) = raw_cli.recv(&wire[pos..m]).unwrap();
                pos += c;
                match ev {
                    Some(H2Event::GrpcMessage {
                        stream_id: 5,
                        payload,
                        ..
                    }) => s5_payload.extend_from_slice(payload),
                    Some(H2Event::Headers {
                        stream_id: 5,
                        end_stream: true,
                        ..
                    }) => s5_trailers = true,
                    Some(
                        H2Event::GrpcMessage { stream_id: 3, .. }
                        | H2Event::Headers { stream_id: 3, .. },
                    ) => panic!("cancelled stream 3 must not emit further frames"),
                    _ => {}
                }
            }
            assert_eq!(s5_payload, b"ok");
            assert!(s5_trailers);
        }

        // 2. Client-side window-stalled outbound request cancelled by server RST_STREAM while sibling call completes.
        {
            const BIG_CAP: usize = 128 * 1024;
            let mut ctx = vec![0u8; BIG_CAP];
            let mut client: GrpcClient<4, BIG_CAP> = GrpcClient::new("http", "localhost", &mut ctx);
            let mut raw_srv: H2Connection<4, BIG_CAP> = H2Connection::new(Role::Server);
            let mut wire = vec![0u8; BIG_CAP];
            let mut b1 = vec![0u8; BIG_CAP];
            let mut b3 = vec![0u8; 1024];
            let big_req = vec![0x42u8; 90 * 1024]; // 90 KiB > 65,535-byte initial window

            // Start 90 KiB call s1 (consumes all 65,535 bytes of initial connection window, stalling ~25 KiB)
            // and sibling call s3 (stalls behind s1 at 0 bytes sent).
            let s1 = client
                .start_call("/test.v1.Echo/Echo", &mut b1, |w| {
                    let _ = w.write_bytes(&big_req);
                })
                .unwrap();
            let s3 = client
                .start_call("/test.v1.Echo/Echo", &mut b3, |w| {
                    let _ = w.write_bytes(b"sibling");
                })
                .unwrap();
            assert_eq!([s1, s3], [1, 3]);

            let n = client.poll_transmit(&mut wire);
            let mut pos = 0;
            while pos < n {
                let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
                pos += c;
            }

            // Server resets stalled s1 with Cancel and replenishes connection window so sibling s3 can send its request!
            raw_srv.queue_rst_stream(s1, H2ErrorCode::Cancel).unwrap();
            raw_srv.replenish_recv_window(0, 65_535).unwrap();
            let m = raw_srv.poll_transmit(&mut wire);
            client.recv(&wire[..m]).unwrap();

            assert_eq!(
                reply_of(&mut client, s1).unwrap_err().0,
                GrpcCode::Cancelled
            );

            // Client now transmits sibling s3's stalled request DATA (and no further s1 DATA).
            let n2 = client.poll_transmit(&mut wire);
            assert!(n2 > 0);
            let mut pos = 0;
            let mut s3_got_msg = false;
            while pos < n2 {
                let (c, ev) = raw_srv.recv(&wire[pos..n2]).unwrap();
                pos += c;
                if let Some(H2Event::GrpcMessage {
                    stream_id, payload, ..
                }) = ev
                {
                    assert_eq!(stream_id, s3);
                    assert_eq!(payload, b"sibling");
                    s3_got_msg = true;
                }
            }
            assert!(s3_got_msg);
        }

        // 3. Early server Trailers-Only response while client request > 65,535 bytes is window-stalled on GrpcClient<1>:
        // Client must emit RST_STREAM(Cancel) to transition the stream out of HalfClosedRemote and free the single slot
        // so a subsequent start_call on GrpcClient<1> succeeds!
        {
            const BIG_CAP: usize = 128 * 1024;
            let mut ctx = vec![0u8; BIG_CAP];
            let mut client: GrpcClient<1, BIG_CAP> = GrpcClient::new("http", "localhost", &mut ctx);
            let mut raw_srv: H2Connection<2, BIG_CAP> = H2Connection::new(Role::Server);
            let mut wire = vec![0u8; BIG_CAP];
            let mut buf = vec![0u8; BIG_CAP];
            let big_req = vec![0x99u8; 90 * 1024]; // 90 KiB > 65,535-byte initial window
            let ct = b"application/grpc+roto";

            let s1 = client
                .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
                    let _ = w.write_bytes(&big_req);
                })
                .unwrap();
            assert_eq!(s1, 1);

            let n = client.poll_transmit(&mut wire);
            let mut pos = 0;
            while pos < n {
                let (c, _) = raw_srv.recv(&wire[pos..n]).unwrap();
                pos += c;
            }

            // Server immediately responds with Trailers-Only (end_stream = true, PermissionDenied)
            // while client still has ~25 KiB of unsent request DATA on stream 1.
            let mut m = raw_srv.poll_transmit(&mut wire);
            m += raw_srv
                .send_grpc_trailers_only(
                    s1,
                    GrpcStatus::permission_denied("early reject"),
                    ct,
                    &mut wire[m..],
                )
                .unwrap();
            raw_srv.replenish_recv_window(0, 65_535).unwrap();
            m += raw_srv.poll_transmit(&mut wire[m..]);
            client.recv(&wire[..m]).unwrap();

            let reply1 = client
                .take_reply(s1)
                .expect("early Trailers-Only finishes call");
            assert_eq!(
                reply1.result().unwrap_err(),
                GrpcStatus::permission_denied("early reject")
            );
            let buf = reply1.into_buf();

            // Client emits 9-byte SETTINGS ACK + 13-byte RST_STREAM(Cancel) = 22 bytes (and NO remaining s1 DATA frames),
            // closing stream 1 and freeing the single slot on GrpcClient<1>.
            let tx_len = client.poll_transmit(&mut wire);
            assert_eq!(tx_len, FRAME_HEADER_LEN + (FRAME_HEADER_LEN + 4));
            let rst_hdr = FrameHeader::decode_slice(&wire[FRAME_HEADER_LEN..tx_len]).unwrap();
            assert_eq!(rst_hdr.frame_type, FrameType::RST_STREAM);
            assert_eq!(rst_hdr.stream_id, s1);

            // Subsequent start_call on GrpcClient<1> succeeds on stream 3!
            let s3 = client
                .start_call("/test.v1.Echo/Echo", buf, |w| {
                    let _ = w.write_bytes(b"recovered");
                })
                .unwrap();
            assert_eq!(s3, 3);
        }

        // 4. Staged response finishes draining all DATA bytes (`start == end`) when `< 24` (`OK_TRAILERS_WIRE_LEN`)
        // bytes remain in `tx_buf`, runs `compact_staging()` without hanging, and emits `OK_TRAILERS` on the next
        // `poll_transmit`!
        {
            let mut msg = [0u8; 512];
            // After handshake drains, server receives a 43-byte LPM Echo request ("a" * 38):
            // - Auto WINDOW_UPDATEs (conn + stream) occupy 26 bytes in `stx`
            // - Response HEADERS occupy 33 bytes (total 59)
            // - Staged path stages 43 bytes at tail, then `drain_pending_responses` sends all 43 bytes as a
            //   52-byte DATA frame (`59 + 52 = 111`), leaving `start == end` with only `120 - 111 = 9 < 24` bytes
            //   available in `stx` when `compact_staging()` runs!
            let mut stx = [0u8; 120];
            let mut ctx = [0u8; 1024];
            let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
            let mut client: GrpcClient = GrpcClient::new("http", "x", &mut ctx);
            let mut buf = [0u8; 256];

            // Complete handshake first so `stx` starts empty (`out.len == 0`) when the request arrives.
            let mut wire = [0u8; 256];
            let n = client.poll_transmit(&mut wire);
            server.recv(&wire[..n]).unwrap();
            let m = server.poll_transmit(&mut wire);
            client.recv(&wire[..m]).unwrap();
            let n = client.poll_transmit(&mut wire);
            server.recv(&wire[..n]).unwrap();
            assert_eq!(server.poll_transmit(&mut wire), 0);

            let body = [b'a'; 38];
            let id = client
                .start_call("/test.v1.Echo/Echo", &mut buf, |w| {
                    let _ = w.write_bytes(&body);
                })
                .unwrap();
            let n = client.poll_transmit(&mut wire);
            server.recv(&wire[..n]).unwrap();

            // First poll_transmit with a 64-byte buffer drains 64 bytes and frees space in `stx` so
            // `drain_pending_responses` emits the deferred 24-byte `OK_TRAILERS` frame!
            let mut small_out = [0u8; 64];
            let m1 = server.poll_transmit(&mut small_out);
            assert_eq!(m1, 64);
            client.recv(&small_out[..m1]).unwrap();
            assert!(client.take_reply(id).is_none());

            let m2 = server.poll_transmit(&mut wire);
            assert!(m2 > 0);
            client.recv(&wire[..m2]).unwrap();
            assert_eq!(reply_of(&mut client, id).unwrap(), body);
        }
    });
}

#[test]
fn streaming_server_client_and_bidi_loopback_and_edge_cases() {
    let routes = routes();

    // 1. Zero-allocation H2LoopbackChannel server-streaming, client-streaming, and bidi-streaming
    {
        let (mut msg, mut stx, mut ctx, mut call) =
            ([0u8; 512], [0u8; 4096], [0u8; 4096], [0u8; 512]);
        let before = allocs();
        let server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
        let client: GrpcClient = GrpcClient::new("http", "localhost", &mut ctx);
        let mut channel = H2LoopbackChannel::new(client, server, &mut call);

        // Server-streaming: 3 messages
        let resp_frames = channel
            .call_server_streaming_bytes("/test.v1.Echo/Expand", &[3, b'x', b'y'])
            .unwrap();
        let mut iter =
            roto_grpc::framing::GrpcFrameIter::new(resp_frames, 1024).with_require_complete(true);
        assert_eq!(iter.next().unwrap().unwrap().payload, b"\x00xy");
        assert_eq!(iter.next().unwrap().unwrap().payload, b"\x01xy");
        assert_eq!(iter.next().unwrap().unwrap().payload, b"\x02xy");
        assert!(iter.next().is_none());

        // Server-streaming: 0 messages (valid empty stream with status 0)
        let empty_frames = channel
            .call_server_streaming_bytes("/test.v1.Echo/Expand", &[0])
            .unwrap();
        assert!(empty_frames.is_empty());

        // Client-streaming: 3 messages -> 1 response
        let sum_resp = channel
            .call_client_streaming("/test.v1.Echo/Sum", |w| {
                for chunk in [&[1u8, 2][..], &[3, 4][..], &[10][..]] {
                    let end = w.cursor();
                    w.write_bytes(chunk)?;
                    roto_grpc::framing::prepend_grpc_header(w, false, end)?;
                    w.rotate_written_to_end(end);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(u32::from_le_bytes(sum_resp.try_into().unwrap()), 20);

        // Client-streaming: 0 messages -> 1 response (sum = 0)
        let zero_sum = channel
            .call_client_streaming("/test.v1.Echo/Sum", |_w| Ok(()))
            .unwrap();
        assert_eq!(u32::from_le_bytes(zero_sum.try_into().unwrap()), 0);

        // Bidirectional-streaming: 3 messages in -> 3 messages out
        let chat_frames = channel
            .call_bidi_streaming("/test.v1.Echo/Chat", |w| {
                for m in [b"a".as_slice(), b"bc".as_slice(), b"def".as_slice()] {
                    let end = w.cursor();
                    w.write_bytes(m)?;
                    roto_grpc::framing::prepend_grpc_header(w, false, end)?;
                    w.rotate_written_to_end(end);
                }
                Ok(())
            })
            .unwrap();
        let mut chat_iter =
            roto_grpc::framing::GrpcFrameIter::new(chat_frames, 1024).with_require_complete(true);
        assert_eq!(chat_iter.next().unwrap().unwrap().payload, b"pong:a");
        assert_eq!(chat_iter.next().unwrap().unwrap().payload, b"pong:bc");
        assert_eq!(chat_iter.next().unwrap().unwrap().payload, b"pong:def");
        assert!(chat_iter.next().is_none());

        // Bidirectional-streaming: 0 messages in -> 0 messages out
        let empty_chat = channel
            .call_bidi_streaming("/test.v1.Echo/Chat", |_w| Ok(()))
            .unwrap();
        assert!(empty_chat.is_empty());

        // Bidirectional-streaming error status propagation
        let err = channel
            .call_bidi_streaming("/test.v1.Echo/Chat", |w| {
                write_grpc_frame_bytes(w, false, b"abort").map_err(GrpcStatus::from)
            })
            .unwrap_err();
        assert_eq!(err.code, GrpcCode::Aborted);
        assert_eq!(err.message, "chat aborted");

        assert_eq!(
            allocs(),
            before,
            "streaming loopback calls must not allocate"
        );
    }

    // 2. Chunked 1-byte delivery for client-streaming & bidi-streaming + compressed frame / REASM_CAP overflow rejection
    {
        let (mut msg, mut stx, mut ctx) = ([0u8; 256], [0u8; 4096], [0u8; 4096]);
        let mut server: GrpcServer<4, 128> = GrpcServer::new(&routes, &mut msg, &mut stx);
        let mut client: GrpcClient<4, 128> = GrpcClient::new("http", "localhost", &mut ctx);
        let (mut b1, mut b3, mut b5) = ([0u8; 256], [0u8; 256], [0u8; 256]);

        // Stream 1: valid 2-message client stream delivered 1 byte at a time
        let s1 = client
            .start_client_streaming_call("/test.v1.Echo/Sum", &mut b1, |w| {
                for m in [&[5u8][..], &[7u8][..]] {
                    let end = w.cursor();
                    w.write_bytes(m)?;
                    roto_grpc::framing::prepend_grpc_header(w, false, end)?;
                    w.rotate_written_to_end(end);
                }
                Ok(())
            })
            .unwrap();

        // Stream 3: valid 2-message bidi stream delivered 1 byte at a time
        let s3 = client
            .start_bidi_streaming_call("/test.v1.Echo/Chat", &mut b3, |w| {
                for m in [b"one".as_slice(), b"two".as_slice()] {
                    let end = w.cursor();
                    w.write_bytes(m)?;
                    roto_grpc::framing::prepend_grpc_header(w, false, end)?;
                    w.rotate_written_to_end(end);
                }
                Ok(())
            })
            .unwrap();

        // Stream 5: client stream with compressed flag = 1 -> Internal
        let s5 = client
            .start_client_streaming_call("/test.v1.Echo/Sum", &mut b5, |w| {
                let end = w.cursor();
                w.write_bytes(&[1])?;
                roto_grpc::framing::prepend_grpc_header(w, true, end)?;
                Ok(())
            })
            .unwrap();

        shuttle(&mut client, &mut server, || 1);

        assert_eq!(
            u32::from_le_bytes(reply_of(&mut client, s1).unwrap().try_into().unwrap()),
            12
        );
        let chat_bytes = reply_of(&mut client, s3).unwrap();
        let mut chat_iter =
            roto_grpc::framing::GrpcFrameIter::new(&chat_bytes, 1024).with_require_complete(true);
        assert_eq!(chat_iter.next().unwrap().unwrap().payload, b"pong:one");
        assert_eq!(chat_iter.next().unwrap().unwrap().payload, b"pong:two");
        assert!(chat_iter.next().is_none());
        assert_eq!(reply_of(&mut client, s5).unwrap_err().0, GrpcCode::Internal);

        // Stream 7: client stream exceeding 128-byte PendingCall REASM_CAP -> ResourceExhausted
        let mut b7 = [0u8; 256];
        let s7 = client
            .start_client_streaming_call("/test.v1.Echo/Sum", &mut b7, |w| {
                for _ in 0..16 {
                    let end = w.cursor();
                    w.write_bytes(&[1, 2, 3, 4, 5])?;
                    roto_grpc::framing::prepend_grpc_header(w, false, end)?;
                    w.rotate_written_to_end(end);
                }
                Ok(())
            })
            .unwrap();
        shuttle(&mut client, &mut server, whole);
        assert_eq!(
            reply_of(&mut client, s7).unwrap_err().0,
            GrpcCode::ResourceExhausted
        );
    }
}
