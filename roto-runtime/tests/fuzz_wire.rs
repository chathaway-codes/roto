//! Deterministic runtime wire-format fuzzer for `roto-runtime`.
//!
//! This test suite exercises the low-level `#![no_std]` wire primitives, single-pass
//! [`FieldIter`] scanner, [`BackwardWriter`] serializer, and zero-allocation repeated /
//! unknown field iterators under both arbitrary malformed inputs and structured
//! round-trip workloads:
//!
//! 1. **Arbitrary & Semi-Structured Wire Fuzzing (`fuzz_arbitrary_and_structured_wire_bytes_never_panic`)**:
//!    Generates uniform random byte buffers (`0..=1024` bytes) as well as adversarial
//!    semi-structured wire streams (tags `0..=u32::MAX`, wire types `0..=7`, nested
//!    `StartGroup`/`EndGroup` pairs up to depth 70 exceeding the 64-level recursion guard,
//!    overlong 10/11-byte varints, truncated payloads, and random bit-flips). Drives
//!    [`FieldIter`], [`UnknownFieldsIter`], [`RepeatedVarintIter`], [`RepeatedZigZagIter`],
//!    [`RepeatedFixed32Iter`], [`RepeatedFixed64Iter`], [`RepeatedBytesIter`],
//!    [`RepeatedStrIter`], [`RepeatedMessageIter`], [`RepeatedGroupIter`], and point decode
//!    helpers, verifying that no input ever panics or fails to terminate and that every
//!    iterator strictly fuses (`next() == None`) after yielding an error.
//!
//! 2. **Unparse -> Parse Wire Round-Trip (`fuzz_backward_writer_and_field_iter_roundtrip`)**:
//!    Generates random trees of valid wire fields across all wire types (varints, ZigZag
//!    32/64, fixed 32/64, IEEE-754 floats/doubles including `NaN` and `-0.0`, length-delimited
//!    payloads, and nested `StartGroup`..`EndGroup` groups). Serializes them back-to-front
//!    with [`BackwardWriter`], parses them front-to-back with [`FieldIter`], and asserts
//!    exact tag, wire-type, span, and bit-identical value preservation as well as filtered
//!    subset forwarding via [`BackwardWriter::copy_filtered_fields`].
//!
//! ## Configuration via Environment Variables
//! - `ROTO_FUZZ_SEED`: 64-bit PRNG seed in decimal or `0x` hex (default: `0x524F_544F_4655_5A5A`).
//! - `ROTO_FUZZ_ITERS`: Number of fuzz iterations per test (default: `2000`).

use roto_runtime::{
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker, MessageView,
    RepeatedBytesIter, RepeatedFixed32Iter, RepeatedFixed64Iter, RepeatedGroupIter,
    RepeatedMessageIter, RepeatedStrIter, RepeatedVarintIter, RepeatedZigZagIter,
    UnknownFieldsIter, WireType, decode_bool_at, decode_bytes_at, decode_f32_at, decode_f64_at,
    decode_fixed32_at, decode_fixed64_at, decode_sfixed32_at, decode_sfixed64_at, decode_sint32_at,
    decode_sint64_at, decode_str_at, decode_varint, decode_varint_at,
};

const DEFAULT_SEED: u64 = 0x524F_544F_4655_5A5A; // "ROTOFUZZ"
const DEFAULT_ITERS: usize = 2000;

/// Fast, deterministic, zero-dependency `SplitMix64` PRNG.
#[derive(Clone, Debug)]
struct Rng {
    state: u64,
}

impl Rng {
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

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }

    fn gen_range(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper_exclusive
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        let mut i = 0;
        while i + 8 <= buf.len() {
            let w = self.next_u64().to_le_bytes();
            buf[i..i + 8].copy_from_slice(&w);
            i += 8;
        }
        if i < buf.len() {
            let w = self.next_u64().to_le_bytes();
            let rem = buf.len() - i;
            buf[i..].copy_from_slice(&w[..rem]);
        }
    }

    fn edge_u64(&mut self) -> u64 {
        match self.gen_range(12) {
            0 => 0,
            1 => 1,
            2 => 127,
            3 => 128,
            4 => 16_383,
            5 => 16_384,
            6 => u32::MAX as u64,
            7 => (u32::MAX as u64) + 1,
            8 => i64::MAX as u64,
            9 => i64::MIN as u64,
            10 => u64::MAX,
            _ => self.next_u64(),
        }
    }

    fn edge_i32(&mut self) -> i32 {
        match self.gen_range(10) {
            0 => 0,
            1 => 1,
            2 => -1,
            3 => 127,
            4 => -128,
            5 => i32::MAX,
            6 => i32::MIN,
            7 => i32::MIN + 1,
            _ => self.next_u32() as i32,
        }
    }

    fn edge_i64(&mut self) -> i64 {
        match self.gen_range(10) {
            0 => 0,
            1 => 1,
            2 => -1,
            3 => i32::MAX as i64,
            4 => i32::MIN as i64,
            5 => i64::MAX,
            6 => i64::MIN,
            7 => i64::MIN + 1,
            _ => self.next_u64() as i64,
        }
    }

    fn edge_f32(&mut self) -> f32 {
        match self.gen_range(10) {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => f32::MIN_POSITIVE,
            5 => f32::MAX,
            6 => f32::MIN,
            7 => f32::INFINITY,
            8 => f32::NEG_INFINITY,
            9 => f32::from_bits(0x7FC0_0001), // quiet NaN with payload
            _ => f32::from_bits(self.next_u32()),
        }
    }

    fn edge_f64(&mut self) -> f64 {
        match self.gen_range(10) {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => f64::MIN_POSITIVE,
            5 => f64::MAX,
            6 => f64::MIN,
            7 => f64::INFINITY,
            8 => f64::NEG_INFINITY,
            9 => f64::from_bits(0x7FF8_0000_0000_0001), // quiet NaN with payload
            _ => f64::from_bits(self.next_u64()),
        }
    }
}

fn read_fuzz_config() -> (u64, usize) {
    let seed = std::env::var("ROTO_FUZZ_SEED")
        .ok()
        .and_then(|s| {
            let s = s.trim();
            if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(hex, 16).ok()
            } else {
                s.parse::<u64>().ok()
            }
        })
        .unwrap_or(DEFAULT_SEED);

    let iters = std::env::var("ROTO_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_ITERS);

    (seed, iters)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DummySubView<'a> {
    buf: &'a [u8],
    field_count: usize,
}

impl<'a> MessageView<'a> for DummySubView<'a> {
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut field_count = 0usize;
        for item in FieldIter::new(buf) {
            let field = item?;
            let _ = field.marker.slice(buf);
            field_count += 1;
        }
        Ok(Self { buf, field_count })
    }
}

fn push_varint(out: &mut Vec<u8>, mut val: u64) {
    while val >= 0x80 {
        out.push((val as u8) | 0x80);
        val >>= 7;
    }
    out.push(val as u8);
}

/// Generates a semi-structured wire stream combining valid, edge-case, and intentionally
/// malformed wire constructs (overlong varints, invalid wire types 6/7, deep groups up to
/// depth 70, mismatched group end tags, and truncated length-delimited slices).
fn generate_semi_structured_wire_bytes(rng: &mut Rng) -> Vec<u8> {
    let mut out = Vec::with_capacity(256);
    let ops = rng.gen_range(24);
    for _ in 0..ops {
        let tag: u32 = match rng.gen_range(8) {
            0 => 0, // invalid tag 0
            1..=4 => (rng.gen_range(5) as u32) + 1,
            5 => 16,
            6 => 0x1FFF_FFFF, // max valid protobuf tag
            _ => rng.next_u32(),
        };
        let wt = rng.gen_range(8) as u8;
        let raw_tag = ((tag as u64) << 3) | (wt as u64);
        push_varint(&mut out, raw_tag);

        match wt {
            0 => {
                // Varint: normal, overlong 10-byte, or overflowing 11-byte
                match rng.gen_range(6) {
                    0 => {
                        // Overlong 10-byte encoding
                        for _ in 0..9 {
                            out.push(0x80 | (rng.next_u32() as u8 & 0x7F));
                        }
                        out.push(rng.gen_range(4) as u8); // 0..=1 valid, 2..=3 overflow
                    }
                    1 => {
                        // 11 continuation bytes (invalid varint)
                        out.extend_from_slice(&[0x80; 11]);
                    }
                    _ => push_varint(&mut out, rng.edge_u64()),
                }
            }
            1 => {
                // Fixed64 (sometimes truncated at end)
                let len = if rng.gen_range(8) == 0 {
                    rng.gen_range(8)
                } else {
                    8
                };
                let start = out.len();
                out.resize(start + len, 0);
                rng.fill_bytes(&mut out[start..]);
            }
            2 => {
                // Length-delimited (packed values, submessage, or truncated)
                let len = rng.gen_range(32);
                let declared_len = if rng.gen_range(6) == 0 {
                    len + rng.gen_range(64)
                } else {
                    len
                };
                push_varint(&mut out, declared_len as u64);
                let start = out.len();
                out.resize(start + len, 0);
                rng.fill_bytes(&mut out[start..]);
            }
            3 => {
                // StartGroup: either matched, mismatched, unclosed, or deeply nested (1..=70)
                let depth = match rng.gen_range(6) {
                    0 => 60 + rng.gen_range(11), // 60..=70 (crosses MAX_GROUP_DEPTH = 64)
                    _ => 1 + rng.gen_range(5),
                };
                let mut group_tags = Vec::with_capacity(depth);
                let first_tag = if tag == 0 { 1 } else { tag };
                group_tags.push(first_tag);
                for _ in 1..depth {
                    let sub_tag = (rng.gen_range(4) as u32) + 1;
                    push_varint(&mut out, ((sub_tag as u64) << 3) | 3);
                    group_tags.push(sub_tag);
                }
                // Inner payload
                if rng.next_bool() {
                    push_varint(&mut out, (1u64 << 3) | 0);
                    push_varint(&mut out, rng.edge_u64());
                }
                // Close groups (possibly with a mismatch or truncation)
                for &gtag in group_tags.iter().rev() {
                    if rng.gen_range(16) == 0 {
                        break; // leave unclosed
                    }
                    let close_tag = if rng.gen_range(16) == 0 {
                        gtag.wrapping_add(1) // mismatched EndGroup tag
                    } else {
                        gtag
                    };
                    push_varint(&mut out, ((close_tag as u64) << 3) | 4);
                }
            }
            4 => {
                // Stray EndGroup
            }
            5 => {
                // Fixed32 (sometimes truncated)
                let len = if rng.gen_range(8) == 0 {
                    rng.gen_range(4)
                } else {
                    4
                };
                let start = out.len();
                out.resize(start + len, 0);
                rng.fill_bytes(&mut out[start..]);
            }
            _ => {
                // Invalid wire types 6 or 7
            }
        }
    }

    // Occasionally flip a random bit
    if !out.is_empty() && rng.gen_range(4) == 0 {
        let idx = rng.gen_range(out.len());
        let bit = 1u8 << rng.gen_range(8);
        out[idx] ^= bit;
    }

    out
}

fn assert_iter_never_panics_and_fuses<T, I>(mut iter: I, max_steps: usize)
where
    I: Iterator<Item = Result<T, DecodeError>>,
{
    let mut steps = 0usize;
    while let Some(res) = iter.next() {
        steps += 1;
        assert!(
            steps <= max_steps,
            "iterator exceeded max_steps ({max_steps}) without terminating"
        );
        if res.is_err() {
            assert!(
                iter.next().is_none(),
                "iterator failed to fuse immediately after yielding Err"
            );
            assert!(
                iter.next().is_none(),
                "iterator failed to remain fused on second next() call"
            );
            return;
        }
    }
    assert!(iter.next().is_none());
}

fn drive_all_runtime_iterators_on_slice(buf: &[u8]) {
    let max_steps = buf.len().saturating_add(16);

    // 1. decode_varint directly at every offset in small buffers or prefix
    let _ = decode_varint(buf);

    // 2. FieldIter + point decoders on yielded markers
    let mut field_iter = FieldIter::new(buf);
    let mut steps = 0usize;
    let mut prev_end = 0usize;
    while let Some(item) = field_iter.next() {
        steps += 1;
        assert!(steps <= max_steps, "FieldIter did not make forward progress");
        match item {
            Ok(field) => {
                assert!(field.tag >= 1);
                assert!(field.raw_span.is_present());
                assert!(field.marker.is_present());
                assert_eq!(field.raw_span.start() as usize, prev_end);
                assert!(field.raw_span.end() as usize <= buf.len());
                assert!(field.marker.start() >= field.raw_span.start());
                assert!(field.marker.end() <= field.raw_span.end());
                prev_end = field.raw_span.end() as usize;

                // Exercise point decoders on the field's marker regardless of wire type
                let _ = decode_varint_at(buf, field.marker);
                let _ = decode_sint32_at(buf, field.marker);
                let _ = decode_sint64_at(buf, field.marker);
                let _ = decode_bool_at(buf, field.marker);
                let _ = decode_fixed32_at(buf, field.marker);
                let _ = decode_sfixed32_at(buf, field.marker);
                let _ = decode_f32_at(buf, field.marker);
                let _ = decode_fixed64_at(buf, field.marker);
                let _ = decode_sfixed64_at(buf, field.marker);
                let _ = decode_f64_at(buf, field.marker);
                let _ = decode_bytes_at(buf, field.marker);
                let _ = decode_str_at(buf, field.marker);
            }
            Err(_) => {
                assert_eq!(field_iter.next(), None, "FieldIter must fuse after Err");
                assert_eq!(field_iter.next(), None, "FieldIter must remain fused");
                break;
            }
        }
    }
    assert_eq!(field_iter.next(), None);

    // 3. UnknownFieldsIter
    let mut unk_iter = UnknownFieldsIter::new(buf, &[1, 3, 5]);
    let mut unk_steps = 0usize;
    while let Some(span) = unk_iter.next() {
        unk_steps += 1;
        assert!(unk_steps <= max_steps);
        assert!(!span.is_empty());
        assert!(span.len() <= buf.len());
    }
    assert_eq!(unk_iter.next(), None);

    // 4. All Repeated*Iter types across tags 1, 2, 3
    for tag in [1u32, 2, 3] {
        assert_iter_never_panics_and_fuses(RepeatedVarintIter::<u64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedVarintIter::<u32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedVarintIter::<i32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedVarintIter::<i64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedVarintIter::<bool>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedZigZagIter::<i32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedZigZagIter::<i64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed32Iter::<u32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed32Iter::<i32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed32Iter::<f32>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed64Iter::<u64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed64Iter::<i64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedFixed64Iter::<f64>::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedBytesIter::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(RepeatedStrIter::new(buf, tag), max_steps);
        assert_iter_never_panics_and_fuses(
            RepeatedMessageIter::<DummySubView<'_>>::new(buf, tag),
            max_steps,
        );
        assert_iter_never_panics_and_fuses(
            RepeatedGroupIter::<DummySubView<'_>>::new(buf, tag),
            max_steps,
        );
    }
}

#[test]
fn fuzz_arbitrary_and_structured_wire_bytes_never_panic() {
    let (seed, iters) = read_fuzz_config();
    let mut rng = Rng::new(seed);

    // Always test empty slice first
    drive_all_runtime_iterators_on_slice(&[]);

    for iter_idx in 0..iters {
        let buf = if iter_idx % 2 == 0 {
            let len = rng.gen_range(1025);
            let mut b = vec![0u8; len];
            rng.fill_bytes(&mut b);
            b
        } else {
            generate_semi_structured_wire_bytes(&mut rng)
        };

        drive_all_runtime_iterators_on_slice(&buf);

        // Also test arbitrary FieldMarker bounds against `buf`
        let start = rng.gen_range(buf.len().saturating_add(8));
        let end = start.saturating_add(rng.gen_range(16));
        let marker = if rng.gen_range(8) == 0 {
            FieldMarker::none()
        } else {
            FieldMarker::new(start, end)
        };
        let _ = decode_varint_at(&buf, marker);
        let _ = decode_sint32_at(&buf, marker);
        let _ = decode_sint64_at(&buf, marker);
        let _ = decode_bool_at(&buf, marker);
        let _ = decode_fixed32_at(&buf, marker);
        let _ = decode_sfixed32_at(&buf, marker);
        let _ = decode_f32_at(&buf, marker);
        let _ = decode_fixed64_at(&buf, marker);
        let _ = decode_sfixed64_at(&buf, marker);
        let _ = decode_f64_at(&buf, marker);
        let _ = decode_bytes_at(&buf, marker);
        let _ = decode_str_at(&buf, marker);
    }
}

#[derive(Clone, Debug, PartialEq)]
enum WireValue {
    Varint(u64),
    ZigZag32(i32),
    ZigZag64(i64),
    Bool(bool),
    Fixed32(u32),
    Sfixed32(i32),
    FloatBits(u32),
    Fixed64(u64),
    Sfixed64(i64),
    DoubleBits(u64),
    Bytes(Vec<u8>),
    Group(Vec<(u32, WireValue)>),
}

impl WireValue {
    fn wire_type(&self) -> WireType {
        match self {
            Self::Varint(_) | Self::ZigZag32(_) | Self::ZigZag64(_) | Self::Bool(_) => {
                WireType::Varint
            }
            Self::Fixed32(_) | Self::Sfixed32(_) | Self::FloatBits(_) => WireType::Fixed32,
            Self::Fixed64(_) | Self::Sfixed64(_) | Self::DoubleBits(_) => WireType::Fixed64,
            Self::Bytes(_) => WireType::LengthDelimited,
            Self::Group(_) => WireType::StartGroup,
        }
    }
}

fn gen_random_wire_fields(rng: &mut Rng, depth: usize, max_fields: usize) -> Vec<(u32, WireValue)> {
    let count = rng.gen_range(max_fields + 1);
    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        let tag = match rng.gen_range(6) {
            0 => 1 + (rng.gen_range(15) as u32),        // 1-byte tag (1..=15)
            1 => 16 + (rng.gen_range(200) as u32),      // 2-byte tag
            2 => 2048 + (rng.gen_range(10_000) as u32), // 3-byte tag
            3 => 0x1FFF_FFFF,                           // max 29-bit protobuf field number
            _ => 1 + (rng.next_u32() & 0x0FFF_FFFF),
        };

        let max_choice = if depth < 3 { 12 } else { 11 };
        let val = match rng.gen_range(max_choice) {
            0 => WireValue::Varint(rng.edge_u64()),
            1 => WireValue::ZigZag32(rng.edge_i32()),
            2 => WireValue::ZigZag64(rng.edge_i64()),
            3 => WireValue::Bool(rng.next_bool()),
            4 => WireValue::Fixed32(rng.next_u32()),
            5 => WireValue::Sfixed32(rng.edge_i32()),
            6 => WireValue::FloatBits(rng.edge_f32().to_bits()),
            7 => WireValue::Fixed64(rng.edge_u64()),
            8 => WireValue::Sfixed64(rng.edge_i64()),
            9 => WireValue::DoubleBits(rng.edge_f64().to_bits()),
            10 => {
                let len = rng.gen_range(48);
                let mut b = vec![0u8; len];
                rng.fill_bytes(&mut b);
                WireValue::Bytes(b)
            }
            _ => WireValue::Group(gen_random_wire_fields(rng, depth + 1, 5)),
        };
        fields.push((tag, val));
    }
    fields
}

fn write_wire_fields_backward(writer: &mut BackwardWriter<'_>, fields: &[(u32, WireValue)]) {
    for (tag, val) in fields.iter().rev() {
        match val {
            WireValue::Varint(v) => {
                let _ = writer.write_varint(*v);
                let _ = writer.write_tag(*tag, WireType::Varint);
            }
            WireValue::ZigZag32(v) => {
                let _ = writer.write_zigzag32(*v);
                let _ = writer.write_tag(*tag, WireType::Varint);
            }
            WireValue::ZigZag64(v) => {
                let _ = writer.write_zigzag64(*v);
                let _ = writer.write_tag(*tag, WireType::Varint);
            }
            WireValue::Bool(v) => {
                let _ = writer.write_bool(*v);
                let _ = writer.write_tag(*tag, WireType::Varint);
            }
            WireValue::Fixed32(v) => {
                let _ = writer.write_fixed32(*v);
                let _ = writer.write_tag(*tag, WireType::Fixed32);
            }
            WireValue::Sfixed32(v) => {
                let _ = writer.write_sfixed32(*v);
                let _ = writer.write_tag(*tag, WireType::Fixed32);
            }
            WireValue::FloatBits(bits) => {
                let _ = writer.write_f32(f32::from_bits(*bits));
                let _ = writer.write_tag(*tag, WireType::Fixed32);
            }
            WireValue::Fixed64(v) => {
                let _ = writer.write_fixed64(*v);
                let _ = writer.write_tag(*tag, WireType::Fixed64);
            }
            WireValue::Sfixed64(v) => {
                let _ = writer.write_sfixed64(*v);
                let _ = writer.write_tag(*tag, WireType::Fixed64);
            }
            WireValue::DoubleBits(bits) => {
                let _ = writer.write_f64(f64::from_bits(*bits));
                let _ = writer.write_tag(*tag, WireType::Fixed64);
            }
            WireValue::Bytes(b) => {
                let _ = writer.write_length_delimited(b);
                let _ = writer.write_tag(*tag, WireType::LengthDelimited);
            }
            WireValue::Group(inner) => {
                let _ = writer.write_tag(*tag, WireType::EndGroup);
                writer.with_sub_writer(|mut sub| {
                    write_wire_fields_backward(&mut sub, inner);
                    sub
                });
                let _ = writer.write_tag(*tag, WireType::StartGroup);
            }
        }
    }
}

fn verify_wire_fields_forward(buf: &[u8], expected: &[(u32, WireValue)]) {
    let parsed: Vec<_> = FieldIter::new(buf)
        .map(|r| r.expect("valid BackwardWriter output must parse without DecodeError"))
        .collect();
    assert_eq!(parsed.len(), expected.len());

    let mut expected_offset = 0usize;
    for (field, (exp_tag, exp_val)) in parsed.iter().zip(expected.iter()) {
        assert_eq!(field.tag, *exp_tag);
        assert_eq!(field.wire_type, exp_val.wire_type());
        assert_eq!(field.raw_span.start() as usize, expected_offset);
        expected_offset = field.raw_span.end() as usize;

        match exp_val {
            WireValue::Varint(v) => {
                assert_eq!(decode_varint_at(buf, field.marker), Ok(*v));
            }
            WireValue::ZigZag32(v) => {
                assert_eq!(decode_sint32_at(buf, field.marker), Ok(*v));
            }
            WireValue::ZigZag64(v) => {
                assert_eq!(decode_sint64_at(buf, field.marker), Ok(*v));
            }
            WireValue::Bool(v) => {
                assert_eq!(decode_bool_at(buf, field.marker), Ok(*v));
            }
            WireValue::Fixed32(v) => {
                assert_eq!(decode_fixed32_at(buf, field.marker), Ok(*v));
            }
            WireValue::Sfixed32(v) => {
                assert_eq!(decode_sfixed32_at(buf, field.marker), Ok(*v));
            }
            WireValue::FloatBits(bits) => {
                let decoded = decode_f32_at(buf, field.marker).unwrap();
                assert_eq!(decoded.to_bits(), *bits);
            }
            WireValue::Fixed64(v) => {
                assert_eq!(decode_fixed64_at(buf, field.marker), Ok(*v));
            }
            WireValue::Sfixed64(v) => {
                assert_eq!(decode_sfixed64_at(buf, field.marker), Ok(*v));
            }
            WireValue::DoubleBits(bits) => {
                let decoded = decode_f64_at(buf, field.marker).unwrap();
                assert_eq!(decoded.to_bits(), *bits);
            }
            WireValue::Bytes(b) => {
                assert_eq!(decode_bytes_at(buf, field.marker), Ok(b.as_slice()));
            }
            WireValue::Group(inner) => {
                let group_body = field.marker.slice(buf);
                verify_wire_fields_forward(group_body, inner);
            }
        }
    }
    assert_eq!(expected_offset, buf.len());
}

#[test]
fn fuzz_backward_writer_and_field_iter_roundtrip() {
    let (seed, iters) = read_fuzz_config();
    let mut rng = Rng::new(seed ^ 0xA5A5_5A5A_F0F0_0F0F);
    let mut backing = vec![0u8; 16 * 1024];

    for _ in 0..iters {
        let fields = gen_random_wire_fields(&mut rng, 0, 20);

        // 1. Unparse via BackwardWriter
        let mut writer = BackwardWriter::new(&mut backing);
        write_wire_fields_backward(&mut writer, &fields);
        assert_eq!(writer.check(), Ok(()));
        let wire_bytes = writer.finish().expect("16 KiB buffer must fit test fields").to_vec();

        // 2. Parse via FieldIter & verify exact round-trip
        verify_wire_fields_forward(&wire_bytes, &fields);

        // 3. Exact-capacity & 1-byte-short capacity checks
        let mut exact_buf = vec![0u8; wire_bytes.len()];
        let mut exact_writer = BackwardWriter::new(&mut exact_buf);
        write_wire_fields_backward(&mut exact_writer, &fields);
        assert_eq!(exact_writer.finish(), Ok(wire_bytes.as_slice()));

        if !wire_bytes.is_empty() {
            let short_len = rng.gen_range(wire_bytes.len());
            let mut short_buf = vec![0u8; short_len];
            let mut short_writer = BackwardWriter::new(&mut short_buf);
            write_wire_fields_backward(&mut short_writer, &fields);
            assert_eq!(short_writer.check(), Err(EncodeError::BufferOverflow));
            assert_eq!(short_writer.finish(), Err(EncodeError::BufferOverflow));
        }

        // 4. Verify copy_filtered_fields preserves order for a random predicate
        let parity = rng.next_bool();
        let expected_filtered: Vec<(u32, WireValue)> = fields
            .iter()
            .filter(|(tag, _)| ((*tag & 1) == 0) == parity)
            .cloned()
            .collect();

        let mut filtered_backing = vec![0u8; wire_bytes.len() + 16];
        let mut filtered_writer = BackwardWriter::new(&mut filtered_backing);
        filtered_writer
            .copy_filtered_fields(&wire_bytes, |tag| ((tag & 1) == 0) == parity)
            .expect("filtered copy must fit");
        let filtered_bytes = filtered_writer.finish().unwrap();
        verify_wire_fields_forward(filtered_bytes, &expected_filtered);
    }
}
