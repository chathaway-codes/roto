use google_protobuf_roto::google::protobuf::compiler::{
    CodeGeneratorRequestBuilder, CodeGeneratorRequestView, VersionBuilder, VersionView,
};
use roto::runtime::{BackwardWriter, DecodeError, EncodeError};
use roto_grpc::framing::{
    decode_grpc_frame, encode_grpc_message, iter_grpc_frames, write_grpc_frame,
    write_grpc_frame_bytes, GrpcFrame, GrpcFrameIter, LpmDecoder, DEFAULT_MAX_MESSAGE_SIZE,
    GRPC_HEADER_LEN,
};
use roto_grpc::status::{
    is_unreserved_grpc_message_byte, percent_decode, percent_encode, percent_encode_backward,
    percent_encoded_len, try_percent_decode_in_place, GrpcCode, GrpcStatus,
};

#[test]
fn grpc_code_all_17_canonical_round_trips_and_rejections() {
    let cases = [
        (0u8, GrpcCode::Ok, "0", "OK"),
        (1, GrpcCode::Cancelled, "1", "CANCELLED"),
        (2, GrpcCode::Unknown, "2", "UNKNOWN"),
        (3, GrpcCode::InvalidArgument, "3", "INVALID_ARGUMENT"),
        (4, GrpcCode::DeadlineExceeded, "4", "DEADLINE_EXCEEDED"),
        (5, GrpcCode::NotFound, "5", "NOT_FOUND"),
        (6, GrpcCode::AlreadyExists, "6", "ALREADY_EXISTS"),
        (7, GrpcCode::PermissionDenied, "7", "PERMISSION_DENIED"),
        (8, GrpcCode::ResourceExhausted, "8", "RESOURCE_EXHAUSTED"),
        (9, GrpcCode::FailedPrecondition, "9", "FAILED_PRECONDITION"),
        (10, GrpcCode::Aborted, "10", "ABORTED"),
        (11, GrpcCode::OutOfRange, "11", "OUT_OF_RANGE"),
        (12, GrpcCode::Unimplemented, "12", "UNIMPLEMENTED"),
        (13, GrpcCode::Internal, "13", "INTERNAL"),
        (14, GrpcCode::Unavailable, "14", "UNAVAILABLE"),
        (15, GrpcCode::DataLoss, "15", "DATA_LOSS"),
        (16, GrpcCode::Unauthenticated, "16", "UNAUTHENTICATED"),
    ];

    for (raw, code, dec_str, name_str) in cases {
        assert_eq!(GrpcCode::from_u8(raw), Some(code));
        assert_eq!(GrpcCode::try_from(raw), Ok(code));
        assert_eq!(code.as_u8(), raw);
        assert_eq!(code.as_str(), dec_str);
        assert_eq!(code.name(), name_str);
        assert_eq!(code.to_string(), name_str);
        assert_eq!(GrpcCode::from_ascii_bytes(dec_str.as_bytes()), Some(code));
        assert_eq!(GrpcCode::from_header_str(dec_str), Some(code));
    }

    for invalid_u8 in 17..=u8::MAX {
        assert_eq!(GrpcCode::from_u8(invalid_u8), None);
        assert_eq!(GrpcCode::try_from(invalid_u8), Err(invalid_u8));
    }

    let invalid_strings = [
        "", "00", "01", "09", "010", "17", "18", "99", "999", "-1", "+0", "+1", " 0", "0 ", "1\n",
        "OK", "a", "1a",
    ];
    for s in invalid_strings {
        assert_eq!(
            GrpcCode::from_header_str(s),
            None,
            "expected None for header string {s:?}"
        );
        assert_eq!(
            GrpcCode::from_ascii_bytes(s.as_bytes()),
            None,
            "expected None for ascii bytes {s:?}"
        );
    }
}

#[test]
fn grpc_status_constructors_display_and_conversions() {
    fn assert_core_error<E: core::error::Error>(_: &E) {}

    let ok = GrpcStatus::ok();
    assert!(ok.is_ok());
    assert_eq!(ok.code, GrpcCode::Ok);
    assert_eq!(ok.message, "");
    assert_eq!(ok.to_string(), "grpc-status: OK (0)");
    assert_core_error(&ok);

    let statuses = [
        (
            GrpcStatus::new(GrpcCode::Ok, "all good"),
            GrpcCode::Ok,
            "all good",
            true,
        ),
        (
            GrpcStatus::cancelled("op cancelled"),
            GrpcCode::Cancelled,
            "op cancelled",
            false,
        ),
        (
            GrpcStatus::unknown("mystery"),
            GrpcCode::Unknown,
            "mystery",
            false,
        ),
        (
            GrpcStatus::invalid_argument("bad arg"),
            GrpcCode::InvalidArgument,
            "bad arg",
            false,
        ),
        (
            GrpcStatus::deadline_exceeded("timeout"),
            GrpcCode::DeadlineExceeded,
            "timeout",
            false,
        ),
        (
            GrpcStatus::not_found("missing"),
            GrpcCode::NotFound,
            "missing",
            false,
        ),
        (
            GrpcStatus::already_exists("duplicate"),
            GrpcCode::AlreadyExists,
            "duplicate",
            false,
        ),
        (
            GrpcStatus::permission_denied("forbidden"),
            GrpcCode::PermissionDenied,
            "forbidden",
            false,
        ),
        (
            GrpcStatus::resource_exhausted("oom"),
            GrpcCode::ResourceExhausted,
            "oom",
            false,
        ),
        (
            GrpcStatus::failed_precondition("wrong state"),
            GrpcCode::FailedPrecondition,
            "wrong state",
            false,
        ),
        (
            GrpcStatus::aborted("conflict"),
            GrpcCode::Aborted,
            "conflict",
            false,
        ),
        (
            GrpcStatus::out_of_range("oob"),
            GrpcCode::OutOfRange,
            "oob",
            false,
        ),
        (
            GrpcStatus::unimplemented("todo"),
            GrpcCode::Unimplemented,
            "todo",
            false,
        ),
        (
            GrpcStatus::internal("bug"),
            GrpcCode::Internal,
            "bug",
            false,
        ),
        (
            GrpcStatus::unavailable("down"),
            GrpcCode::Unavailable,
            "down",
            false,
        ),
        (
            GrpcStatus::data_loss("corrupt"),
            GrpcCode::DataLoss,
            "corrupt",
            false,
        ),
        (
            GrpcStatus::unauthenticated("no token"),
            GrpcCode::Unauthenticated,
            "no token",
            false,
        ),
    ];

    for (status, expected_code, expected_msg, expected_ok) in statuses {
        assert_eq!(status.code, expected_code);
        assert_eq!(status.message, expected_msg);
        assert_eq!(status.is_ok(), expected_ok);
        assert_eq!(
            status.to_string(),
            format!(
                "grpc-status: {} ({}): {}",
                expected_code.name(),
                expected_code.as_u8(),
                expected_msg
            )
        );
    }

    let from_decode: GrpcStatus<'static> = DecodeError::UnexpectedEof.into();
    assert_eq!(from_decode.code, GrpcCode::Internal);
    assert!(!from_decode.message.is_empty());

    let from_encode: GrpcStatus<'static> = EncodeError::BufferOverflow.into();
    assert_eq!(from_encode.code, GrpcCode::ResourceExhausted);
    assert!(!from_encode.message.is_empty());
}

#[test]
fn percent_encoding_and_decoding_comprehensive() {
    // Verify unreserved byte classification per gRPC HTTP/2 spec
    for b in 0u8..=u8::MAX {
        let expected = (0x20..=0x7E).contains(&b) && b != b'%';
        assert_eq!(is_unreserved_grpc_message_byte(b), expected);
    }

    let vectors = [
        ("", ""),
        ("hello world! ~_-.()", "hello world! ~_-.()"),
        ("100% done", "100%25 done"),
        ("%%%", "%25%25%25"),
        ("line1\r\nline2\t\0\x7f", "line1%0D%0Aline2%09%00%7F"),
        ("é", "%C3%A9"),
        (
            "你好，世界",
            "%E4%BD%A0%E5%A5%BD%EF%BC%8C%E4%B8%96%E7%95%8C",
        ),
        ("🚀🔥gRPC%", "%F0%9F%9A%80%F0%9F%94%A5gRPC%25"),
    ];

    for (raw, expected_encoded) in vectors {
        assert_eq!(percent_encoded_len(raw), expected_encoded.len());

        // Forward percent_encode into exact-sized buffer
        let mut enc_buf = vec![0u8; expected_encoded.len()];
        let encoded = percent_encode(raw, &mut enc_buf).unwrap();
        assert_eq!(encoded, expected_encoded);

        // Backward percent_encode_backward into BackwardWriter
        let mut bw_buf = vec![0u8; expected_encoded.len() + 8];
        let mut writer = BackwardWriter::new(&mut bw_buf);
        percent_encode_backward(raw, &mut writer);
        let bw_bytes = writer.finish().unwrap();
        assert_eq!(bw_bytes, expected_encoded.as_bytes());

        // Fast-path in-place decode works iff there are no '%' bytes
        if !expected_encoded.contains('%') {
            assert_eq!(
                try_percent_decode_in_place(expected_encoded.as_bytes()),
                Some(raw)
            );
        } else {
            assert_eq!(
                try_percent_decode_in_place(expected_encoded.as_bytes()),
                None
            );
        }

        // Decode into exact-sized output buffer
        let mut dec_buf = vec![0u8; raw.len()];
        let decoded = percent_decode(expected_encoded.as_bytes(), &mut dec_buf).unwrap();
        assert_eq!(decoded, raw);

        // Decode lowercase and mixed-case hex escapes
        let mut lower_hex = String::with_capacity(expected_encoded.len());
        let eb = expected_encoded.as_bytes();
        let mut idx = 0;
        while idx < eb.len() {
            if eb[idx] == b'%' && idx + 2 < eb.len() {
                lower_hex.push('%');
                lower_hex.push((eb[idx + 1] as char).to_ascii_lowercase());
                lower_hex.push((eb[idx + 2] as char).to_ascii_lowercase());
                idx += 3;
            } else {
                lower_hex.push(eb[idx] as char);
                idx += 1;
            }
        }
        let mut dec_lower_buf = vec![0u8; raw.len()];
        let decoded_lower = percent_decode(lower_hex.as_bytes(), &mut dec_lower_buf).unwrap();
        assert_eq!(decoded_lower, raw);
    }

    // Buffer overflow checks on percent_encode and percent_encode_backward
    let sample = "a%b"; // encodes to "a%25b" (5 bytes)
    for cap in 0..5 {
        let mut small = vec![0u8; cap];
        assert_eq!(
            percent_encode(sample, &mut small),
            Err(EncodeError::BufferOverflow)
        );

        let mut bw_small = vec![0u8; cap];
        let mut writer = BackwardWriter::new(&mut bw_small);
        percent_encode_backward(sample, &mut writer);
        assert_eq!(writer.check(), Err(EncodeError::BufferOverflow));
        assert_eq!(writer.finish(), Err(EncodeError::BufferOverflow));
    }

    // Buffer overflow on percent_decode when output slice is too small
    let mut tiny_out = [0u8; 2];
    assert_eq!(
        percent_decode(b"abc", &mut tiny_out),
        Err(DecodeError::UnexpectedEof)
    );
    assert_eq!(
        percent_decode(b"ab%25", &mut tiny_out),
        Err(DecodeError::UnexpectedEof)
    );

    // Fast-path in-place decode returns None on invalid UTF-8 even when no '%' is present
    assert_eq!(try_percent_decode_in_place(&[0xFF, 0xFE]), None);
    assert_eq!(try_percent_decode_in_place(&[0xC3, 0x28]), None);

    // Malformed % escapes and invalid UTF-8 sequences
    let invalid_inputs: &[&[u8]] = &[
        b"%",
        b"%4",
        b"abc%",
        b"abc%A",
        b"%ZZ",
        b"%0G",
        b"%G0",
        b"%/0",
        b"%:0",
        b"%FF",
        b"%FF%FF",
        b"%C3%28",
        b"%80",
        &[0xFF],
    ];
    for &bad in invalid_inputs {
        let mut out = [0u8; 32];
        assert_eq!(
            percent_decode(bad, &mut out),
            Err(DecodeError::InvalidUtf8),
            "expected InvalidUtf8 for input {bad:?}"
        );
    }
}

#[test]
fn lpm_framing_with_generated_roto_builders_and_views() {
    // 1. Single frame via encode_grpc_message with VersionBuilder
    let mut buf = [0u8; 128];
    let frame_bytes = encode_grpc_message(&mut buf, false, |w| {
        w.with_sub_writer(|sub| {
            let mut vb = VersionBuilder::from_writer(sub);
            vb.write_suffix("rc2")
                .write_patch(3)
                .write_minor(29)
                .write_major(5);
            vb.writer
        });
    })
    .unwrap();

    assert_eq!(frame_bytes[0], 0);
    let payload_len = u32::from_be_bytes([
        frame_bytes[1],
        frame_bytes[2],
        frame_bytes[3],
        frame_bytes[4],
    ]) as usize;
    assert_eq!(frame_bytes.len(), GRPC_HEADER_LEN + payload_len);

    let (decoded_frame, rem) = decode_grpc_frame(frame_bytes, DEFAULT_MAX_MESSAGE_SIZE)
        .unwrap()
        .unwrap();
    assert!(!decoded_frame.compressed);
    assert!(rem.is_empty());

    let ver = VersionView::from_bytes(decoded_frame.payload).unwrap();
    assert_eq!(ver.major(), Ok(5));
    assert_eq!(ver.minor(), Ok(29));
    assert_eq!(ver.patch(), Ok(3));
    assert_eq!(ver.suffix(), Ok("rc2"));

    // 2. Empty message (0-byte payload) with compressed = false and compressed = true
    let mut empty_buf = [0u8; 16];
    let empty_uncompressed = encode_grpc_message(&mut empty_buf, false, |_w| {}).unwrap();
    assert_eq!(empty_uncompressed, &[0, 0, 0, 0, 0]);
    let (ef, erem) = decode_grpc_frame(empty_uncompressed, DEFAULT_MAX_MESSAGE_SIZE)
        .unwrap()
        .unwrap();
    assert_eq!(
        ef,
        GrpcFrame {
            compressed: false,
            payload: &[]
        }
    );
    assert!(erem.is_empty());

    let empty_compressed = encode_grpc_message(&mut empty_buf, true, |_w| {}).unwrap();
    assert_eq!(empty_compressed, &[1, 0, 0, 0, 0]);
    let (ecf, ecrem) = decode_grpc_frame(empty_compressed, DEFAULT_MAX_MESSAGE_SIZE)
        .unwrap()
        .unwrap();
    assert_eq!(
        ecf,
        GrpcFrame {
            compressed: true,
            payload: &[]
        }
    );
    assert!(ecrem.is_empty());

    // 3. Multiple sequential frames written in a single BackwardWriter pass
    let mut multi_buf = [0u8; 256];
    let mut writer = BackwardWriter::new(&mut multi_buf);

    // Since BackwardWriter writes from tail to front, writing Frame 3, then Frame 2, then Frame 1
    // places Frame 1 first on the wire, followed by Frame 2, followed by Frame 3.
    write_grpc_frame(&mut writer, false, |w| {
        w.with_sub_writer(|sub| {
            let mut req = CodeGeneratorRequestBuilder::from_writer(sub);
            req.write_parameter("opt=3")
                .write_file_to_generate("c.proto");
            req.writer
        });
    })
    .unwrap();

    write_grpc_frame_bytes(&mut writer, true, &[]).unwrap();

    write_grpc_frame(&mut writer, false, |w| {
        w.with_sub_writer(|sub| {
            let mut req = CodeGeneratorRequestBuilder::from_writer(sub);
            req.write_parameter("opt=1")
                .write_file_to_generate("a.proto");
            req.writer
        });
    })
    .unwrap();

    let stream_bytes = writer.finish().unwrap();

    let frames: Vec<GrpcFrame<'_>> = iter_grpc_frames(stream_bytes, DEFAULT_MAX_MESSAGE_SIZE)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(frames.len(), 3);

    assert!(!frames[0].compressed);
    let req1 = CodeGeneratorRequestView::from_bytes(frames[0].payload).unwrap();
    assert_eq!(req1.parameter(), Ok("opt=1"));
    assert_eq!(
        req1.file_to_generate()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        vec!["a.proto"]
    );

    assert!(frames[1].compressed);
    assert!(frames[1].payload.is_empty());

    assert!(!frames[2].compressed);
    let req3 = CodeGeneratorRequestView::from_bytes(frames[2].payload).unwrap();
    assert_eq!(req3.parameter(), Ok("opt=3"));
    assert_eq!(
        req3.file_to_generate()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        vec!["c.proto"]
    );
}

#[test]
fn write_grpc_frame_buffer_overflow_for_payload_and_header() {
    let payload = [0x11, 0x22, 0x33, 0x44]; // 4 bytes payload + 5 bytes header = 9 bytes total

    // Case A: Buffer too small for the payload itself (0..=3 bytes)
    for cap in 0..4 {
        let mut buf = vec![0u8; cap];
        let mut writer = BackwardWriter::new(&mut buf);
        assert_eq!(
            write_grpc_frame_bytes(&mut writer, false, &payload),
            Err(EncodeError::BufferOverflow)
        );
        assert_eq!(writer.check(), Err(EncodeError::BufferOverflow));
    }

    // Case B: Buffer fits the 4-byte payload, but has only 0..=4 bytes left for the 5-byte LPM header
    for cap in 4..9 {
        let mut buf = vec![0u8; cap];
        let mut writer = BackwardWriter::new(&mut buf);
        assert_eq!(
            write_grpc_frame_bytes(&mut writer, false, &payload),
            Err(EncodeError::BufferOverflow)
        );
        assert_eq!(writer.check(), Err(EncodeError::BufferOverflow));
    }

    // Case C: Exact 9-byte buffer succeeds
    let mut exact = [0u8; 9];
    let mut writer = BackwardWriter::new(&mut exact);
    write_grpc_frame_bytes(&mut writer, true, &payload).unwrap();
    let encoded = writer.finish().unwrap();
    assert_eq!(encoded, &[1, 0, 0, 0, 4, 0x11, 0x22, 0x33, 0x44]);
}

#[test]
fn decode_grpc_frame_and_iterator_edge_cases() {
    // 1. Empty buffer and 1..4 byte partial headers return Ok(None)
    assert_eq!(decode_grpc_frame(&[], DEFAULT_MAX_MESSAGE_SIZE), Ok(None));
    let empty_iter = LpmDecoder::new(&[], DEFAULT_MAX_MESSAGE_SIZE);
    assert!(empty_iter.is_empty());
    for header_prefix_len in 1..GRPC_HEADER_LEN {
        let prefix = &[0u8, 0, 0, 0, 5][..header_prefix_len];
        assert_eq!(
            decode_grpc_frame(prefix, DEFAULT_MAX_MESSAGE_SIZE),
            Ok(None)
        );
    }

    // 2. Partial payloads return Ok(None)
    let full_frame = [0u8, 0, 0, 0, 4, b'r', b'o', b't', b'o'];
    for truncated_len in GRPC_HEADER_LEN..full_frame.len() {
        assert_eq!(
            decode_grpc_frame(&full_frame[..truncated_len], DEFAULT_MAX_MESSAGE_SIZE),
            Ok(None)
        );
    }

    // 3. Invalid compression flag (> 1) returns GrpcCode::Internal
    for bad_flag in [2u8, 3, 127, 255] {
        let bad_frame = [bad_flag, 0, 0, 0, 0];
        let err = decode_grpc_frame(&bad_frame, DEFAULT_MAX_MESSAGE_SIZE).unwrap_err();
        assert_eq!(err.code, GrpcCode::Internal);
    }

    // 4. Exact max_message_size boundary (msg_len == max_message_size vs msg_len == max_message_size + 1)
    assert_eq!(
        decode_grpc_frame(&[0u8, 0, 0, 0, 0], 0),
        Ok(Some((
            GrpcFrame {
                compressed: false,
                payload: &[]
            },
            &[][..]
        )))
    );
    assert_eq!(
        decode_grpc_frame(&full_frame, 4),
        Ok(Some((
            GrpcFrame {
                compressed: false,
                payload: b"roto"
            },
            &[][..]
        )))
    );
    let boundary_err = decode_grpc_frame(&full_frame, 3).unwrap_err();
    assert_eq!(boundary_err.code, GrpcCode::ResourceExhausted);

    // Message exceeding max_message_size returns GrpcCode::ResourceExhausted
    // even if the full payload has not arrived yet!
    let oversized_header = [0u8, 0, 0, 0, 11];
    let err = decode_grpc_frame(&oversized_header, 10).unwrap_err();
    assert_eq!(err.code, GrpcCode::ResourceExhausted);

    // u32::MAX payload length header does not overflow and returns ResourceExhausted
    let max_u32_header = [0u8, 0xFF, 0xFF, 0xFF, 0xFF];
    let err = decode_grpc_frame(&max_u32_header, DEFAULT_MAX_MESSAGE_SIZE).unwrap_err();
    assert_eq!(err.code, GrpcCode::ResourceExhausted);

    // 5. Complete stream exhaustion sets iter.is_empty() == true
    let mut complete_stream = Vec::new();
    complete_stream.extend_from_slice(&[0u8, 0, 0, 0, 2, b'h', b'i']);
    complete_stream.extend_from_slice(&[1u8, 0, 0, 0, 3, b'b', b'y', b'e']);
    let mut complete_iter = LpmDecoder::new(&complete_stream, DEFAULT_MAX_MESSAGE_SIZE);
    assert!(!complete_iter.is_empty());
    assert!(complete_iter.next().unwrap().is_ok());
    assert!(!complete_iter.is_empty());
    assert!(complete_iter.next().unwrap().is_ok());
    assert!(complete_iter.is_empty());
    assert_eq!(complete_iter.next(), None);
    assert_eq!(complete_iter.remainder(), &[]);

    // 6. Streaming GrpcFrameIter / LpmDecoder with trailing partial bytes
    let mut stream = Vec::new();
    stream.extend_from_slice(&[0u8, 0, 0, 0, 2, b'h', b'i']);
    stream.extend_from_slice(&[1u8, 0, 0, 0, 3, b'b', b'y', b'e']);
    stream.extend_from_slice(&[0u8, 0, 0, 0, 5, b'p', b'a']); // trailing partial frame (7 bytes, needs 10)

    // Default streaming mode stops cleanly after the 2 complete frames and preserves remainder()
    let mut iter = LpmDecoder::new(&stream, DEFAULT_MAX_MESSAGE_SIZE);
    assert!(!iter.is_empty());
    assert_eq!(
        iter.next(),
        Some(Ok(GrpcFrame {
            compressed: false,
            payload: b"hi"
        }))
    );
    assert_eq!(
        iter.next(),
        Some(Ok(GrpcFrame {
            compressed: true,
            payload: b"bye"
        }))
    );
    assert_eq!(iter.next(), None);
    assert_eq!(iter.remainder(), &[0u8, 0, 0, 0, 5, b'p', b'a']);

    // require_complete = true mode yields an Internal error on the trailing partial bytes
    let mut strict_iter =
        GrpcFrameIter::new(&stream, DEFAULT_MAX_MESSAGE_SIZE).with_require_complete(true);
    assert!(strict_iter.next().unwrap().is_ok());
    assert!(strict_iter.next().unwrap().is_ok());
    let trailing_err = strict_iter.next().unwrap().unwrap_err();
    assert_eq!(trailing_err.code, GrpcCode::Internal);
    assert_eq!(strict_iter.next(), None);

    // 7. Error branch in GrpcFrameIter::next (invalid compression flag and oversized frame) latches self.failed = true
    let mut bad_flag_stream = Vec::new();
    bad_flag_stream.extend_from_slice(&[0u8, 0, 0, 0, 2, b'o', b'k']);
    bad_flag_stream.extend_from_slice(&[2u8, 0, 0, 0, 0]);
    bad_flag_stream.extend_from_slice(&[0u8, 0, 0, 0, 2, b'h', b'i']);
    let mut bad_flag_iter = GrpcFrameIter::new(&bad_flag_stream, DEFAULT_MAX_MESSAGE_SIZE);
    assert_eq!(
        bad_flag_iter.next(),
        Some(Ok(GrpcFrame {
            compressed: false,
            payload: b"ok"
        }))
    );
    let flag_err = bad_flag_iter.next().unwrap().unwrap_err();
    assert_eq!(flag_err.code, GrpcCode::Internal);
    assert_eq!(bad_flag_iter.next(), None);
    assert_eq!(bad_flag_iter.next(), None);

    let mut oversized_stream = Vec::new();
    oversized_stream.extend_from_slice(&[0u8, 0, 0, 0, 2, b'o', b'k']);
    oversized_stream.extend_from_slice(&[0u8, 0, 0, 0, 5, 1, 2, 3, 4, 5]);
    let mut oversized_iter = GrpcFrameIter::new(&oversized_stream, 4);
    assert_eq!(
        oversized_iter.next(),
        Some(Ok(GrpcFrame {
            compressed: false,
            payload: b"ok"
        }))
    );
    let size_err = oversized_iter.next().unwrap().unwrap_err();
    assert_eq!(size_err.code, GrpcCode::ResourceExhausted);
    assert_eq!(oversized_iter.next(), None);
    assert_eq!(oversized_iter.next(), None);
}
