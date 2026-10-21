//! In-memory Sans-I/O client <-> server loopback integration tests for `roto_grpc::{server, client}`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use roto::runtime::BackwardWriter;
use roto_grpc::framing::write_grpc_frame_bytes;
use roto_grpc::h2::{H2Connection, H2Event, Role};
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
            _ => Err(GrpcStatus::unimplemented("unimplemented gRPC method")),
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
        (roto_grpc::h2::H2ErrorCode::Cancel, GrpcCode::Cancelled),
        (
            roto_grpc::h2::H2ErrorCode::RefusedStream,
            GrpcCode::Unavailable,
        ),
        (
            roto_grpc::h2::H2ErrorCode::EnhanceYourCalm,
            GrpcCode::ResourceExhausted,
        ),
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
        .queue_goaway(1, roto_grpc::h2::H2ErrorCode::NoError, b"draining")
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

    assert_eq!(reply_of(&mut client, s3).unwrap_err().0, GrpcCode::Unavailable);
    assert_eq!(reply_of(&mut client, s1).unwrap(), b"");

    let after_goaway = client
        .start_call("/test.v1.Echo/Echo", &mut b5, |_| {})
        .unwrap_err();
    assert_eq!(after_goaway.result().unwrap_err().code, GrpcCode::Unavailable);
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
    let recovered = client2.take_reply(sid).expect("in-flight call finished on fatal error");
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
    let _ = format!("{:?} {:?} {:?}", channel, channel.client(), channel.server());
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

    assert_eq!(allocs(), before, "loopback and multiplexed paths must not allocate");
}
