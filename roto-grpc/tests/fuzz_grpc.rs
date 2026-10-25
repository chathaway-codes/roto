//! Deterministic fuzzer feeding random and mutated wire bytes into the gRPC server and client
//! drivers. Neither may panic or hang on any input.
//!
//! Tune with `ROTO_FUZZ_SEED` and `ROTO_FUZZ_ITERS`.

use roto::runtime::BackwardWriter;
use roto_grpc::framing::write_grpc_frame_bytes;
use roto_grpc::h2::{
    FrameHeader, FrameType, CLIENT_CONNECTION_PREFACE, FLAG_ACK, FRAME_HEADER_LEN,
};
use roto_grpc::{GrpcClient, GrpcCode, GrpcServer, GrpcService, GrpcStatus, Route};

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            0
        } else {
            (self.next_u64() as usize) % bound
        }
    }
}

fn fuzz_seed() -> u64 {
    std::env::var("ROTO_FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0x0032_6772_7063_F00D)
}

fn fuzz_iters(default_iters: usize) -> usize {
    std::env::var("ROTO_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default_iters)
}

struct Echo;

impl GrpcService for Echo {
    const SERVICE_NAME: &'static str = "fuzz.Echo";

    fn call_unary(
        &self,
        method_name: &str,
        req_payload: &[u8],
        writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        match method_name {
            "Echo" => Ok(write_grpc_frame_bytes(writer, false, req_payload)?),
            "Fail" => Err(GrpcStatus::aborted("fail: 100% ✗")),
            _ => Err(GrpcStatus::unimplemented("unimplemented gRPC method")),
        }
    }
}

const PATHS: [&str; 4] = ["/fuzz.Echo/Echo", "/fuzz.Echo/Fail", "/fuzz.Nope/X", "/bad"];

/// Mutates `bytes` in place with a few random byte flips, overwrites, truncation, or splices.
fn mutate(rng: &mut SplitMix64, bytes: &mut Vec<u8>) {
    for _ in 0..1 + rng.next_usize(4) {
        if bytes.is_empty() {
            return;
        }
        let i = rng.next_usize(bytes.len());
        match rng.next_usize(5) {
            0 => bytes[i] ^= 1 << rng.next_usize(8),
            1 => bytes[i] = rng.next_u64() as u8,
            2 => bytes.truncate(i),
            3 => bytes.insert(i, rng.next_u64() as u8),
            _ => {
                let j = rng.next_usize(bytes.len());
                bytes[i] = bytes[j];
            }
        }
    }
}

/// Feeds `input` in random chunks; stops at the first (fatal) connection error.
fn feed(rng: &mut SplitMix64, input: &[u8], mut recv: impl FnMut(&[u8]) -> bool) {
    let mut rest = input;
    while !rest.is_empty() {
        let take = (1 + rng.next_usize(64)).min(rest.len());
        if !recv(&rest[..take]) {
            return;
        }
        rest = &rest[take..];
    }
}

/// Records a valid client transcript (several calls) and the server's valid response transcript.
fn transcripts() -> (Vec<u8>, Vec<u8>) {
    let routes: [&dyn Route; 1] = [&Echo];
    let (mut msg, mut stx, mut ctx) = ([0u8; 256], [0u8; 4096], [0u8; 4096]);
    let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut client: GrpcClient = GrpcClient::new("http", "fuzz", &mut ctx);
    let mut bufs = [[0u8; 64]; PATHS.len()];
    for (path, buf) in PATHS.iter().zip(bufs.iter_mut()) {
        client
            .start_call(path, buf, |w| {
                let _ = w.write_bytes(b"payload");
            })
            .unwrap();
    }
    let mut wire = [0u8; 4096];
    let n = client.poll_transmit(&mut wire);
    let to_server = wire[..n].to_vec();
    server.recv(&to_server).unwrap();
    let m = server.poll_transmit(&mut wire);
    (to_server, wire[..m].to_vec())
}

/// Validates that `bytes` is a sequence of complete, well-formed HTTP/2 frames.
fn assert_valid_h2_frames(mut bytes: &[u8]) {
    while !bytes.is_empty() {
        let hdr = FrameHeader::decode_slice(bytes).expect("complete 9-byte frame header");
        let frame_len = FRAME_HEADER_LEN + hdr.length as usize;
        assert!(bytes.len() >= frame_len, "truncated emitted HTTP/2 frame");
        match hdr.frame_type {
            FrameType::SETTINGS => {
                assert_eq!(hdr.stream_id, 0);
                if (hdr.flags & FLAG_ACK) != 0 {
                    assert_eq!(hdr.length, 0);
                } else {
                    assert_eq!(hdr.length % 6, 0);
                }
            }
            FrameType::PING => {
                assert_eq!(hdr.stream_id, 0);
                assert_eq!(hdr.length, 8);
            }
            FrameType::GOAWAY => {
                assert_eq!(hdr.stream_id, 0);
                assert!(hdr.length >= 8);
            }
            FrameType::WINDOW_UPDATE => {
                assert_eq!(hdr.length, 4);
            }
            FrameType::RST_STREAM => {
                assert_ne!(hdr.stream_id, 0);
                assert_eq!(hdr.length, 4);
            }
            FrameType::HEADERS | FrameType::CONTINUATION | FrameType::DATA => {
                assert_ne!(hdr.stream_id, 0);
            }
            other => panic!("unexpected frame type {other:#x} emitted by server"),
        }
        bytes = &bytes[frame_len..];
    }
}

/// Returns the number of bytes the server transmitted.
fn run_server(rng: &mut SplitMix64, input: &[u8]) -> usize {
    let routes: [&dyn Route; 1] = [&Echo];
    let (mut msg, mut stx, mut wire) = ([0u8; 128], [0u8; 1024], [0u8; 512]);
    let mut server: GrpcServer<4, 256> = GrpcServer::new(&routes, &mut msg, &mut stx);
    let mut emitted = Vec::new();
    feed(rng, input, |chunk| {
        let ok = server.recv(chunk).is_ok();
        loop {
            match server.poll_transmit(&mut wire) {
                0 => return ok,
                n => emitted.extend_from_slice(&wire[..n]),
            }
        }
    });
    assert_valid_h2_frames(&emitted);
    emitted.len()
}

/// Returns the number of calls that finished.
fn run_client(rng: &mut SplitMix64, input: &[u8]) -> usize {
    let (mut ctx, mut wire) = ([0u8; 2048], [0u8; 512]);
    let mut client: GrpcClient<4, 256> = GrpcClient::new("http", "fuzz", &mut ctx);
    let mut bufs = [[0u8; 32]; PATHS.len()];
    let mut ids = [0u32; PATHS.len()];
    for ((path, buf), id) in PATHS.iter().zip(bufs.iter_mut()).zip(ids.iter_mut()) {
        *id = client.start_call(path, buf, |_| {}).unwrap_or(0);
    }
    while client.poll_transmit(&mut wire) > 0 {}
    feed(rng, input, |chunk| {
        let ok = client.recv(chunk).is_ok();
        while client.poll_transmit(&mut wire) > 0 {}
        ok
    });
    let mut finished = 0;
    for id in ids {
        if let Some(reply) = client.take_reply(id) {
            let _ = reply.result();
            finished += 1;
        }
    }
    finished
}

#[test]
fn fuzz_server_with_random_and_mutated_client_bytes() {
    let mut rng = SplitMix64::new(fuzz_seed());
    let (valid, _) = transcripts();
    let mut sent = 0;
    for _ in 0..fuzz_iters(2000) {
        let mut input = if rng.next_usize(4) == 0 {
            // Random frames after a valid preface.
            let mut v = CLIENT_CONNECTION_PREFACE.to_vec();
            v.extend((0..rng.next_usize(256)).map(|_| rng.next_u64() as u8));
            v
        } else {
            valid.clone()
        };
        mutate(&mut rng, &mut input);
        sent += run_server(&mut rng, &input);
    }
    assert!(sent > 0, "fuzzer must reach the server's transmit path");
}

#[test]
fn fuzz_client_with_random_and_mutated_server_bytes() {
    let mut rng = SplitMix64::new(fuzz_seed() ^ 0x00C1_1E17);
    let (_, valid) = transcripts();
    let mut finished = 0;
    for _ in 0..fuzz_iters(2000) {
        let mut input = if rng.next_usize(4) == 0 {
            (0..rng.next_usize(256))
                .map(|_| rng.next_u64() as u8)
                .collect()
        } else {
            valid.clone()
        };
        mutate(&mut rng, &mut input);
        finished += run_client(&mut rng, &input);
    }
    assert!(finished > 0, "fuzzer must reach call completion");
}

#[test]
fn unmutated_transcripts_complete_every_call() {
    let (to_server, to_client) = transcripts();
    let mut rng = SplitMix64::new(fuzz_seed());
    assert!(run_server(&mut rng, &to_server) > to_client.len() / 2);

    let (mut ctx, mut wire) = ([0u8; 2048], [0u8; 512]);
    let mut client: GrpcClient = GrpcClient::new("http", "fuzz", &mut ctx);
    let mut bufs = [[0u8; 64]; PATHS.len()];
    let mut ids = Vec::new();
    for (path, buf) in PATHS.iter().zip(bufs.iter_mut()) {
        ids.push(
            client
                .start_call(path, buf, |w| {
                    let _ = w.write_bytes(b"payload");
                })
                .unwrap(),
        );
    }
    while client.poll_transmit(&mut wire) > 0 {}
    client.recv(&to_client).unwrap();

    assert_eq!(
        client.take_reply(ids[0]).unwrap().result().unwrap(),
        b"payload"
    );
    assert_eq!(
        client.take_reply(ids[1]).unwrap().result().unwrap_err(),
        GrpcStatus::aborted("fail: 100% ✗")
    );
    assert_eq!(
        client
            .take_reply(ids[2])
            .unwrap()
            .result()
            .unwrap_err()
            .code,
        GrpcCode::Unimplemented
    );
    assert_eq!(
        client
            .take_reply(ids[3])
            .unwrap()
            .result()
            .unwrap_err()
            .code,
        GrpcCode::Unimplemented
    );
}
