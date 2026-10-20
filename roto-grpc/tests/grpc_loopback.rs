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
    assert_eq!([echo, rev, fail, no_svc], [1, 3, 5, 7]);

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

    client
        .start_call("/test.v1.Echo/Echo", &mut a, |_| {})
        .unwrap();
    client
        .start_call("/test.v1.Echo/Echo", &mut b, |_| {})
        .unwrap();
    let err = client
        .start_call("/test.v1.Echo/Echo", &mut c, |_| {})
        .unwrap_err();
    assert_eq!(err.result().unwrap_err().code, GrpcCode::ResourceExhausted);
    shuttle(&mut client, &mut server, whole);
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

    let mut block = [0u8; 512];
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
    // Compressed-flag messages are not supported.
    let compressed = [1, 0, 0, 0, 0];
    assert_eq!(
        raw_call(&request(b"POST", b"trailers", ct), Some(&compressed)).0,
        GrpcCode::Unimplemented
    );
    // Plain `application/grpc` is accepted.
    let echo = raw_call(
        &request(b"POST", b"trailers", b"application/grpc"),
        Some(LPM_EMPTY),
    );
    assert_eq!(echo.0, GrpcCode::Ok);
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
    assert_eq!(allocs(), before, "loopback path must not allocate");
}
