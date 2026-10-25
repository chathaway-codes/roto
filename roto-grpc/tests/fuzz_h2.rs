use google_protobuf_roto::google::protobuf::{TimestampBuilder, TimestampView};
use roto::runtime::{BackwardWriter, EncodeError};
use roto_grpc::framing::{encode_grpc_message, write_grpc_frame_bytes};
use roto_grpc::h2::{
    encode_grpc_data_frame, write_grpc_data_frame, write_grpc_data_frame_bytes, FrameHeader,
    FrameType, H2Connection, H2ErrorCode, H2Event, Role, StreamState, CLIENT_CONNECTION_PREFACE,
    DEFAULT_INITIAL_WINDOW_SIZE, FLAG_ACK, FLAG_END_HEADERS, FLAG_END_STREAM, FLAG_PADDED,
    FLAG_PRIORITY, FRAME_HEADER_LEN, MAX_FLOW_CONTROL_WINDOW, MAX_STREAM_ID, SETTINGS_ENABLE_PUSH,
    SETTINGS_INITIAL_WINDOW_SIZE, SETTINGS_MAX_FRAME_SIZE,
};
use roto_grpc::hpack::{GrpcRequestHeaders, HpackDecoder, HpackEncoder, GRPC_CONTENT_TYPE_ROTO};
use roto_grpc::status::GrpcStatus;

// ============================================================================
// Deterministic PRNG & Test Helpers
// ============================================================================

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

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

fn fuzz_seed() -> u64 {
    std::env::var("ROTO_FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0x0030_9113_DEAD_BEEF)
}

fn fuzz_iters(default_iters: usize) -> usize {
    std::env::var("ROTO_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default_iters)
}

fn make_frame(frame_type: u8, flags: u8, stream_id: u32, payload: &[u8]) -> Vec<u8> {
    let hdr = FrameHeader::new(payload.len() as u32, frame_type, flags, stream_id);
    let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    out.extend_from_slice(&hdr.encode());
    out.extend_from_slice(payload);
    out
}

fn complete_handshake<const N: usize, const CAP: usize>(
    client: &mut H2Connection<N, CAP>,
    server: &mut H2Connection<N, CAP>,
) {
    let mut c2s = [0u8; 256];
    let c2s_len = client.poll_transmit(&mut c2s);
    assert!(c2s_len >= CLIENT_CONNECTION_PREFACE.len() + FRAME_HEADER_LEN);

    let mut pos = 0;
    while pos < c2s_len {
        let (n, _ev) = server.recv(&c2s[pos..c2s_len]).unwrap();
        assert!(n > 0);
        pos += n;
    }
    assert!(server.is_handshake_complete());

    let mut s2c = [0u8; 256];
    let s2c_len = server.poll_transmit(&mut s2c);
    assert!(s2c_len > 0);

    let mut pos = 0;
    while pos < s2c_len {
        let (n, _ev) = client.recv(&s2c[pos..s2c_len]).unwrap();
        assert!(n > 0);
        pos += n;
    }
    assert!(client.is_handshake_complete());
    assert!(client.peer_acked_settings());

    // Flush client's SETTINGS ACK back to server
    let c_ack_len = client.poll_transmit(&mut c2s);
    assert_eq!(c_ack_len, FRAME_HEADER_LEN);
    let (n, ev) = server.recv(&c2s[..c_ack_len]).unwrap();
    assert_eq!(n, FRAME_HEADER_LEN);
    assert_eq!(ev, Some(H2Event::SettingsAck));
    assert!(server.peer_acked_settings());
}

// ============================================================================
// 1. FrameHeader Codec & BackwardWriter Tests
// ============================================================================

#[test]
fn frame_header_codec_and_reserved_bit_masking() {
    let hdr = FrameHeader::new(0x00AB_CDEF, FrameType::DATA, FLAG_END_STREAM, 0x7FFF_1234);
    let encoded = hdr.encode();
    assert_eq!(encoded[0..3], [0xAB, 0xCD, 0xEF]);
    assert_eq!(encoded[3], FrameType::DATA);
    assert_eq!(encoded[4], FLAG_END_STREAM);
    // Reserved high bit MUST be 0 on encode
    assert_eq!(encoded[5] & 0x80, 0);

    let decoded = FrameHeader::decode(encoded);
    assert_eq!(decoded, hdr);
    assert_eq!(FrameHeader::decode_slice(&encoded), Some(hdr));
    assert_eq!(FrameHeader::decode_slice(&encoded[..8]), None);

    // Reserved high bit 0x8000_0000 in wire bytes MUST be masked out on decode
    let mut with_reserved = encoded;
    with_reserved[5] |= 0x80;
    let decoded_masked = FrameHeader::decode(with_reserved);
    assert_eq!(decoded_masked.stream_id, 0x7FFF_1234);

    // BackwardWriter verification
    let mut buf = [0u8; 32];
    let mut bw = BackwardWriter::new(&mut buf);
    hdr.write_backward(&mut bw);
    let slice = bw.finish().unwrap();
    assert_eq!(slice, &encoded);

    // Default H2Connection stack footprint must stay under 10 KiB for bare-metal targets
    assert!(core::mem::size_of::<H2Connection>() < 10 * 1024);
}

#[test]
fn single_pass_backward_grpc_data_frame_encoding() {
    let mut buf = [0u8; 128];
    let mut writer = BackwardWriter::new(&mut buf);

    write_grpc_data_frame(&mut writer, 3, true, false, |w| {
        w.with_sub_writer(|sub| {
            let mut tb = TimestampBuilder::from_writer(sub);
            tb.write_nanos(42_000).write_seconds(1_700_000_000);
            tb.writer
        });
    })
    .unwrap();

    let wire = writer.finish().unwrap();
    assert!(wire.len() > FRAME_HEADER_LEN + 5);

    let hdr = FrameHeader::decode_slice(wire).unwrap();
    assert_eq!(hdr.frame_type, FrameType::DATA);
    assert_eq!(hdr.flags, FLAG_END_STREAM);
    assert_eq!(hdr.stream_id, 3);
    assert_eq!(hdr.length as usize, wire.len() - FRAME_HEADER_LEN);

    // Also verify write_grpc_data_frame_bytes equivalence
    let payload_slice = &wire[FRAME_HEADER_LEN + 5..];
    let mut buf2 = [0u8; 128];
    let mut writer2 = BackwardWriter::new(&mut buf2);
    write_grpc_data_frame_bytes(&mut writer2, 3, true, false, payload_slice).unwrap();
    assert_eq!(writer2.finish().unwrap(), wire);

    // Overflow on tiny buffer must return Err(EncodeError::BufferOverflow)
    let mut tiny = [0u8; 10];
    let mut tiny_writer = BackwardWriter::new(&mut tiny);
    assert_eq!(
        encode_grpc_data_frame(&mut tiny_writer, 1, false, false, |w| {
            let _ = w.write_bytes(b"hello world");
        }),
        Err(EncodeError::BufferOverflow)
    );
}

// ============================================================================
// 2. Client-to-Server Handshake & Preface Validation
// ============================================================================

#[test]
fn client_server_handshake_and_preface_rejection() {
    let mut client = H2Connection::<8, 1024>::new_client();
    let mut server = H2Connection::<8, 1024>::new_server();

    assert_eq!(client.role(), Role::Client);
    assert_eq!(server.role(), Role::Server);
    assert!(!client.is_handshake_complete());
    assert!(!server.is_handshake_complete());
    assert_eq!(server.local_settings().header_table_size, 0);
    assert!(!server.local_settings().enable_push);
    assert_eq!(server.local_settings().max_concurrent_streams, 8);

    complete_handshake(&mut client, &mut server);

    assert_eq!(client.peer_settings().header_table_size, 0);
    assert!(!client.peer_settings().enable_push);
    assert_eq!(client.peer_settings().max_concurrent_streams, 8);
    assert_eq!(server.peer_settings().header_table_size, 0);
    assert!(!server.peer_settings().enable_push);

    // Bad client connection preface -> ProtocolError + queued GOAWAY
    let mut bad_server = H2Connection::<4, 512>::new_server();
    let mut drain = [0u8; 128];
    let _ = bad_server.poll_transmit(&mut drain);
    let err = bad_server.recv(b"GET / HTTP/1.1\r\n\r\n").unwrap_err();
    assert!(err.is_connection_error());
    assert_eq!(err.code, H2ErrorCode::ProtocolError);
    assert!(bad_server.has_pending_transmit());

    // Valid 24-byte preface followed by PING instead of SETTINGS -> ProtocolError
    let mut bad_server2 = H2Connection::<4, 512>::new_server();
    let mut wire = Vec::from(&CLIENT_CONNECTION_PREFACE[..]);
    wire.extend_from_slice(&make_frame(
        FrameType::PING,
        0,
        0,
        &[1, 2, 3, 4, 5, 6, 7, 8],
    ));
    let err = bad_server2.recv(&wire).unwrap_err();
    assert_eq!(err.code, H2ErrorCode::ProtocolError);
}

// ============================================================================
// 3. PING & Automatic PING ACK
// ============================================================================

#[test]
fn ping_and_automatic_ping_ack() {
    let mut client = H2Connection::<4, 512>::new_client();
    let mut server = H2Connection::<4, 512>::new_server();
    complete_handshake(&mut client, &mut server);

    let ping_data = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    client.queue_ping(false, ping_data).unwrap();

    let mut wire = [0u8; 64];
    let n = client.poll_transmit(&mut wire);
    assert_eq!(n, FRAME_HEADER_LEN + 8);

    // Server receives PING -> emits Ping { ack: false } and automatically queues PING ACK
    let (consumed, ev) = server.recv(&wire[..n]).unwrap();
    assert_eq!(consumed, n);
    assert_eq!(
        ev,
        Some(H2Event::Ping {
            ack: false,
            data: ping_data
        })
    );
    assert!(server.has_pending_transmit());

    // Transmit PING ACK back to client
    let ack_len = server.poll_transmit(&mut wire);
    assert_eq!(ack_len, FRAME_HEADER_LEN + 8);
    let (consumed, ev) = client.recv(&wire[..ack_len]).unwrap();
    assert_eq!(consumed, ack_len);
    assert_eq!(
        ev,
        Some(H2Event::Ping {
            ack: true,
            data: ping_data
        })
    );
    // Client MUST NOT queue another PING ACK in response to a PING ACK
    assert!(!client.has_pending_transmit());
}

// ============================================================================
// 4. Single-Frame DATA Zero-Copy Fast Path (Pointer Identity Verification)
// ============================================================================

#[test]
fn single_frame_data_zero_copy_fast_path_and_padded_data() {
    let mut client = H2Connection::<4, 1024>::new_client();
    let mut server = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    // Open stream 1 with request HEADERS
    let mut req_hdr_wire = [0u8; 128];
    let (stream_id, hdr_len) = client
        .send_grpc_request_headers(
            b"http",
            b"localhost:50051",
            b"/test.Service/Unary",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut req_hdr_wire,
        )
        .unwrap();
    assert_eq!(stream_id, 1);

    let (c, ev) = server.recv(&req_hdr_wire[..hdr_len]).unwrap();
    assert_eq!(c, hdr_len);
    assert!(matches!(
        ev,
        Some(H2Event::Headers {
            stream_id: 1,
            end_stream: false,
            ..
        })
    ));

    // Encode a single-frame gRPC DATA frame using send_grpc_data_frame
    let mut data_buf = [0u8; 128];
    let data_wire = client
        .send_grpc_data_frame(1, false, true, &mut data_buf, |w| {
            w.with_sub_writer(|sub| {
                let mut tb = TimestampBuilder::from_writer(sub);
                tb.write_nanos(123_456_789).write_seconds(1_700_000_001);
                tb.writer
            });
        })
        .unwrap()
        .to_vec();

    assert_eq!(client.stream_state(1), StreamState::HalfClosedLocal);

    // Feed `data_wire` into `server.recv` and verify that `payload` points DIRECTLY inside `data_wire`!
    let input_range = data_wire.as_ptr_range();
    let (consumed, ev) = server.recv(&data_wire).unwrap();
    assert_eq!(consumed, data_wire.len());
    let Some(H2Event::GrpcMessage {
        stream_id: 1,
        compressed: false,
        payload,
        end_stream: true,
    }) = ev
    else {
        panic!("expected GrpcMessage, got {ev:?}");
    };
    assert!(
        input_range.contains(&payload.as_ptr()),
        "expected zero-copy slice borrowing directly from input `data_wire`"
    );
    let ts = TimestampView::from_bytes(payload).unwrap();
    assert_eq!(ts.seconds(), Ok(1_700_000_001));
    assert_eq!(ts.nanos(), Ok(123_456_789));
    assert_eq!(server.stream_state(1), StreamState::HalfClosedRemote);

    // Also test PADDED DATA frame on stream 3 (zero-copy fast path)
    let (s3, hdr3_len) = client
        .send_grpc_request_headers(
            b"http",
            b"localhost:50051",
            b"/test.Service/Padded",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut req_hdr_wire,
        )
        .unwrap();
    assert_eq!(s3, 3);
    server.recv(&req_hdr_wire[..hdr3_len]).unwrap();

    let mut lpm_buf = [0u8; 64];
    let lpm = encode_grpc_message(&mut lpm_buf, true, |w| {
        let _ = w.write_bytes(b"padded-zero-copy-payload");
    })
    .unwrap();
    let pad_bytes = [0u8; 7];
    let mut padded_payload = Vec::new();
    padded_payload.push(pad_bytes.len() as u8);
    padded_payload.extend_from_slice(lpm);
    padded_payload.extend_from_slice(&pad_bytes);
    let padded_frame = make_frame(
        FrameType::DATA,
        FLAG_PADDED | FLAG_END_STREAM,
        3,
        &padded_payload,
    );
    let padded_range = padded_frame.as_ptr_range();
    let (c, ev) = server.recv(&padded_frame).unwrap();
    assert_eq!(c, padded_frame.len());
    let Some(H2Event::GrpcMessage {
        stream_id: 3,
        compressed: true,
        payload,
        end_stream: true,
    }) = ev
    else {
        panic!("expected padded GrpcMessage, got {ev:?}");
    };
    assert!(padded_range.contains(&payload.as_ptr()));
    assert_eq!(payload, b"padded-zero-copy-payload");
}

// ============================================================================
// 5. Multi-Frame DATA Reassembly & Multiple Messages in One DATA Frame
// ============================================================================

#[test]
fn multi_frame_data_reassembly_and_multi_message_packing() {
    let mut client = H2Connection::<4, 1024>::new_client();
    let mut server = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut hdr_buf = [0u8; 128];
    let (stream_id, hdr_len) = client
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            b"/pkg.Svc/Split",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut hdr_buf,
        )
        .unwrap();
    assert_eq!(stream_id, 1);
    server.recv(&hdr_buf[..hdr_len]).unwrap();

    // Build a gRPC LPM message (5-byte header + 18-byte payload = 23 bytes)
    let mut lpm_buf = [0u8; 64];
    let lpm = encode_grpc_message(&mut lpm_buf, false, |w| {
        let _ = w.write_bytes(b"split-across-three");
    })
    .unwrap();
    assert_eq!(lpm.len(), 23);

    // Split across 3 DATA frames:
    // Frame 1: first 2 bytes of the 5-byte LPM header
    // Frame 2: remaining 3 bytes of LPM header + 7 bytes of payload (10 bytes)
    // Frame 3: final 11 bytes of payload (with END_STREAM = false)
    let f1 = make_frame(FrameType::DATA, 0, 1, &lpm[..2]);
    let f2 = make_frame(FrameType::DATA, 0, 1, &lpm[2..12]);
    let f3 = make_frame(FrameType::DATA, 0, 1, &lpm[12..]);

    assert_eq!(server.recv(&f1).unwrap(), (f1.len(), None));
    assert_eq!(server.recv(&f2).unwrap(), (f2.len(), None));
    let (c3, ev3) = server.recv(&f3).unwrap();
    assert_eq!(c3, f3.len());
    assert_eq!(
        ev3,
        Some(H2Event::GrpcMessage {
            stream_id: 1,
            compressed: false,
            payload: b"split-across-three",
            end_stream: false,
        })
    );

    // Now pack 3 complete gRPC messages into a SINGLE HTTP/2 DATA frame with END_STREAM = true!
    let mut packed_buf = [0u8; 128];
    let mut bw = BackwardWriter::new(&mut packed_buf);
    write_grpc_frame_bytes(&mut bw, false, b"msg-three").unwrap();
    write_grpc_frame_bytes(&mut bw, true, b"msg-two").unwrap();
    write_grpc_frame_bytes(&mut bw, false, b"msg-one").unwrap();
    let packed_lpm = bw.finish().unwrap();
    let packed_frame = make_frame(FrameType::DATA, FLAG_END_STREAM, 1, packed_lpm);
    let packed_range = packed_frame.as_ptr_range();

    let mut offset = 0;
    let mut received = Vec::new();
    while offset < packed_frame.len() {
        let (n, ev) = server.recv(&packed_frame[offset..]).unwrap();
        assert!(n > 0);
        if let Some(H2Event::GrpcMessage {
            stream_id,
            compressed,
            payload,
            end_stream,
        }) = ev
        {
            if received.is_empty() {
                // First message in a multi-message DATA frame still takes the zero-copy fast path!
                assert!(packed_range.contains(&payload.as_ptr()));
            }
            received.push((stream_id, compressed, payload.to_vec(), end_stream));
        }
        offset += n;
    }

    assert_eq!(
        received,
        vec![
            (1, false, b"msg-one".to_vec(), false),
            (1, true, b"msg-two".to_vec(), false),
            (1, false, b"msg-three".to_vec(), true),
        ]
    );
    assert_eq!(server.stream_state(1), StreamState::HalfClosedRemote);

    // Feed a multi-message DATA frame in 1-byte chunks (unpadded on stream 3) and 1..=5 byte chunks (padded on stream 5)
    for (expected_sid, padded, chunk_step) in [(3u32, false, 1usize), (5u32, true, 3usize)] {
        let (sid, n) = client
            .send_grpc_request_headers(
                b"http",
                b"localhost",
                b"/pkg.Svc/ChunkedMulti",
                GRPC_CONTENT_TYPE_ROTO,
                false,
                &mut hdr_buf,
            )
            .unwrap();
        assert_eq!(sid, expected_sid);
        server.recv(&hdr_buf[..n]).unwrap();

        let frame_wire = if padded {
            let pad = [0xEEu8; 6];
            let mut body = Vec::with_capacity(1 + packed_lpm.len() + pad.len());
            body.push(pad.len() as u8);
            body.extend_from_slice(packed_lpm);
            body.extend_from_slice(&pad);
            make_frame(
                FrameType::DATA,
                FLAG_PADDED | FLAG_END_STREAM,
                expected_sid,
                &body,
            )
        } else {
            make_frame(FrameType::DATA, FLAG_END_STREAM, expected_sid, packed_lpm)
        };

        let mut pos = 0;
        let mut chunked_msgs = Vec::new();
        while pos < frame_wire.len() {
            let end = (pos + chunk_step).min(frame_wire.len());
            let (consumed, ev) = server.recv(&frame_wire[pos..end]).unwrap();
            assert!(
                consumed > 0,
                "recv must make forward progress on every chunk"
            );
            pos += consumed;
            if let Some(H2Event::GrpcMessage {
                stream_id,
                compressed,
                payload,
                end_stream,
            }) = ev
            {
                chunked_msgs.push((stream_id, compressed, payload.to_vec(), end_stream));
            }
        }
        assert_eq!(
            chunked_msgs,
            vec![
                (expected_sid, false, b"msg-one".to_vec(), false),
                (expected_sid, true, b"msg-two".to_vec(), false),
                (expected_sid, false, b"msg-three".to_vec(), true),
            ]
        );
        assert_eq!(
            server.stream_state(expected_sid),
            StreamState::HalfClosedRemote
        );
    }
}

// ============================================================================
// 6. Multiplexed Concurrent Streams (Streams 1, 3, 5 Interleaved)
// ============================================================================

#[test]
fn multiplexed_concurrent_streams_interleaved() {
    let mut client = H2Connection::<8, 1024>::new_client();
    let mut server = H2Connection::<8, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut wire = Vec::new();
    let mut tmp = [0u8; 256];

    // Client opens streams 1, 3, 5
    for expected_sid in [1u32, 3, 5] {
        let (sid, n) = client
            .send_grpc_request_headers(
                b"https",
                b"api.internal",
                b"/mux.Svc/Call",
                GRPC_CONTENT_TYPE_ROTO,
                false,
                &mut tmp,
            )
            .unwrap();
        assert_eq!(sid, expected_sid);
        wire.extend_from_slice(&tmp[..n]);
    }
    assert_eq!(client.active_stream_count(), 3);

    // Interleave split DATA frames across streams 1, 3, 5
    let mut lpm1_buf = [0u8; 64];
    let lpm1 = encode_grpc_message(&mut lpm1_buf, false, |w| {
        let _ = w.write_bytes(b"payload-for-stream-1");
    })
    .unwrap();
    let mut lpm3_buf = [0u8; 64];
    let lpm3 = encode_grpc_message(&mut lpm3_buf, false, |w| {
        let _ = w.write_bytes(b"payload-for-stream-3");
    })
    .unwrap();
    let mut lpm5_buf = [0u8; 64];
    let lpm5 = encode_grpc_message(&mut lpm5_buf, false, |w| {
        let _ = w.write_bytes(b"payload-for-stream-5");
    })
    .unwrap();

    // First half of each stream
    wire.extend_from_slice(&make_frame(FrameType::DATA, 0, 1, &lpm1[..6]));
    wire.extend_from_slice(&make_frame(FrameType::DATA, 0, 3, &lpm3[..6]));
    wire.extend_from_slice(&make_frame(FrameType::DATA, 0, 5, &lpm5[..6]));
    // Second half of each stream (in reverse order 5, 3, 1 with END_STREAM)
    wire.extend_from_slice(&make_frame(FrameType::DATA, FLAG_END_STREAM, 5, &lpm5[6..]));
    wire.extend_from_slice(&make_frame(FrameType::DATA, FLAG_END_STREAM, 3, &lpm3[6..]));
    wire.extend_from_slice(&make_frame(FrameType::DATA, FLAG_END_STREAM, 1, &lpm1[6..]));

    // Client local state transitions to HalfClosedLocal when sending END_STREAM DATA
    for sid in [1u32, 3, 5] {
        let _ = client
            .send_grpc_data_frame(sid, false, true, &mut tmp, |w| {
                let _ = w.write_bytes(b"done");
            })
            .unwrap();
        assert_eq!(client.stream_state(sid), StreamState::HalfClosedLocal);
    }

    let mut pos = 0;
    let mut hdr_streams = Vec::new();
    let mut msgs = Vec::new();
    while pos < wire.len() {
        let (n, ev) = server.recv(&wire[pos..]).unwrap();
        pos += n;
        match ev {
            Some(H2Event::Headers {
                stream_id,
                end_stream: false,
                ..
            }) => hdr_streams.push(stream_id),
            Some(H2Event::GrpcMessage {
                stream_id,
                payload,
                end_stream: true,
                ..
            }) => msgs.push((stream_id, payload.to_vec())),
            _ => {}
        }
    }

    assert_eq!(hdr_streams, vec![1, 3, 5]);
    assert_eq!(
        msgs,
        vec![
            (5, b"payload-for-stream-5".to_vec()),
            (3, b"payload-for-stream-3".to_vec()),
            (1, b"payload-for-stream-1".to_vec()),
        ]
    );

    // Server sends response HEADERS + DATA + TRAILERS on stream 3, and Trailers-Only on stream 1 and 5
    let mut resp_wire = Vec::new();
    let n = server
        .send_grpc_trailers_only(
            1,
            GrpcStatus::unimplemented("method disabled"),
            GRPC_CONTENT_TYPE_ROTO,
            &mut tmp,
        )
        .unwrap();
    resp_wire.extend_from_slice(&tmp[..n]);
    assert_eq!(server.stream_state(1), StreamState::Closed);

    let n = server
        .send_grpc_response_headers(3, GRPC_CONTENT_TYPE_ROTO, &mut tmp)
        .unwrap();
    resp_wire.extend_from_slice(&tmp[..n]);
    let data_s3 = server
        .send_grpc_data_frame(3, false, false, &mut tmp, |w| {
            let _ = w.write_bytes(b"resp-3");
        })
        .unwrap();
    resp_wire.extend_from_slice(data_s3);
    let n = server
        .send_grpc_trailers(3, GrpcStatus::ok(), &mut tmp)
        .unwrap();
    resp_wire.extend_from_slice(&tmp[..n]);
    assert_eq!(server.stream_state(3), StreamState::Closed);

    let n = server
        .send_grpc_trailers_only(5, GrpcStatus::ok(), GRPC_CONTENT_TYPE_ROTO, &mut tmp)
        .unwrap();
    resp_wire.extend_from_slice(&tmp[..n]);
    assert_eq!(server.stream_state(5), StreamState::Closed);
    assert_eq!(server.active_stream_count(), 0);

    // Feed resp_wire into client and verify all client events and stream closures
    let mut c_pos = 0;
    let mut client_events = Vec::new();
    while c_pos < resp_wire.len() {
        let (n, ev) = client.recv(&resp_wire[c_pos..]).unwrap();
        assert!(n > 0);
        c_pos += n;
        match ev {
            Some(H2Event::Headers {
                stream_id,
                end_stream,
                ..
            }) => client_events.push((stream_id, "headers", Vec::new(), end_stream)),
            Some(H2Event::GrpcMessage {
                stream_id,
                payload,
                end_stream,
                ..
            }) => client_events.push((stream_id, "msg", payload.to_vec(), end_stream)),
            _ => {}
        }
    }
    assert_eq!(
        client_events,
        vec![
            (1, "headers", Vec::new(), true),
            (3, "headers", Vec::new(), false),
            (3, "msg", b"resp-3".to_vec(), false),
            (3, "headers", Vec::new(), true),
            (5, "headers", Vec::new(), true),
        ]
    );
    assert_eq!(client.stream_state(1), StreamState::Closed);
    assert_eq!(client.stream_state(3), StreamState::Closed);
    assert_eq!(client.stream_state(5), StreamState::Closed);
    assert_eq!(client.active_stream_count(), 0);
}

// ============================================================================
// 7. HEADERS + CONTINUATION Reassembly & Interleaving Rejection
// ============================================================================

#[test]
fn headers_continuation_reassembly_and_interleaved_frame_rejection() {
    let mut client = H2Connection::<4, 1024>::new_client();
    let mut server = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut hpack_buf = [0u8; 256];
    let hpack_len = HpackEncoder::with_huffman(true)
        .encode_grpc_request_headers(
            b"http",
            b"localhost:50051",
            b"/continuation.Test/Method",
            GRPC_CONTENT_TYPE_ROTO,
            &mut hpack_buf,
        )
        .unwrap();
    let block = &hpack_buf[..hpack_len];

    // Split `block` across HEADERS (without END_HEADERS, with PADDED + PRIORITY) + 2 CONTINUATION frames
    let mut hdr_payload = Vec::new();
    hdr_payload.push(3); // 3 bytes of padding
    hdr_payload.extend_from_slice(&0u32.to_be_bytes()); // stream dependency = 0
    hdr_payload.push(16); // weight = 16
    hdr_payload.extend_from_slice(&block[..5]);
    hdr_payload.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // padding

    let f1 = make_frame(
        FrameType::HEADERS,
        FLAG_PADDED | FLAG_PRIORITY | FLAG_END_STREAM,
        1,
        &hdr_payload,
    );
    let f2 = make_frame(FrameType::CONTINUATION, 0, 1, &block[5..15]);
    let f3 = make_frame(FrameType::CONTINUATION, FLAG_END_HEADERS, 1, &block[15..]);

    assert_eq!(server.recv(&f1).unwrap(), (f1.len(), None));
    assert_eq!(server.recv(&f2).unwrap(), (f2.len(), None));
    let (c3, ev3) = server.recv(&f3).unwrap();
    assert_eq!(c3, f3.len());
    let Some(H2Event::Headers {
        stream_id: 1,
        header_block,
        end_stream: true,
    }) = ev3
    else {
        panic!("expected reassembled Headers event, got {ev3:?}");
    };
    assert_eq!(header_block, block);

    let mut dec = HpackDecoder::<256>::new();
    let mut scratch = [0u8; 256];
    let req = GrpcRequestHeaders::decode(&mut dec, header_block, &mut scratch).unwrap();
    assert!(req.is_valid_grpc_post());
    assert_eq!(req.path_str(), Some("/continuation.Test/Method"));
    assert_eq!(server.stream_state(1), StreamState::HalfClosedRemote);

    // Interleaved PING during HEADERS..CONTINUATION must fail with ProtocolError
    let f_start = make_frame(FrameType::HEADERS, 0, 3, &block[..4]);
    let f_ping = make_frame(FrameType::PING, 0, 0, &[0u8; 8]);
    assert_eq!(server.recv(&f_start).unwrap(), (f_start.len(), None));
    let err = server.recv(&f_ping).unwrap_err();
    assert_eq!(err.code, H2ErrorCode::ProtocolError);

    // Unexpected CONTINUATION without preceding HEADERS must fail with ProtocolError
    let mut server2 = H2Connection::<4, 1024>::new_server();
    let mut client2 = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut client2, &mut server2);
    let stray_cont = make_frame(FrameType::CONTINUATION, FLAG_END_HEADERS, 1, &block[..4]);
    let err = server2.recv(&stray_cont).unwrap_err();
    assert_eq!(err.code, H2ErrorCode::ProtocolError);
}

// ============================================================================
// 8. Flow Control Window Tracking, Replenishment, and Overflow Rejection
// ============================================================================

#[test]
fn flow_control_window_tracking_replenishment_and_overflow_rejection() {
    let mut client = H2Connection::<4, 1024>::new_client();
    let mut server = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut tmp = [0u8; 256];
    let (sid, n) = client
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            b"/fc.Svc/Test",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut tmp,
        )
        .unwrap();
    assert_eq!(sid, 1);
    server.recv(&tmp[..n]).unwrap();

    assert_eq!(
        client.conn_send_window(),
        DEFAULT_INITIAL_WINDOW_SIZE as i64
    );
    assert_eq!(
        server.conn_recv_window(),
        DEFAULT_INITIAL_WINDOW_SIZE as i64
    );

    // Send 100-byte gRPC message (105 bytes with 5-byte LPM header)
    let payload_100 = [0x42u8; 100];
    let mut data_buf = [0u8; 256];
    let wire = client
        .send_grpc_data_frame(1, false, false, &mut data_buf, |w| {
            let _ = w.write_bytes(&payload_100);
        })
        .unwrap();
    assert_eq!(
        client.conn_send_window(),
        (DEFAULT_INITIAL_WINDOW_SIZE - 105) as i64
    );
    assert_eq!(
        client.stream_send_window(1),
        Some((DEFAULT_INITIAL_WINDOW_SIZE - 105) as i64)
    );

    let (c, ev) = server.recv(wire).unwrap();
    assert_eq!(c, wire.len());
    assert!(matches!(
        ev,
        Some(H2Event::GrpcMessage { stream_id: 1, .. })
    ));
    assert_eq!(
        server.conn_recv_window(),
        (DEFAULT_INITIAL_WINDOW_SIZE - 105) as i64
    );
    assert_eq!(
        server.stream_recv_window(1),
        Some((DEFAULT_INITIAL_WINDOW_SIZE - 105) as i64)
    );

    // Server replenishes 105 bytes on stream 1 (and connection 0)
    server.replenish_recv_window(1, 105).unwrap();
    assert_eq!(
        server.conn_recv_window(),
        DEFAULT_INITIAL_WINDOW_SIZE as i64
    );
    assert_eq!(
        server.stream_recv_window(1),
        Some(DEFAULT_INITIAL_WINDOW_SIZE as i64)
    );

    let mut wu_wire = [0u8; 64];
    let wu_len = server.poll_transmit(&mut wu_wire);
    assert_eq!(wu_len, (FRAME_HEADER_LEN + 4) * 2);

    // Client processes both WINDOW_UPDATE frames (stream 0 and stream 1)
    let (c1, ev1) = client.recv(&wu_wire[..wu_len]).unwrap();
    assert_eq!(
        ev1,
        Some(H2Event::WindowUpdate {
            stream_id: 0,
            increment: 105
        })
    );
    let (c2, ev2) = client.recv(&wu_wire[c1..wu_len]).unwrap();
    assert_eq!(c1 + c2, wu_len);
    assert_eq!(
        ev2,
        Some(H2Event::WindowUpdate {
            stream_id: 1,
            increment: 105
        })
    );
    assert_eq!(
        client.conn_send_window(),
        DEFAULT_INITIAL_WINDOW_SIZE as i64
    );
    assert_eq!(
        client.stream_send_window(1),
        Some(DEFAULT_INITIAL_WINDOW_SIZE as i64)
    );

    // Overflowing stream 1 send window (> 0x7FFF_FFFF) must return stream FlowControlError
    let overflow_wu = make_frame(
        FrameType::WINDOW_UPDATE,
        0,
        1,
        &MAX_FLOW_CONTROL_WINDOW.to_be_bytes(),
    );
    let (consumed, ev) = client.recv(&overflow_wu).unwrap();
    assert_eq!(consumed, overflow_wu.len());
    assert_eq!(
        ev,
        Some(H2Event::StreamError {
            stream_id: 1,
            error_code: H2ErrorCode::FlowControlError,
        })
    );

    // Overflowing connection send window (stream 0) must return connection FlowControlError
    let conn_overflow_wu = make_frame(
        FrameType::WINDOW_UPDATE,
        0,
        0,
        &MAX_FLOW_CONTROL_WINDOW.to_be_bytes(),
    );
    let err = client.recv(&conn_overflow_wu).unwrap_err();
    assert!(err.is_connection_error());
    assert_eq!(err.code, H2ErrorCode::FlowControlError);
}

// ============================================================================
// 9. RFC 9113 Protocol Error & Invalid Frame Rejections
// ============================================================================

#[test]
fn rfc9113_protocol_and_frame_size_error_rejections() {
    // Helper to create a handshaken server with an open stream 1
    let setup_server = || {
        let mut client = H2Connection::<4, 1024>::new_client();
        let mut server = H2Connection::<4, 1024>::new_server();
        complete_handshake(&mut client, &mut server);
        let mut tmp = [0u8; 128];
        let (_, n) = client
            .send_grpc_request_headers(
                b"http",
                b"localhost",
                b"/test.Svc/Call",
                GRPC_CONTENT_TYPE_ROTO,
                false,
                &mut tmp,
            )
            .unwrap();
        server.recv(&tmp[..n]).unwrap();
        server
    };

    // 1. DATA on stream 0 -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::DATA, 0, 0, &[0; 5]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );

    // 2. HEADERS on stream 0 -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(
            FrameType::HEADERS,
            FLAG_END_HEADERS,
            0,
            &[0x82]
        ))
        .unwrap_err()
        .code,
        H2ErrorCode::ProtocolError
    );

    // 3. Even stream ID (stream 2) from client -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(
            FrameType::HEADERS,
            FLAG_END_HEADERS,
            2,
            &[0x82]
        ))
        .unwrap_err()
        .code,
        H2ErrorCode::ProtocolError
    );

    // 4. Decreasing stream ID (open stream 5, then try opening stream 3) -> ProtocolError
    let mut s = setup_server();
    s.recv(&make_frame(
        FrameType::HEADERS,
        FLAG_END_HEADERS,
        5,
        &[0x82],
    ))
    .unwrap();
    assert_eq!(
        s.recv(&make_frame(
            FrameType::HEADERS,
            FLAG_END_HEADERS,
            3,
            &[0x82]
        ))
        .unwrap_err()
        .code,
        H2ErrorCode::ProtocolError
    );

    // 5. PUSH_PROMISE frame -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(
            FrameType::PUSH_PROMISE,
            FLAG_END_HEADERS,
            1,
            &[0, 0, 0, 2, 0x82]
        ))
        .unwrap_err()
        .code,
        H2ErrorCode::ProtocolError
    );

    // 6. SETTINGS with non-multiple-of-6 payload -> FrameSizeError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::SETTINGS, 0, 0, &[1, 2, 3, 4]))
            .unwrap_err()
            .code,
        H2ErrorCode::FrameSizeError
    );

    // 7. Non-empty SETTINGS ACK -> FrameSizeError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::SETTINGS, FLAG_ACK, 0, &[0; 6]))
            .unwrap_err()
            .code,
        H2ErrorCode::FrameSizeError
    );

    // 8. PING with length != 8 -> FrameSizeError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::PING, 0, 0, &[1, 2, 3]))
            .unwrap_err()
            .code,
        H2ErrorCode::FrameSizeError
    );

    // 9. WINDOW_UPDATE with increment 0 -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::WINDOW_UPDATE, 0, 0, &[0, 0, 0, 0]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );
    let mut s = setup_server();
    let zero_wu = make_frame(FrameType::WINDOW_UPDATE, 0, 1, &[0, 0, 0, 0]);
    let (consumed, ev) = s.recv(&zero_wu).unwrap();
    assert_eq!(consumed, zero_wu.len());
    assert_eq!(
        ev,
        Some(H2Event::StreamError {
            stream_id: 1,
            error_code: H2ErrorCode::ProtocolError,
        })
    );

    // 10. Invalid SETTINGS values:
    // - ENABLE_PUSH = 2 -> ProtocolError
    let mut s = setup_server();
    let mut bad_push = [0u8; 6];
    bad_push[..2].copy_from_slice(&SETTINGS_ENABLE_PUSH.to_be_bytes());
    bad_push[2..].copy_from_slice(&2u32.to_be_bytes());
    assert_eq!(
        s.recv(&make_frame(FrameType::SETTINGS, 0, 0, &bad_push))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );

    // - INITIAL_WINDOW_SIZE > 0x7FFF_FFFF -> FlowControlError
    let mut s = setup_server();
    let mut bad_win = [0u8; 6];
    bad_win[..2].copy_from_slice(&SETTINGS_INITIAL_WINDOW_SIZE.to_be_bytes());
    bad_win[2..].copy_from_slice(&0x8000_0000u32.to_be_bytes());
    assert_eq!(
        s.recv(&make_frame(FrameType::SETTINGS, 0, 0, &bad_win))
            .unwrap_err()
            .code,
        H2ErrorCode::FlowControlError
    );

    // - MAX_FRAME_SIZE < 16384 -> ProtocolError
    let mut s = setup_server();
    let mut bad_mfs = [0u8; 6];
    bad_mfs[..2].copy_from_slice(&SETTINGS_MAX_FRAME_SIZE.to_be_bytes());
    bad_mfs[2..].copy_from_slice(&1024u32.to_be_bytes());
    assert_eq!(
        s.recv(&make_frame(FrameType::SETTINGS, 0, 0, &bad_mfs))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );

    // 11. Invalid DATA padding (pad_len >= payload.len()) -> ProtocolError
    let mut s = setup_server();
    assert_eq!(
        s.recv(&make_frame(FrameType::DATA, FLAG_PADDED, 1, &[5, 1, 2]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );

    // 12. PRIORITY frame (length 5, stream 1) and unknown frame type (0xFE) are ignored cleanly
    let mut s = setup_server();
    let prio = make_frame(FrameType::PRIORITY, 0, 1, &[0, 0, 0, 0, 16]);
    let unknown = make_frame(0xFE, 0, 1, b"custom-extension-payload");
    assert_eq!(s.recv(&prio).unwrap(), (prio.len(), None));
    assert_eq!(s.recv(&unknown).unwrap(), (unknown.len(), None));

    // 13. Client receives DATA on an open stream BEFORE initial response HEADERS -> ProtocolError (RFC 9113 §8.1)
    let mut c = H2Connection::<4, 1024>::new_client();
    let mut s = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut c, &mut s);
    let mut tmp = [0u8; 128];
    let (sid, _) = c
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            b"/test.Svc/Call",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut tmp,
        )
        .unwrap();
    assert_eq!(sid, 1);
    let premature_data = make_frame(FrameType::DATA, 0, 1, &[0, 0, 0, 0, 0]);
    let err = c.recv(&premature_data).unwrap_err();
    assert_eq!(err.code, H2ErrorCode::ProtocolError);
}

// ============================================================================
// 9b. RST_STREAM, GOAWAY & Outbound HEADERS + CONTINUATION Splitting
// ============================================================================

#[test]
fn rst_stream_goaway_and_outbound_continuation_splitting() {
    let mut client = H2Connection::<4, 1024>::new_client();
    let mut server = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut tmp = [0u8; 256];
    for expected_sid in [1u32, 3] {
        let (sid, n) = client
            .send_grpc_request_headers(
                b"http",
                b"localhost",
                b"/rst.Svc/Call",
                GRPC_CONTENT_TYPE_ROTO,
                false,
                &mut tmp,
            )
            .unwrap();
        assert_eq!(sid, expected_sid);
        server.recv(&tmp[..n]).unwrap();
    }

    // 1. Outbound queue_rst_stream from client -> inbound H2Event::RstStream on server
    client.queue_rst_stream(1, H2ErrorCode::Cancel).unwrap();
    assert_eq!(client.stream_state(1), StreamState::Closed);
    let rst_len = client.poll_transmit(&mut tmp);
    assert_eq!(rst_len, FRAME_HEADER_LEN + 4);

    let (c, ev) = server.recv(&tmp[..rst_len]).unwrap();
    assert_eq!(c, rst_len);
    assert_eq!(
        ev,
        Some(H2Event::RstStream {
            stream_id: 1,
            error_code: H2ErrorCode::Cancel,
        })
    );
    assert_eq!(server.stream_state(1), StreamState::Closed);

    // 2. Inbound RST_STREAM with unknown error code (0xDEAD_BEEF) -> falls back to InternalError per RFC 9113 §7
    let unknown_rst = make_frame(FrameType::RST_STREAM, 0, 3, &0xDEAD_BEEFu32.to_be_bytes());
    let (c, ev) = server.recv(&unknown_rst).unwrap();
    assert_eq!(c, unknown_rst.len());
    assert_eq!(
        ev,
        Some(H2Event::RstStream {
            stream_id: 3,
            error_code: H2ErrorCode::InternalError,
        })
    );
    assert_eq!(server.stream_state(3), StreamState::Closed);

    // 3. RST_STREAM RFC 9113 rejections:
    // - Stream 0 -> ProtocolError
    assert_eq!(
        server
            .recv(&make_frame(FrameType::RST_STREAM, 0, 0, &[0, 0, 0, 8]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );
    // - Idle stream (stream 99 when last peer stream is 3) -> ProtocolError
    let mut s_idle = H2Connection::<4, 1024>::new_server();
    let mut c_idle = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut c_idle, &mut s_idle);
    assert_eq!(
        s_idle
            .recv(&make_frame(FrameType::RST_STREAM, 0, 1, &[0, 0, 0, 8]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );
    // - Length != 4 -> FrameSizeError
    let mut s_sz = H2Connection::<4, 1024>::new_server();
    let mut c_sz = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut c_sz, &mut s_sz);
    let (_, n) = c_sz
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            b"/rst.Svc/Call",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut tmp,
        )
        .unwrap();
    s_sz.recv(&tmp[..n]).unwrap();
    assert_eq!(
        s_sz.recv(&make_frame(FrameType::RST_STREAM, 0, 1, &[0, 0, 0]))
            .unwrap_err()
            .code,
        H2ErrorCode::FrameSizeError
    );

    // 4. Outbound queue_goaway & inbound GOAWAY with debug_data
    let mut s_go = H2Connection::<4, 1024>::new_server();
    let mut c_go = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut c_go, &mut s_go);
    s_go.queue_goaway(0, H2ErrorCode::NoError, b"graceful")
        .unwrap();
    let go_len = s_go.poll_transmit(&mut tmp);
    assert_eq!(go_len, FRAME_HEADER_LEN + 8 + 8);
    let (c, ev) = c_go.recv(&tmp[..go_len]).unwrap();
    assert_eq!(c, go_len);
    assert_eq!(
        ev,
        Some(H2Event::GoAway {
            last_stream_id: 0,
            error_code: H2ErrorCode::NoError,
            debug_data: b"graceful",
        })
    );
    assert_eq!(c_go.goaway_received(), Some((0, H2ErrorCode::NoError)));
    // Client cannot open new streams after GOAWAY is received
    assert_eq!(
        c_go.open_stream().unwrap_err().code,
        H2ErrorCode::ProtocolError
    );

    // Inbound GOAWAY with non-empty debug_data
    let mut go_payload = Vec::new();
    go_payload.extend_from_slice(&3u32.to_be_bytes());
    go_payload.extend_from_slice(&(H2ErrorCode::EnhanceYourCalm as u32).to_be_bytes());
    go_payload.extend_from_slice(b"rate-limit-exceeded");
    let go_frame = make_frame(FrameType::GOAWAY, 0, 0, &go_payload);
    let (c, ev) = s_go.recv(&go_frame).unwrap();
    assert_eq!(c, go_frame.len());
    assert_eq!(
        ev,
        Some(H2Event::GoAway {
            last_stream_id: 3,
            error_code: H2ErrorCode::EnhanceYourCalm,
            debug_data: b"rate-limit-exceeded",
        })
    );

    // GOAWAY rejections: non-zero stream -> ProtocolError, length < 8 -> FrameSizeError
    let mut s_bad_go = H2Connection::<4, 1024>::new_server();
    let mut c_bad_go = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut c_bad_go, &mut s_bad_go);
    assert_eq!(
        s_bad_go
            .recv(&make_frame(FrameType::GOAWAY, 0, 1, &[0; 8]))
            .unwrap_err()
            .code,
        H2ErrorCode::ProtocolError
    );
    let mut s_short_go = H2Connection::<4, 1024>::new_server();
    let mut c_short_go = H2Connection::<4, 1024>::new_client();
    complete_handshake(&mut c_short_go, &mut s_short_go);
    assert_eq!(
        s_short_go
            .recv(&make_frame(FrameType::GOAWAY, 0, 0, &[0; 7]))
            .unwrap_err()
            .code,
        H2ErrorCode::FrameSizeError
    );

    // 5. Outbound send_headers_frame auto-splitting into HEADERS + CONTINUATION when > max_frame_size (16384)
    let mut c_big = H2Connection::<4, 1024>::new_client();
    let mut s_big = H2Connection::<4, 1024>::new_server();
    complete_handshake(&mut c_big, &mut s_big);
    let sid = c_big.open_stream().unwrap();
    let big_block = vec![0x82u8; 17_000];
    let mut big_out = vec![0u8; 18_000];
    let written = c_big
        .send_headers_frame(sid, &big_block, true, &mut big_out)
        .unwrap();
    assert_eq!(written, FRAME_HEADER_LEN * 2 + big_block.len());
    let first_hdr = FrameHeader::decode_slice(&big_out[..FRAME_HEADER_LEN]).unwrap();
    assert_eq!(first_hdr.frame_type, FrameType::HEADERS);
    assert_eq!(first_hdr.length, 16_384);
    assert_eq!(first_hdr.flags, FLAG_END_STREAM); // FLAG_END_HEADERS is on CONTINUATION
    assert_eq!(first_hdr.stream_id, sid);

    let second_offset = FRAME_HEADER_LEN + 16_384;
    let second_hdr =
        FrameHeader::decode_slice(&big_out[second_offset..second_offset + FRAME_HEADER_LEN])
            .unwrap();
    assert_eq!(second_hdr.frame_type, FrameType::CONTINUATION);
    assert_eq!(second_hdr.length, (17_000 - 16_384) as u32);
    assert_eq!(second_hdr.flags, FLAG_END_HEADERS);
    assert_eq!(second_hdr.stream_id, sid);

    // Verify H2Error::from_grpc_status helper
    let mapped = roto_grpc::h2::H2Error::from_grpc_status(
        sid,
        GrpcStatus::resource_exhausted("quota exceeded"),
    );
    assert!(mapped.is_stream_error());
    assert_eq!(mapped.stream_id, sid);
    assert_eq!(mapped.code, H2ErrorCode::EnhanceYourCalm);
}

#[test]
fn stream_error_recovery_preserves_connection_and_subsequent_streams() {
    // MAX_STREAMS = 1, REASM_CAP = 32
    let mut client = H2Connection::<1, 32>::new_client();
    let mut server = H2Connection::<1, 32>::new_server();
    complete_handshake(&mut client, &mut server);

    let mut tmp = [0u8; 256];

    // 1. Open stream 1 on server (fills the single stream slot).
    let (sid1, n) = client
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            b"/test.Svc/One",
            GRPC_CONTENT_TYPE_ROTO,
            false,
            &mut tmp,
        )
        .unwrap();
    assert_eq!(sid1, 1);
    let (c, ev) = server.recv(&tmp[..n]).unwrap();
    assert_eq!(c, n);
    assert!(matches!(ev, Some(H2Event::Headers { stream_id: 1, .. })));

    // 2. While stream 1 is active, peer opens stream 3 with HEADERS (no END_HEADERS) + CONTINUATION (END_HEADERS).
    // Server must emit StreamError(RefusedStream) on HEADERS, discard the CONTINUATION cleanly, and queue RST_STREAM.
    let refused_hdr = make_frame(FrameType::HEADERS, 0, 3, &[0x82]);
    let refused_cont = make_frame(FrameType::CONTINUATION, FLAG_END_HEADERS, 3, &[0x86]);
    let (c, ev) = server.recv(&refused_hdr).unwrap();
    assert_eq!(c, refused_hdr.len());
    assert_eq!(
        ev,
        Some(H2Event::StreamError {
            stream_id: 3,
            error_code: H2ErrorCode::RefusedStream,
        })
    );
    let (c, ev) = server.recv(&refused_cont).unwrap();
    assert_eq!(c, refused_cont.len());
    assert_eq!(ev, None);
    let rst_len = server.poll_transmit(&mut tmp);
    assert_eq!(rst_len, FRAME_HEADER_LEN + 4);

    // 3. DATA on closed/refused stream 3 fed in two chunks: must emit StreamError(StreamClosed),
    // account conn_recv_window, and skip the remaining payload chunk without desynchronizing.
    let before_conn_win = server.conn_recv_window();
    let closed_data = make_frame(FrameType::DATA, FLAG_END_STREAM, 3, &[0xAA; 12]);
    let split = FRAME_HEADER_LEN + 4;
    let (c1, ev1) = server.recv(&closed_data[..split]).unwrap();
    assert_eq!(c1, split);
    assert_eq!(
        ev1,
        Some(H2Event::StreamError {
            stream_id: 3,
            error_code: H2ErrorCode::StreamClosed,
        })
    );
    assert_eq!(server.conn_recv_window(), before_conn_win - 12);
    let (c2, ev2) = server.recv(&closed_data[split..]).unwrap();
    assert_eq!(c2, closed_data.len() - split);
    assert_eq!(ev2, None);
    let _ = server.poll_transmit(&mut tmp);

    // 4. Stream 1 sends a gRPC message whose declared LPM length (40 > REASM_CAP = 32) triggers
    // StreamError(EnhanceYourCalm) mid-frame, followed by trailing bytes in a second chunk.
    let mut oversized_lpm = vec![0u8; 5 + 40];
    oversized_lpm[1..5].copy_from_slice(&40u32.to_be_bytes());
    let oversized_data = make_frame(FrameType::DATA, FLAG_END_STREAM, 1, &oversized_lpm);
    let split_ov = FRAME_HEADER_LEN + 10;
    let (c1, ev1) = server.recv(&oversized_data[..split_ov]).unwrap();
    assert_eq!(c1, split_ov);
    assert_eq!(
        ev1,
        Some(H2Event::StreamError {
            stream_id: 1,
            error_code: H2ErrorCode::EnhanceYourCalm,
        })
    );
    let (c2, ev2) = server.recv(&oversized_data[split_ov..]).unwrap();
    assert_eq!(c2, oversized_data.len() - split_ov);
    assert_eq!(ev2, None);
    let _ = server.poll_transmit(&mut tmp);
    assert_eq!(server.active_stream_count(), 0);

    // 5. Subsequent valid stream 5 on the same connection succeeds end-to-end.
    let mut hpack_buf = [0u8; 128];
    let hpack_len = HpackEncoder::with_huffman(true)
        .encode_grpc_request_headers(
            b"http",
            b"localhost",
            b"/test.Svc/Five",
            GRPC_CONTENT_TYPE_ROTO,
            &mut hpack_buf,
        )
        .unwrap();
    let hdr5 = make_frame(
        FrameType::HEADERS,
        FLAG_END_HEADERS,
        5,
        &hpack_buf[..hpack_len],
    );
    let (c, ev) = server.recv(&hdr5).unwrap();
    assert_eq!(c, hdr5.len());
    assert!(matches!(ev, Some(H2Event::Headers { stream_id: 5, .. })));

    let mut lpm5_buf = [0u8; 32];
    let lpm5 = encode_grpc_message(&mut lpm5_buf, false, |w| {
        let _ = w.write_bytes(b"ok-after-errors");
    })
    .unwrap();
    let data5 = make_frame(FrameType::DATA, FLAG_END_STREAM, 5, lpm5);
    let (c, ev) = server.recv(&data5).unwrap();
    assert_eq!(c, data5.len());
    assert_eq!(
        ev,
        Some(H2Event::GrpcMessage {
            stream_id: 5,
            compressed: false,
            payload: b"ok-after-errors",
            end_stream: true,
        })
    );
}

// ============================================================================
// 10. Deterministic SplitMix64 Fuzzer
// ============================================================================

#[test]
fn fuzz_h2_arbitrary_and_structured_streams() {
    let seed = fuzz_seed();
    let iters = fuzz_iters(600);
    let mut rng = SplitMix64::new(seed);

    for iter in 0..iters {
        match iter % 3 {
            0 => {
                // Mode 0: Arbitrary byte streams fed in random chunk sizes after optional preface
                let mut conn = if rng.next_bool() {
                    H2Connection::<4, 512>::new_server()
                } else {
                    H2Connection::<4, 512>::new_client()
                };
                let mut wire = Vec::new();
                if conn.role() == Role::Server && rng.next_bool() {
                    wire.extend_from_slice(CLIENT_CONNECTION_PREFACE);
                    if rng.next_bool() {
                        wire.extend_from_slice(&make_frame(FrameType::SETTINGS, 0, 0, &[]));
                    }
                } else if conn.role() == Role::Client && rng.next_bool() {
                    wire.extend_from_slice(&make_frame(FrameType::SETTINGS, 0, 0, &[]));
                }
                let random_len = rng.next_usize(256);
                let start = wire.len();
                wire.resize(start + random_len, 0);
                rng.fill_bytes(&mut wire[start..]);

                let mut pos = 0;
                let mut out_drain = [0u8; 128];
                while pos < wire.len() {
                    let chunk_len = 1 + rng.next_usize((wire.len() - pos).min(32));
                    match conn.recv(&wire[pos..pos + chunk_len]) {
                        Ok((consumed, _ev)) => {
                            let _ = conn.poll_transmit(&mut out_drain);
                            if consumed == 0 {
                                break;
                            }
                            pos += consumed;
                        }
                        Err(_) => {
                            let _ = conn.poll_transmit(&mut out_drain);
                            break;
                        }
                    }
                }
            }
            1 => {
                // Mode 1: Semi-structured mutated HTTP/2 frame sequences against both Server and Client
                let mut server = H2Connection::<4, 512>::new_server();
                let mut client = H2Connection::<4, 512>::new_client();
                complete_handshake(&mut client, &mut server);

                let target_is_client = rng.next_bool();
                if target_is_client {
                    // Open 1..=2 client streams so client state machine is exercised on active stream IDs 1, 3
                    let mut tmp = [0u8; 128];
                    let open_cnt = 1 + rng.next_usize(2);
                    for _ in 0..open_cnt {
                        let _ = client.send_grpc_request_headers(
                            b"http",
                            b"fuzz.local",
                            b"/fuzz.Svc/Method",
                            GRPC_CONTENT_TYPE_ROTO,
                            rng.next_bool(),
                            &mut tmp,
                        );
                    }
                }

                let num_frames = 1 + rng.next_usize(16);
                let mut wire = Vec::new();
                for _ in 0..num_frames {
                    let ftype = match rng.next_usize(12) {
                        0 => FrameType::DATA,
                        1 => FrameType::HEADERS,
                        2 => FrameType::PRIORITY,
                        3 => FrameType::RST_STREAM,
                        4 => FrameType::SETTINGS,
                        5 => FrameType::PUSH_PROMISE,
                        6 => FrameType::PING,
                        7 => FrameType::GOAWAY,
                        8 => FrameType::WINDOW_UPDATE,
                        9 => FrameType::CONTINUATION,
                        _ => (rng.next_u32() & 0xFF) as u8,
                    };
                    let flags = (rng.next_u32() & 0xFF) as u8;
                    let stream_id = match rng.next_usize(6) {
                        0 => 0,
                        1 => 1,
                        2 => 3,
                        3 => 5,
                        4 => 2,
                        _ => rng.next_u32() & MAX_STREAM_ID,
                    };
                    let payload_len = rng.next_usize(48);
                    let mut payload = vec![0u8; payload_len];
                    rng.fill_bytes(&mut payload);
                    wire.extend_from_slice(&make_frame(ftype, flags, stream_id, &payload));
                }

                let target = if target_is_client {
                    &mut client
                } else {
                    &mut server
                };
                let mut pos = 0;
                let mut tx_drain = [0u8; 128];
                while pos < wire.len() {
                    let step = 1 + rng.next_usize((wire.len() - pos).min(64));
                    match target.recv(&wire[pos..pos + step]) {
                        Ok((n, _ev)) => {
                            let _ = target.poll_transmit(&mut tx_drain);
                            if n == 0 {
                                break;
                            }
                            pos += n;
                        }
                        Err(_) => {
                            let _ = target.poll_transmit(&mut tx_drain);
                            break;
                        }
                    }
                }
            }
            _ => {
                // Mode 2: Valid randomized multiplexed gRPC round-trip under random chunking
                let mut client = H2Connection::<8, 1024>::new_client();
                let mut server = H2Connection::<8, 1024>::new_server();
                server.set_auto_window_update(true);
                complete_handshake(&mut client, &mut server);

                let stream_count = 1 + rng.next_usize(4);
                let mut c2s_wire = Vec::new();
                let mut expected_msgs = Vec::new();
                let mut tmp = [0u8; 256];

                for _ in 0..stream_count {
                    let (sid, n) = client
                        .send_grpc_request_headers(
                            b"http",
                            b"fuzz.local",
                            b"/fuzz.Service/Call",
                            GRPC_CONTENT_TYPE_ROTO,
                            false,
                            &mut tmp,
                        )
                        .unwrap();
                    c2s_wire.extend_from_slice(&tmp[..n]);

                    let msg_len = rng.next_usize(64);
                    let mut msg = vec![0u8; msg_len];
                    rng.fill_bytes(&mut msg);
                    let compressed = rng.next_bool();

                    let mut lpm_buf = [0u8; 128];
                    let lpm = encode_grpc_message(&mut lpm_buf, compressed, |w| {
                        let _ = w.write_bytes(&msg);
                    })
                    .unwrap();

                    // Randomly split into 1 or 2 DATA frames
                    if rng.next_bool() && lpm.len() > 2 {
                        let split = 1 + rng.next_usize(lpm.len() - 1);
                        c2s_wire.extend_from_slice(&make_frame(
                            FrameType::DATA,
                            0,
                            sid,
                            &lpm[..split],
                        ));
                        c2s_wire.extend_from_slice(&make_frame(
                            FrameType::DATA,
                            FLAG_END_STREAM,
                            sid,
                            &lpm[split..],
                        ));
                    } else {
                        c2s_wire.extend_from_slice(&make_frame(
                            FrameType::DATA,
                            FLAG_END_STREAM,
                            sid,
                            lpm,
                        ));
                    }
                    expected_msgs.push((sid, compressed, msg));
                }

                // Feed c2s_wire into server in random chunk sizes (1..=37 bytes)
                let mut pos = 0;
                let mut actual_msgs = Vec::new();
                while pos < c2s_wire.len() {
                    let chunk = 1 + rng.next_usize((c2s_wire.len() - pos).min(37));
                    let (n, ev) = server.recv(&c2s_wire[pos..pos + chunk]).unwrap();
                    assert!(n > 0);
                    pos += n;
                    if let Some(H2Event::GrpcMessage {
                        stream_id,
                        compressed,
                        payload,
                        end_stream,
                    }) = ev
                    {
                        assert!(end_stream);
                        actual_msgs.push((stream_id, compressed, payload.to_vec()));
                    }
                    let mut drain = [0u8; 128];
                    let _ = server.poll_transmit(&mut drain);
                }
                assert_eq!(actual_msgs, expected_msgs);
            }
        }
    }
}
