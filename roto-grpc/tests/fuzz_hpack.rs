use roto::runtime::{BackwardWriter, EncodeError};
use roto_grpc::hpack::{
    decode_header_block, decode_prefix_int, encode_dynamic_table_size_update,
    encode_grpc_request_headers, encode_grpc_response_headers, encode_grpc_trailers,
    encode_grpc_trailers_backward, encode_grpc_trailers_only, encode_indexed,
    encode_indexed_backward, encode_literal, encode_literal_backward, encode_literal_ext,
    encode_literal_indexed_name, encode_literal_indexed_name_backward,
    encode_literal_indexed_name_ext, encode_prefix_int, encode_prefix_int_backward,
    encode_string_literal, huffman_decode, huffman_encode, huffman_encoded_len,
    is_grpc_content_type, static_table_get, DynamicTable, GrpcRequestHeaders, GrpcResponseHeaders,
    GrpcTrailers, HeaderField, HpackDecoder, HpackEncoder, HpackError, GRPC_CONTENT_TYPE_ROTO,
    INDEX_AUTHORITY, INDEX_CONTENT_TYPE, INDEX_METHOD_GET, INDEX_METHOD_POST,
    INDEX_PATH_INDEX_HTML, INDEX_PATH_ROOT, INDEX_SCHEME_HTTP, INDEX_SCHEME_HTTPS,
    INDEX_STATUS_200, INDEX_USER_AGENT, STATIC_TABLE, STATIC_TABLE_LEN,
};
use roto_grpc::status::{GrpcCode, GrpcStatus};

// ============================================================================
// Helper Utilities
// ============================================================================

type OwnedHeaderPair = (Vec<u8>, Vec<u8>);

fn unhex(hex: &str) -> Vec<u8> {
    let clean: Vec<u8> = hex
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'|')
        .collect();
    assert_eq!(clean.len() % 2, 0, "odd hex length in {hex:?}");
    clean
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let s = core::str::from_utf8(pair).unwrap();
            u8::from_str_radix(s, 16).unwrap()
        })
        .collect()
}

fn collect_headers<const CAP: usize>(
    decoder: &mut HpackDecoder<CAP>,
    block: &[u8],
) -> Result<Vec<OwnedHeaderPair>, HpackError> {
    let mut scratch = [0u8; 4096];
    let mut out = Vec::new();
    decoder.decode(block, &mut scratch, |field| {
        out.push((field.name.to_vec(), field.value.to_vec()));
        Ok(())
    })?;
    Ok(out)
}

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
        .unwrap_or(0x0029_7541_CAFE_BEEF)
}

fn fuzz_iters(default_iters: usize) -> usize {
    std::env::var("ROTO_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default_iters)
}

// ============================================================================
// 1. RFC 7541 Appendix C.1: Integer Representation Examples
// ============================================================================

#[test]
fn rfc7541_appendix_c1_integer_examples_and_boundaries() {
    let mut buf = [0u8; 32];

    // C.1.1: Encoding 10 using a 5-bit prefix
    let n = encode_prefix_int(5, 0x00, 10, &mut buf).unwrap();
    assert_eq!(&buf[..n], &[0x0a]);
    assert_eq!(decode_prefix_int(5, &buf[..n]), Ok((10, 1)));

    // C.1.2: Encoding 1337 using a 5-bit prefix
    let n = encode_prefix_int(5, 0x00, 1337, &mut buf).unwrap();
    assert_eq!(&buf[..n], &[0x1f, 0x9a, 0x0a]);
    assert_eq!(decode_prefix_int(5, &buf[..n]), Ok((1337, 3)));

    // C.1.3: Encoding 42 starting at an octet boundary (8-bit prefix)
    let n = encode_prefix_int(8, 0x00, 42, &mut buf).unwrap();
    assert_eq!(&buf[..n], &[0x2a]);
    assert_eq!(decode_prefix_int(8, &buf[..n]), Ok((42, 1)));

    // Test all prefix bit widths 1..=8 across boundary values and backward writer equivalence
    let test_values = [
        0usize,
        1,
        2,
        3,
        6,
        7,
        8,
        14,
        15,
        16,
        30,
        31,
        32,
        62,
        63,
        64,
        126,
        127,
        128,
        254,
        255,
        256,
        1337,
        4096,
        65535,
        1_000_000,
        u32::MAX as usize,
        usize::MAX,
    ];

    for prefix_bits in 1..=8 {
        let mask = !(((1u16 << prefix_bits) - 1) as u8);
        for &val in &test_values {
            let n = encode_prefix_int(prefix_bits, mask, val, &mut buf).unwrap();
            assert_eq!(buf[0] & mask, mask);
            let (decoded, consumed) = decode_prefix_int(prefix_bits, &buf[..n]).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(consumed, n);

            // BackwardWriter equivalence
            let mut bw_buf = [0u8; 32];
            let mut bw = BackwardWriter::new(&mut bw_buf);
            encode_prefix_int_backward(prefix_bits, mask, val, &mut bw);
            let bw_slice = bw.finish().unwrap();
            assert_eq!(bw_slice, &buf[..n]);

            // Buffer too small by 1 byte must return BufferOverflow
            let mut short_buf = vec![0u8; n - 1];
            assert_eq!(
                encode_prefix_int(prefix_bits, mask, val, &mut short_buf),
                Err(EncodeError::BufferOverflow)
            );
            // Truncated input by 1 byte must return UnexpectedEof
            assert_eq!(
                decode_prefix_int(prefix_bits, &buf[..n - 1]),
                Err(HpackError::UnexpectedEof)
            );
        }
    }

    // Invalid prefix_bits
    assert_eq!(
        encode_prefix_int(0, 0, 1, &mut buf),
        Err(EncodeError::BufferOverflow)
    );
    assert_eq!(
        encode_prefix_int(9, 0, 1, &mut buf),
        Err(EncodeError::BufferOverflow)
    );
    assert_eq!(
        decode_prefix_int(0, &[0x01]),
        Err(HpackError::IntegerOverflow)
    );
    assert_eq!(
        decode_prefix_int(9, &[0x01]),
        Err(HpackError::IntegerOverflow)
    );

    // Overflowing continuation bytes
    let overlong_zeros = [
        0xFF, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00,
    ];
    assert_eq!(
        decode_prefix_int(8, &overlong_zeros),
        Err(HpackError::IntegerOverflow)
    );
    let overflow_max = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02,
    ];
    assert_eq!(
        decode_prefix_int(8, &overflow_max),
        Err(HpackError::IntegerOverflow)
    );
}

// ============================================================================
// 2. RFC 7541 Appendix A Static Table Verification
// ============================================================================

#[test]
fn rfc7541_appendix_a_static_table_entries() {
    assert_eq!(STATIC_TABLE.len(), 61);
    assert_eq!(static_table_get(0), None);
    assert_eq!(static_table_get(62), None);
    assert_eq!(static_table_get(usize::MAX), None);

    assert_eq!(
        static_table_get(INDEX_AUTHORITY),
        Some((&b":authority"[..], &b""[..]))
    );
    assert_eq!(
        static_table_get(INDEX_METHOD_GET),
        Some((&b":method"[..], &b"GET"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_METHOD_POST),
        Some((&b":method"[..], &b"POST"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_PATH_ROOT),
        Some((&b":path"[..], &b"/"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_PATH_INDEX_HTML),
        Some((&b":path"[..], &b"/index.html"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_SCHEME_HTTP),
        Some((&b":scheme"[..], &b"http"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_SCHEME_HTTPS),
        Some((&b":scheme"[..], &b"https"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_STATUS_200),
        Some((&b":status"[..], &b"200"[..]))
    );
    assert_eq!(
        static_table_get(16),
        Some((&b"accept-encoding"[..], &b"gzip, deflate"[..]))
    );
    assert_eq!(
        static_table_get(INDEX_CONTENT_TYPE),
        Some((&b"content-type"[..], &b""[..]))
    );
    assert_eq!(
        static_table_get(INDEX_USER_AGENT),
        Some((&b"user-agent"[..], &b""[..]))
    );
    assert_eq!(
        static_table_get(61),
        Some((&b"www-authenticate"[..], &b""[..]))
    );

    for idx in 1..=STATIC_TABLE_LEN {
        let (name, _val) = static_table_get(idx).unwrap();
        assert!(!name.is_empty());
    }
}

// ============================================================================
// 3. RFC 7541 Appendix C.2: Header Field Representation Examples
// ============================================================================

#[test]
fn rfc7541_appendix_c2_header_field_representations() {
    // C.2.1: Literal Header Field with Indexing
    let c2_1 = unhex("400a 6375 7374 6f6d 2d6b 6579 0d63 7573 746f 6d2d 6865 6164 6572");
    let mut dec = HpackDecoder::<512>::new();
    let headers = collect_headers(&mut dec, &c2_1).unwrap();
    assert_eq!(
        headers,
        vec![(b"custom-key".to_vec(), b"custom-header".to_vec())]
    );
    assert_eq!(dec.dynamic_table().len(), 1);
    assert_eq!(dec.dynamic_table_size(), 55);
    assert_eq!(
        dec.dynamic_table().get(0),
        Some((&b"custom-key"[..], &b"custom-header"[..]))
    );

    // C.2.2: Literal Header Field without Indexing
    let c2_2 = unhex("040c 2f73 616d 706c 652f 7061 7468");
    let mut dec = HpackDecoder::<512>::new();
    let headers = collect_headers(&mut dec, &c2_2).unwrap();
    assert_eq!(headers, vec![(b":path".to_vec(), b"/sample/path".to_vec())]);
    assert!(dec.dynamic_table().is_empty());
    assert_eq!(dec.dynamic_table_size(), 0);

    // C.2.3: Literal Header Field Never Indexed
    let c2_3 = unhex("1008 7061 7373 776f 7264 0673 6563 7265 74");
    let mut dec = HpackDecoder::<512>::new();
    let headers = collect_headers(&mut dec, &c2_3).unwrap();
    assert_eq!(headers, vec![(b"password".to_vec(), b"secret".to_vec())]);
    assert!(dec.dynamic_table().is_empty());
    assert_eq!(dec.dynamic_table_size(), 0);

    // C.2.4: Indexed Header Field
    let c2_4 = unhex("82");
    let mut dec = HpackDecoder::<512>::new();
    let headers = collect_headers(&mut dec, &c2_4).unwrap();
    assert_eq!(headers, vec![(b":method".to_vec(), b"GET".to_vec())]);
    assert!(dec.dynamic_table().is_empty());
    assert_eq!(dec.dynamic_table_size(), 0);
}

// ============================================================================
// 4. RFC 7541 Appendix C.3 & C.4: Request Examples (Plain & Huffman)
// ============================================================================

#[test]
fn rfc7541_appendix_c3_and_c4_request_examples() {
    let c3_blocks = [
        unhex("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d"),
        unhex("8286 84be 5808 6e6f 2d63 6163 6865"),
        unhex("8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65"),
    ];
    let c4_blocks = [
        unhex("8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff"),
        unhex("8286 84be 5886 a8eb 1064 9cbf"),
        unhex("8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf"),
    ];

    for blocks in [&c3_blocks, &c4_blocks] {
        let mut dec = HpackDecoder::<512>::new();

        // First Request
        let h1 = collect_headers(&mut dec, &blocks[0]).unwrap();
        assert_eq!(
            h1,
            vec![
                (b":method".to_vec(), b"GET".to_vec()),
                (b":scheme".to_vec(), b"http".to_vec()),
                (b":path".to_vec(), b"/".to_vec()),
                (b":authority".to_vec(), b"www.example.com".to_vec()),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 1);
        assert_eq!(dec.dynamic_table_size(), 57);
        assert_eq!(
            dec.dynamic_table().get(0),
            Some((&b":authority"[..], &b"www.example.com"[..]))
        );

        // Second Request
        let h2 = collect_headers(&mut dec, &blocks[1]).unwrap();
        assert_eq!(
            h2,
            vec![
                (b":method".to_vec(), b"GET".to_vec()),
                (b":scheme".to_vec(), b"http".to_vec()),
                (b":path".to_vec(), b"/".to_vec()),
                (b":authority".to_vec(), b"www.example.com".to_vec()),
                (b"cache-control".to_vec(), b"no-cache".to_vec()),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 2);
        assert_eq!(dec.dynamic_table_size(), 110);
        assert_eq!(
            dec.dynamic_table().get(0),
            Some((&b"cache-control"[..], &b"no-cache"[..]))
        );
        assert_eq!(
            dec.dynamic_table().get(1),
            Some((&b":authority"[..], &b"www.example.com"[..]))
        );

        // Third Request
        let h3 = collect_headers(&mut dec, &blocks[2]).unwrap();
        assert_eq!(
            h3,
            vec![
                (b":method".to_vec(), b"GET".to_vec()),
                (b":scheme".to_vec(), b"https".to_vec()),
                (b":path".to_vec(), b"/index.html".to_vec()),
                (b":authority".to_vec(), b"www.example.com".to_vec()),
                (b"custom-key".to_vec(), b"custom-value".to_vec()),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 3);
        assert_eq!(dec.dynamic_table_size(), 164);
        assert_eq!(
            dec.dynamic_table().get(0),
            Some((&b"custom-key"[..], &b"custom-value"[..]))
        );
        assert_eq!(
            dec.dynamic_table().get(1),
            Some((&b"cache-control"[..], &b"no-cache"[..]))
        );
        assert_eq!(
            dec.dynamic_table().get(2),
            Some((&b":authority"[..], &b"www.example.com"[..]))
        );
    }
}

// ============================================================================
// 5. RFC 7541 Appendix C.5 & C.6: Response Examples (Plain & Huffman, Eviction)
// ============================================================================

#[test]
fn rfc7541_appendix_c5_and_c6_response_examples_with_eviction() {
    let c5_blocks = [
        unhex(
            "4803 3330 3258 0770 7269 7661 7465 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3120 474d 546e 1768 7474 7073 3a2f 2f77 7777 2e65 7861 6d70 6c65 2e63 6f6d",
        ),
        unhex("4803 3330 37c1 c0bf"),
        unhex(
            "88c1 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3220 474d 54c0 5a04 677a 6970 7738 666f 6f3d 4153 444a 4b48 514b 425a 584f 5157 454f 5049 5541 5851 5745 4f49 553b 206d 6178 2d61 6765 3d33 3630 303b 2076 6572 7369 6f6e 3d31",
        ),
    ];
    let c6_blocks = [
        unhex(
            "4882 6402 5885 aec3 771a 4b61 96d0 7abe 9410 54d4 44a8 2005 9504 0b81 66e0 82a6 2d1b ff6e 919d 29ad 1718 63c7 8f0b 97c8 e9ae 82ae 43d3",
        ),
        unhex("4883 640e ffc1 c0bf"),
        unhex(
            "88c1 6196 d07a be94 1054 d444 a820 0595 040b 8166 e084 a62d 1bff c05a 839b d9ab 77ad 94e7 821d d7f2 e6c7 b335 dfdf cd5b 3960 d5af 2708 7f36 72c1 ab27 0fb5 291f 9587 3160 65c0 03ed 4ee5 b106 3d50 07",
        ),
    ];

    for blocks in [&c5_blocks, &c6_blocks] {
        // Max dynamic table size = 256 octets per RFC 7541 Appendix C.5 / C.6
        let mut dec = HpackDecoder::<512>::with_max_table_size(256);

        // First Response
        let h1 = collect_headers(&mut dec, &blocks[0]).unwrap();
        assert_eq!(
            h1,
            vec![
                (b":status".to_vec(), b"302".to_vec()),
                (b"cache-control".to_vec(), b"private".to_vec()),
                (b"date".to_vec(), b"Mon, 21 Oct 2013 20:13:21 GMT".to_vec()),
                (b"location".to_vec(), b"https://www.example.com".to_vec()),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 4);
        assert_eq!(dec.dynamic_table_size(), 222);

        // Second Response (evicts `:status: 302` to make room for `:status: 307`)
        let h2 = collect_headers(&mut dec, &blocks[1]).unwrap();
        assert_eq!(
            h2,
            vec![
                (b":status".to_vec(), b"307".to_vec()),
                (b"cache-control".to_vec(), b"private".to_vec()),
                (b"date".to_vec(), b"Mon, 21 Oct 2013 20:13:21 GMT".to_vec()),
                (b"location".to_vec(), b"https://www.example.com".to_vec()),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 4);
        assert_eq!(dec.dynamic_table_size(), 222);
        assert_eq!(
            dec.dynamic_table().get(0),
            Some((&b":status"[..], &b"307"[..]))
        );

        // Third Response
        let h3 = collect_headers(&mut dec, &blocks[2]).unwrap();
        assert_eq!(
            h3,
            vec![
                (b":status".to_vec(), b"200".to_vec()),
                (b"cache-control".to_vec(), b"private".to_vec()),
                (b"date".to_vec(), b"Mon, 21 Oct 2013 20:13:22 GMT".to_vec()),
                (b"location".to_vec(), b"https://www.example.com".to_vec()),
                (b"content-encoding".to_vec(), b"gzip".to_vec()),
                (
                    b"set-cookie".to_vec(),
                    b"foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1".to_vec()
                ),
            ]
        );
        assert_eq!(dec.dynamic_table().len(), 3);
        assert_eq!(dec.dynamic_table_size(), 215);
        assert_eq!(
            dec.dynamic_table().get(0),
            Some((
                &b"set-cookie"[..],
                &b"foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1"[..]
            ))
        );
        assert_eq!(
            dec.dynamic_table().get(1),
            Some((&b"content-encoding"[..], &b"gzip"[..]))
        );
        assert_eq!(
            dec.dynamic_table().get(2),
            Some((&b"date"[..], &b"Mon, 21 Oct 2013 20:13:22 GMT"[..]))
        );
    }
}

// ============================================================================
// 6. Huffman Padding, EOS Rejection, and All 256 Bytes Round-Trip
// ============================================================================

#[test]
fn huffman_all_256_bytes_and_strict_padding_validation() {
    // Round-trip all 256 individual byte values and the full 0..=255 sequence
    let all_bytes: Vec<u8> = (0u8..=255).collect();
    let mut enc_buf = [0u8; 1024];
    let mut dec_buf = [0u8; 1024];

    for b in 0u8..=255 {
        let single = [b];
        let expected_len = huffman_encoded_len(&single);
        let written = huffman_encode(&single, &mut enc_buf).unwrap();
        assert_eq!(written, expected_len);
        let decoded = huffman_decode(&enc_buf[..written], &mut dec_buf).unwrap();
        assert_eq!(decoded, &single);
    }

    let total_enc_len = huffman_encoded_len(&all_bytes);
    let written = huffman_encode(&all_bytes, &mut enc_buf).unwrap();
    assert_eq!(written, total_enc_len);
    let decoded = huffman_decode(&enc_buf[..written], &mut dec_buf).unwrap();
    assert_eq!(decoded, &all_bytes[..]);

    // Empty slice
    assert_eq!(huffman_encoded_len(&[]), 0);
    assert_eq!(huffman_encode(&[], &mut enc_buf), Ok(0));
    assert_eq!(huffman_decode(&[], &mut dec_buf), Ok(&[][..]));

    // Invalid padding: >= 8 bits of 1s (0xFF alone has 8 bits of 1s)
    assert_eq!(
        huffman_decode(&[0xFF], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );
    assert_eq!(
        huffman_decode(&[0xFF, 0xFF], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );

    // Valid Huffman for "0" (`00000` + `111` padding = `0x07`) followed by extra `0xFF` padding byte
    assert_eq!(huffman_decode(&[0x07], &mut dec_buf), Ok(&b"0"[..]));
    assert_eq!(
        huffman_decode(&[0x07, 0xFF], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );

    // Non-1 padding bits: "0" (`00000`) + `110` (`0x06`) instead of `111` (`0x07`)
    assert_eq!(
        huffman_decode(&[0x06], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );
    assert_eq!(
        huffman_decode(&[0x00, 0x06], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );

    // Explicit 30-bit EOS symbol (30 ones + 2 one padding bits = 4 bytes of 0xFF)
    assert_eq!(
        huffman_decode(&[0xFF, 0xFF, 0xFF, 0xFF], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );
    // "0" (5 bits 00000) + 30-bit EOS (30 ones) + 5-bit padding (5 ones) = 40 bits (5 bytes):
    // [0x07, 0xFF, 0xFF, 0xFF, 0xFF]
    assert_eq!(
        huffman_decode(&[0x07, 0xFF, 0xFF, 0xFF, 0xFF], &mut dec_buf),
        Err(HpackError::InvalidHuffman)
    );

    // Buffer overflow when `out` is too small
    let mut tiny = [0u8; 2];
    assert_eq!(
        huffman_decode(&unhex("f1e3c2e5f23a6ba0ab90f4ff"), &mut tiny),
        Err(HpackError::BufferOverflow)
    );
}

// ============================================================================
// 7. Invalid Indices & Dynamic Table Size Update Protocol Rules
// ============================================================================

#[test]
fn hpack_decoder_invalid_indices_and_table_size_updates() {
    let mut scratch = [0u8; 256];

    // Index 0 in Indexed Header Field (0x80) -> InvalidIndex
    assert_eq!(
        decode_header_block(&[0x80], &mut scratch, |_| Ok(())),
        Err(HpackError::InvalidIndex)
    );

    // Index 62 when dynamic table is empty -> InvalidIndex
    let mut idx_62 = [0u8; 4];
    let n = encode_indexed(62, &mut idx_62).unwrap();
    assert_eq!(
        decode_header_block(&idx_62[..n], &mut scratch, |_| Ok(())),
        Err(HpackError::InvalidIndex)
    );

    // Literal with indexed name > 61 when dynamic table is empty -> InvalidIndex
    let mut lit_62 = [0u8; 16];
    let n = encode_literal_indexed_name(62, b"val", false, &mut lit_62).unwrap();
    assert_eq!(
        decode_header_block(&lit_62[..n], &mut scratch, |_| Ok(())),
        Err(HpackError::InvalidIndex)
    );

    // Stateless decoder (max_allowed_table_size = 0) accepts Literal with Incremental Indexing
    // by evicting/keeping the dynamic table empty per RFC 7541 §4.4, while rejecting dynamic index 62!
    let mut stateless_dec = HpackDecoder::<0>::stateless();
    let mut inc_buf = [0u8; 32];
    let n = encode_literal_ext(b"x-test", b"val", false, true, false, &mut inc_buf).unwrap();
    let headers = collect_headers(&mut stateless_dec, &inc_buf[..n]).unwrap();
    assert_eq!(headers, vec![(b"x-test".to_vec(), b"val".to_vec())]);
    assert!(stateless_dec.dynamic_table().is_empty());

    // Dynamic table size update exceeding max_allowed_table_size -> DynamicTableSizeExceeded
    let mut dec = HpackDecoder::<512>::with_max_table_size(128);
    let mut upd_buf = [0u8; 8];
    let n = encode_dynamic_table_size_update(129, &mut upd_buf).unwrap();
    assert_eq!(
        collect_headers(&mut dec, &upd_buf[..n]),
        Err(HpackError::DynamicTableSizeExceeded)
    );

    // Two dynamic table size updates at the very beginning of a header block (valid per RFC 7541 §4.2)
    let mut block = Vec::new();
    let mut tmp = [0u8; 16];
    let n1 = encode_dynamic_table_size_update(0, &mut tmp).unwrap();
    block.extend_from_slice(&tmp[..n1]);
    let n2 = encode_dynamic_table_size_update(128, &mut tmp).unwrap();
    block.extend_from_slice(&tmp[..n2]);
    let n3 = encode_indexed(INDEX_METHOD_POST, &mut tmp).unwrap();
    block.extend_from_slice(&tmp[..n3]);
    let headers = collect_headers(&mut dec, &block).unwrap();
    assert_eq!(headers, vec![(b":method".to_vec(), b"POST".to_vec())]);
    assert_eq!(dec.dynamic_table().max_size(), 128);

    // Dynamic table size update AFTER a header field in a header block -> InvalidTableSizeUpdate
    let mut bad_block = Vec::new();
    bad_block.push(0x83); // :method: POST
    let n = encode_dynamic_table_size_update(64, &mut tmp).unwrap();
    bad_block.extend_from_slice(&tmp[..n]);
    assert_eq!(
        collect_headers(&mut dec, &bad_block),
        Err(HpackError::InvalidTableSizeUpdate)
    );

    // Self-evicting incremental indexing: name_index references dynamic entry 62, and inserting
    // the new entry evicts entry 62 (RFC 7541 §6.2.1).
    let mut small_dec = HpackDecoder::<128>::with_max_table_size(64);
    let mut b1 = [0u8; 64];
    // Entry 1: "x-custom-a" (10) + "123456" (6) + 32 = 48 bytes <= 64
    let n1 = encode_literal_ext(b"x-custom-a", b"123456", false, true, false, &mut b1).unwrap();
    assert_eq!(
        collect_headers(&mut small_dec, &b1[..n1]).unwrap(),
        vec![(b"x-custom-a".to_vec(), b"123456".to_vec())]
    );
    assert_eq!(small_dec.dynamic_table().len(), 1);
    // Entry 2 references name_index = 62 ("x-custom-a") and value "abcdef" (48 bytes), evicting Entry 1!
    let n2 = encode_literal_indexed_name_ext(62, b"abcdef", false, true, false, &mut b1).unwrap();
    assert_eq!(
        collect_headers(&mut small_dec, &b1[..n2]).unwrap(),
        vec![(b"x-custom-a".to_vec(), b"abcdef".to_vec())]
    );
    assert_eq!(small_dec.dynamic_table().len(), 1);
    assert_eq!(
        small_dec.dynamic_table().get(0),
        Some((&b"x-custom-a"[..], &b"abcdef"[..]))
    );
}

// ============================================================================
// 8. High-Level gRPC Request, Response, and Trailers Round-Trips
// ============================================================================

#[test]
fn grpc_request_response_and_trailers_round_trips() {
    for use_huffman in [false, true] {
        let encoder = HpackEncoder::with_huffman(use_huffman);
        let mut wire = [0u8; 512];
        let mut scratch = [0u8; 512];
        let mut dec = HpackDecoder::<512>::new();

        // 1. gRPC Request Headers
        let req_len = encoder
            .encode_grpc_request_headers(
                b"http",
                b"127.0.0.1:50051",
                b"/ telemetry.SensorService/RecordReading",
                GRPC_CONTENT_TYPE_ROTO,
                &mut wire,
            )
            .unwrap();
        let req = GrpcRequestHeaders::decode(&mut dec, &wire[..req_len], &mut scratch).unwrap();
        assert!(req.is_valid_grpc_post());
        assert_eq!(req.method, Some(&b"POST"[..]));
        assert_eq!(req.scheme, Some(&b"http"[..]));
        assert_eq!(req.authority, Some(&b"127.0.0.1:50051"[..]));
        assert_eq!(
            req.path_str(),
            Some("/ telemetry.SensorService/RecordReading")
        );
        assert_eq!(req.content_type, Some(GRPC_CONTENT_TYPE_ROTO));
        assert_eq!(req.te, Some(&b"trailers"[..]));

        // 2. gRPC Response Headers
        let resp_len = encoder
            .encode_grpc_response_headers(GRPC_CONTENT_TYPE_ROTO, &mut wire)
            .unwrap();
        let resp = GrpcResponseHeaders::decode(&mut dec, &wire[..resp_len], &mut scratch).unwrap();
        assert_eq!(resp.status, Some(&b"200"[..]));
        assert_eq!(resp.content_type, Some(GRPC_CONTENT_TYPE_ROTO));
        assert!(!resp.is_trailers_only());

        // 3. gRPC Trailers with percent-encoded message
        let status = GrpcStatus::invalid_argument("bad field: 50% > max\nretry ☕");
        let tr_len = encoder.encode_grpc_trailers(status, &mut wire).unwrap();
        let trailers = GrpcTrailers::decode(&mut dec, &wire[..tr_len], &mut scratch).unwrap();
        let mut msg_scratch = [0u8; 256];
        let decoded_status = trailers.to_grpc_status(&mut msg_scratch).unwrap();
        assert_eq!(decoded_status, status);

        // 4. gRPC Trailers-Only Response
        let tr_only_status = GrpcStatus::unimplemented("service/method not found");
        let tr_only_len = encoder
            .encode_grpc_trailers_only(tr_only_status, GRPC_CONTENT_TYPE_ROTO, &mut wire)
            .unwrap();
        let resp_tr_only =
            GrpcResponseHeaders::decode(&mut dec, &wire[..tr_only_len], &mut scratch).unwrap();
        assert!(resp_tr_only.is_trailers_only());
        assert_eq!(resp_tr_only.status, Some(&b"200"[..]));
        assert_eq!(resp_tr_only.content_type, Some(GRPC_CONTENT_TYPE_ROTO));
        assert_eq!(
            resp_tr_only.to_grpc_status(&mut msg_scratch).unwrap(),
            Some(tr_only_status)
        );
    }

    // Verify top-level convenience functions and BackwardWriter trailers equivalence
    let mut f_buf = [0u8; 256];
    let status = GrpcStatus::resource_exhausted("quota 100% exhausted\r\n");
    let f_len = encode_grpc_trailers_only(status, GRPC_CONTENT_TYPE_ROTO, &mut f_buf).unwrap();

    let mut bw_buf = [0u8; 256];
    let mut bw = BackwardWriter::new(&mut bw_buf);
    encode_grpc_trailers_backward(status, true, GRPC_CONTENT_TYPE_ROTO, &mut bw);
    let bw_slice = bw.finish().unwrap();
    assert_eq!(bw_slice, &f_buf[..f_len]);

    // Also test top-level request/response/trailers functions
    let mut req_buf = [0u8; 256];
    let n = encode_grpc_request_headers(
        b"https",
        b"example.com",
        b"/pkg.Svc/Call",
        GRPC_CONTENT_TYPE_ROTO,
        &mut req_buf,
    )
    .unwrap();
    let mut scratch = [0u8; 256];
    let mut dec = HpackDecoder::<512>::stateless();
    let req = GrpcRequestHeaders::decode(&mut dec, &req_buf[..n], &mut scratch).unwrap();
    assert!(req.is_valid_grpc_post());
    assert_eq!(req.scheme, Some(&b"https"[..]));

    let mut resp_buf = [0u8; 128];
    let n = encode_grpc_response_headers(GRPC_CONTENT_TYPE_ROTO, &mut resp_buf).unwrap();
    let resp = GrpcResponseHeaders::decode(&mut dec, &resp_buf[..n], &mut scratch).unwrap();
    assert_eq!(resp.status, Some(&b"200"[..]));

    let mut tr_buf = [0u8; 128];
    let n = encode_grpc_trailers(GrpcStatus::ok(), &mut tr_buf).unwrap();
    let tr = GrpcTrailers::decode(&mut dec, &tr_buf[..n], &mut scratch).unwrap();
    let mut msg_scratch = [0u8; 64];
    assert_eq!(tr.to_grpc_status(&mut msg_scratch), Ok(GrpcStatus::ok()));

    assert!(is_grpc_content_type(b"application/grpc"));
    assert!(is_grpc_content_type(b"application/grpc+roto"));
    assert!(is_grpc_content_type(b"application/grpc+proto"));
    assert!(is_grpc_content_type(b"application/grpc;charset=utf-8"));
    assert!(!is_grpc_content_type(b"application/json"));
    assert!(!is_grpc_content_type(b"application/grpc_bad"));

    // Backward literal & indexed helpers
    let mut b_buf = [0u8; 128];
    let mut bw = BackwardWriter::new(&mut b_buf);
    encode_literal_backward(b"x-key", b"x-val", true, &mut bw);
    encode_literal_indexed_name_backward(INDEX_USER_AGENT, b"roto-grpc/0.1", false, &mut bw);
    encode_indexed_backward(INDEX_METHOD_GET, &mut bw);
    let block = bw.finish().unwrap();
    let decoded = collect_headers(&mut dec, block).unwrap();
    assert_eq!(
        decoded,
        vec![
            (b":method".to_vec(), b"GET".to_vec()),
            (b"user-agent".to_vec(), b"roto-grpc/0.1".to_vec()),
            (b"x-key".to_vec(), b"x-val".to_vec()),
        ]
    );

    // Direct encode_literal, HeaderField helpers, and HpackError conversions
    let mut lit_buf = [0u8; 64];
    let lit_len = encode_literal(b"x-trace-id", b"abc-123", true, &mut lit_buf).unwrap();
    decode_header_block(&lit_buf[..lit_len], &mut scratch, |hdr| {
        assert!(!hdr.is_pseudo());
        assert_eq!(hdr.name_str(), Some("x-trace-id"));
        assert_eq!(hdr.value_str(), Some("abc-123"));
        Ok(())
    })
    .unwrap();
    assert!(HeaderField::new(b":path", b"/").is_pseudo());
    assert_eq!(
        GrpcStatus::from(HpackError::BufferOverflow).code,
        GrpcCode::ResourceExhausted
    );
    assert_eq!(
        GrpcStatus::from(HpackError::InvalidHuffman).code,
        GrpcCode::Internal
    );
    assert!(!HpackError::UnexpectedEof.to_string().is_empty());
}

// ============================================================================
// 9. Deterministic Fuzzer: Arbitrary Wire Bytes -> Zero Panics
// ============================================================================

#[test]
fn fuzz_arbitrary_wire_bytes_never_panic() {
    let seed = fuzz_seed();
    let iters = fuzz_iters(2_500);
    let mut rng = SplitMix64::new(seed);

    let mut input = [0u8; 512];
    let mut scratch = [0u8; 1024];
    let mut small_scratch = [0u8; 16];
    let mut stateful_dec = HpackDecoder::<512>::new();

    for _ in 0..iters {
        let len = rng.next_usize(input.len() + 1);
        rng.fill_bytes(&mut input[..len]);
        let slice = &input[..len];

        // 1. Prefix integer decoder across all prefix bit widths 0..=9
        for prefix_bits in 0..=9 {
            let _ = decode_prefix_int(prefix_bits, slice);
        }

        // 2. Huffman decoder with normal and tiny scratch buffers
        let _ = huffman_decode(slice, &mut scratch);
        let _ = huffman_decode(slice, &mut small_scratch);

        // 3. Stateless and stateful HPACK decoders
        let _ = decode_header_block(slice, &mut scratch, |_hdr| Ok(()));
        let _ = decode_header_block(slice, &mut small_scratch, |_hdr| Ok(()));
        let _ = stateful_dec.decode(slice, &mut scratch, |_hdr| Ok(()));

        // 4. High-level gRPC header extractors
        let mut req_dec = HpackDecoder::<256>::new();
        let _ = GrpcRequestHeaders::decode(&mut req_dec, slice, &mut scratch);
        let mut resp_dec = HpackDecoder::<256>::new();
        if let Ok(resp) = GrpcResponseHeaders::decode(&mut resp_dec, slice, &mut scratch) {
            let mut msg_buf = [0u8; 128];
            let _ = resp.to_grpc_status(&mut msg_buf);
        }
        let mut tr_dec = HpackDecoder::<256>::new();
        if let Ok(tr) = GrpcTrailers::decode(&mut tr_dec, slice, &mut scratch) {
            let mut msg_buf = [0u8; 128];
            let _ = tr.to_grpc_status(&mut msg_buf);
        }
    }
}

// ============================================================================
// 10. Deterministic Fuzzer: Structured Header Blocks & Round-Trip Equality
// ============================================================================

#[test]
fn fuzz_structured_hpack_round_trip_and_mutations() {
    let seed = fuzz_seed().wrapping_add(0x1337_C0DE);
    let iters = fuzz_iters(600);
    let mut rng = SplitMix64::new(seed);

    let mut enc_dyn = DynamicTable::<512>::with_max_size(512);
    let mut dec = HpackDecoder::<512>::with_max_table_size(512);

    let mut wire = [0u8; 8192];
    let mut scratch = [0u8; 8192];

    for iter in 0..iters {
        // Periodically reset both tables or issue dynamic table size updates at the start of a block
        if iter % 50 == 0 {
            enc_dyn = DynamicTable::<512>::with_max_size(512);
            dec = HpackDecoder::<512>::with_max_table_size(512);
        }

        let mut pos = 0usize;
        let mut expected_headers: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        // Optional 0, 1, or 2 dynamic table size updates at the start of the header block
        let num_updates = rng.next_usize(3);
        for _ in 0..num_updates {
            let new_max = rng.next_usize(513);
            pos += encode_dynamic_table_size_update(new_max, &mut wire[pos..]).unwrap();
            enc_dyn.set_max_size(new_max);
        }

        let num_headers = 1 + rng.next_usize(20);
        for _ in 0..num_headers {
            let total_entries = STATIC_TABLE_LEN + enc_dyn.len();
            let mode = rng.next_usize(5);

            match mode {
                0 => {
                    // Indexed Header Field (static or dynamic)
                    let idx = 1 + rng.next_usize(total_entries);
                    let (name, val) = if let Some((n, v)) = static_table_get(idx) {
                        (n.to_vec(), v.to_vec())
                    } else {
                        let (n, v) = enc_dyn.get(idx - STATIC_TABLE_LEN - 1).unwrap();
                        (n.to_vec(), v.to_vec())
                    };
                    pos += encode_indexed(idx, &mut wire[pos..]).unwrap();
                    expected_headers.push((name, val));
                }
                1 | 2 => {
                    // Literal Header Field with Indexed Name (with or without incremental indexing)
                    let incremental = mode == 1;
                    let never_index = !incremental && rng.next_bool();
                    let use_huffman = rng.next_bool();
                    let name_idx = 1 + rng.next_usize(total_entries);
                    let name = if let Some((n, _)) = static_table_get(name_idx) {
                        n.to_vec()
                    } else {
                        enc_dyn
                            .get(name_idx - STATIC_TABLE_LEN - 1)
                            .unwrap()
                            .0
                            .to_vec()
                    };
                    let val_len = rng.next_usize(48);
                    let mut val = vec![0u8; val_len];
                    rng.fill_bytes(&mut val);

                    pos += encode_literal_indexed_name_ext(
                        name_idx,
                        &val,
                        never_index,
                        incremental,
                        use_huffman,
                        &mut wire[pos..],
                    )
                    .unwrap();
                    if incremental {
                        enc_dyn.insert(&name, &val);
                    }
                    expected_headers.push((name, val));
                }
                _ => {
                    // Literal Header Field with Literal Name and Value
                    let incremental = mode == 3;
                    let never_index = !incremental && rng.next_bool();
                    let use_huffman = rng.next_bool();
                    let name_len = 1 + rng.next_usize(24);
                    let val_len = rng.next_usize(48);
                    let mut name = vec![0u8; name_len];
                    let mut val = vec![0u8; val_len];
                    rng.fill_bytes(&mut name);
                    rng.fill_bytes(&mut val);

                    // Encode name and value independently with random Huffman choices
                    let name_huff = rng.next_bool();
                    wire[pos] = if incremental {
                        0x40
                    } else if never_index {
                        0x10
                    } else {
                        0x00
                    };
                    pos += 1;
                    pos += encode_string_literal(&name, name_huff, &mut wire[pos..]).unwrap();
                    pos += encode_string_literal(&val, use_huffman, &mut wire[pos..]).unwrap();

                    if incremental {
                        enc_dyn.insert(&name, &val);
                    }
                    expected_headers.push((name, val));
                }
            }
        }

        let block = &wire[..pos];
        let mut actual_headers: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        dec.decode(block, &mut scratch, |HeaderField { name, value }| {
            actual_headers.push((name.to_vec(), value.to_vec()));
            Ok(())
        })
        .unwrap();

        assert_eq!(actual_headers, expected_headers, "failed on iter {iter}");
        assert_eq!(dec.dynamic_table().len(), enc_dyn.len());
        assert_eq!(dec.dynamic_table_size(), enc_dyn.size());

        // Also mutate a clone of the valid block and verify a cloned decoder never panics
        if !block.is_empty() {
            let mut mutated = block.to_vec();
            let mut_idx = rng.next_usize(mutated.len());
            mutated[mut_idx] ^= 1u8 << (rng.next_usize(8) as u32);
            let mut clone_dec = dec.clone();
            let _ = clone_dec.decode(&mutated, &mut scratch, |_| Ok(()));

            let trunc_len = rng.next_usize(block.len());
            let mut clone_dec2 = dec.clone();
            let _ = clone_dec2.decode(&block[..trunc_len], &mut scratch, |_| Ok(()));
        }
    }
}
