#![no_std]
#![forbid(unsafe_code)]

pub mod error;
pub mod iter;
pub mod repeated;
pub mod wire;
pub mod writer;

pub use error::{DecodeError, EncodeError};
pub use iter::{Field, FieldIter};
pub use repeated::{
    Fixed32Value, Fixed64Value, MessageView, RepeatedBytesIter, RepeatedFixed32Iter,
    RepeatedFixed64Iter, RepeatedMessageIter, RepeatedStrIter, RepeatedVarintIter,
    RepeatedZigZagIter, UnknownFieldsIter, VarintValue, ZigZagValue,
};
pub use wire::{
    FieldMarker, WireType, decode_bool_at, decode_bytes_at, decode_f32_at, decode_f64_at,
    decode_fixed32_at, decode_fixed64_at, decode_sfixed32_at, decode_sfixed64_at,
    decode_sint32_at, decode_sint64_at, decode_str_at, decode_varint, decode_varint_at,
    decode_zigzag32, decode_zigzag64,
};
pub use writer::{BackwardWriter, encode_zigzag32, encode_zigzag64};

/// Compile-time and runtime verification marker confirming `roto-runtime` is linked.
#[inline]
pub const fn runtime_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;
    use core::mem::size_of;

    struct FixedBuf<const N: usize> {
        buf: [u8; N],
        len: usize,
    }

    impl<const N: usize> FixedBuf<N> {
        const fn new() -> Self {
            Self {
                buf: [0u8; N],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buf[..self.len]).unwrap()
        }
    }

    impl<const N: usize> Write for FixedBuf<N> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if self.len + bytes.len() > N {
                return Err(core::fmt::Error);
            }
            self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
            Ok(())
        }
    }

    #[test]
    fn runtime_is_ready() {
        assert!(runtime_ready());
    }

    #[test]
    fn field_marker_layout_and_operations() {
        assert_eq!(size_of::<FieldMarker>(), 8);

        let absent = FieldMarker::none();
        assert_eq!(absent, FieldMarker::default());
        assert!(!absent.is_present());
        assert_eq!(absent.len(), 0);
        assert!(absent.is_empty());
        assert_eq!(absent.slice(&[1, 2, 3]), &[]);

        let empty_present = FieldMarker::new(2, 2);
        assert!(empty_present.is_present());
        assert_eq!(empty_present.start(), 2);
        assert_eq!(empty_present.end(), 2);
        assert_eq!(empty_present.len(), 0);
        assert!(empty_present.is_empty());
        assert_eq!(empty_present.slice(&[10, 20, 30]), &[]);

        let span = FieldMarker::new(1, 4);
        assert!(span.is_present());
        assert_eq!(span.start(), 1);
        assert_eq!(span.end(), 4);
        assert_eq!(span.len(), 3);
        assert!(!span.is_empty());
        assert_eq!(span.slice(&[10, 20, 30, 40, 50]), &[20, 30, 40]);
    }

    #[test]
    fn wire_type_conversions() {
        assert_eq!(WireType::from_u8(0), Ok(WireType::Varint));
        assert_eq!(WireType::from_u8(1), Ok(WireType::Fixed64));
        assert_eq!(WireType::from_u8(2), Ok(WireType::LengthDelimited));
        assert_eq!(WireType::from_u8(3), Ok(WireType::StartGroup));
        assert_eq!(WireType::from_u8(4), Ok(WireType::EndGroup));
        assert_eq!(WireType::from_u8(5), Ok(WireType::Fixed32));
        assert_eq!(WireType::from_u8(6), Err(DecodeError::InvalidWireType(6)));
        assert_eq!(WireType::from_u8(7), Err(DecodeError::InvalidWireType(7)));
        assert_eq!(
            WireType::from_u8(255),
            Err(DecodeError::InvalidWireType(255))
        );
    }

    #[test]
    fn error_display_and_trait() {
        fn assert_error<E: core::error::Error>(_: &E) {}

        let errors = [
            (DecodeError::UnexpectedEof, "unexpected end of buffer"),
            (
                DecodeError::InvalidVarint,
                "invalid or overflowing varint",
            ),
            (DecodeError::InvalidWireType(6), "invalid wire type: 6"),
            (DecodeError::InvalidUtf8, "invalid UTF-8 string"),
            (
                DecodeError::MissingRequiredField("Location.lat_e7"),
                "missing required field: Location.lat_e7",
            ),
            (
                DecodeError::UnexpectedWireType {
                    expected: WireType::Varint,
                    actual: WireType::Fixed32,
                },
                "unexpected wire type: expected Varint, got Fixed32",
            ),
        ];

        for (err, expected) in errors {
            assert_error(&err);
            let mut out = FixedBuf::<128>::new();
            write!(&mut out, "{err}").unwrap();
            assert_eq!(out.as_str(), expected);
        }

        let enc_err = EncodeError::BufferOverflow;
        assert_error(&enc_err);
        let mut out = FixedBuf::<64>::new();
        write!(&mut out, "{enc_err}").unwrap();
        assert_eq!(out.as_str(), "encode buffer overflow");
    }

    #[test]
    fn varint_decoding_valid_and_edge_cases() {
        // 0
        assert_eq!(decode_varint(&[0x00]), Ok((0, 1)));
        // 1
        assert_eq!(decode_varint(&[0x01]), Ok((1, 1)));
        // 127
        assert_eq!(decode_varint(&[0x7F]), Ok((127, 1)));
        // 128 -> [0x80, 0x01]
        assert_eq!(decode_varint(&[0x80, 0x01]), Ok((128, 2)));
        // 300 -> [0xAC, 0x02], with trailing bytes ignored
        assert_eq!(decode_varint(&[0xAC, 0x02, 0xFF]), Ok((300, 2)));
        // u32::MAX -> 5 bytes
        let u32_max_bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        assert_eq!(decode_varint(&u32_max_bytes), Ok((u32::MAX as u64, 5)));
        // u64::MAX / -1i64 / negative int32 sign-extended to 10 bytes
        let u64_max_bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(decode_varint(&u64_max_bytes), Ok((u64::MAX, 10)));
        assert_eq!(
            decode_varint_at(&u64_max_bytes, FieldMarker::new(0, 10)).unwrap() as i64,
            -1i64
        );
        assert_eq!(
            decode_varint_at(&u64_max_bytes, FieldMarker::new(0, 10)).unwrap() as i32,
            -1i32
        );
        // -2 sign-extended to 10 bytes
        let neg2_bytes = [0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(
            decode_varint_at(&neg2_bytes, FieldMarker::new(0, 10)).unwrap() as i32,
            -2i32
        );
        // Overlong 10-byte encoding of 0
        let overlong_zero = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        assert_eq!(decode_varint(&overlong_zero), Ok((0, 10)));

        // Errors:
        // Empty buffer
        assert_eq!(decode_varint(&[]), Err(DecodeError::UnexpectedEof));
        // Truncated varints
        assert_eq!(decode_varint(&[0x80]), Err(DecodeError::UnexpectedEof));
        assert_eq!(
            decode_varint(&[0xFF; 9]),
            Err(DecodeError::UnexpectedEof)
        );
        // 10th byte overflow (> 0x01 without continuation bit)
        let overflow_10th = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02];
        assert_eq!(
            decode_varint(&overflow_10th),
            Err(DecodeError::InvalidVarint)
        );
        // 10th byte with continuation bit set (11-byte varint)
        let cont_10th = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        assert_eq!(decode_varint(&cont_10th), Err(DecodeError::InvalidVarint));

        // decode_varint_at with invalid markers
        assert_eq!(
            decode_varint_at(&[0x01], FieldMarker::none()),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_varint_at(&[0x01], FieldMarker::new(0, 5)),
            Err(DecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn zigzag_and_sint_decoding() {
        assert_eq!(decode_zigzag32(0), 0);
        assert_eq!(decode_zigzag32(1), -1);
        assert_eq!(decode_zigzag32(2), 1);
        assert_eq!(decode_zigzag32(3), -2);
        assert_eq!(decode_zigzag32(4), 2);
        assert_eq!(decode_zigzag32(0xFFFFFFFE), i32::MAX);
        assert_eq!(decode_zigzag32(0xFFFFFFFF), i32::MIN);

        assert_eq!(decode_zigzag64(0), 0);
        assert_eq!(decode_zigzag64(1), -1);
        assert_eq!(decode_zigzag64(2), 1);
        assert_eq!(decode_zigzag64(3), -2);
        assert_eq!(decode_zigzag64(4), 2);
        assert_eq!(decode_zigzag64(0xFFFFFFFFFFFFFFFE), i64::MAX);
        assert_eq!(decode_zigzag64(0xFFFFFFFFFFFFFFFF), i64::MIN);

        let buf = [0x01, 0x02, 0x03];
        assert_eq!(decode_sint32_at(&buf, FieldMarker::new(0, 1)), Ok(-1));
        assert_eq!(decode_sint32_at(&buf, FieldMarker::new(1, 2)), Ok(1));
        assert_eq!(decode_sint64_at(&buf, FieldMarker::new(2, 3)), Ok(-2));
    }

    #[test]
    fn bool_decoding() {
        let buf = [0x00, 0x01, 0xAC, 0x02];
        assert_eq!(decode_bool_at(&buf, FieldMarker::new(0, 1)), Ok(false));
        assert_eq!(decode_bool_at(&buf, FieldMarker::new(1, 2)), Ok(true));
        assert_eq!(decode_bool_at(&buf, FieldMarker::new(2, 4)), Ok(true));
        assert_eq!(
            decode_bool_at(&buf, FieldMarker::none()),
            Err(DecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn fixed_and_float_decoding() {
        let mut buf = [0u8; 12];
        buf[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        buf[4..12].copy_from_slice(&0x0123456789ABCDEFu64.to_le_bytes());

        assert_eq!(
            decode_fixed32_at(&buf, FieldMarker::new(0, 4)),
            Ok(0xDEADBEEF)
        );
        assert_eq!(
            decode_sfixed32_at(&buf, FieldMarker::new(0, 4)),
            Ok(0xDEADBEEFu32 as i32)
        );
        assert_eq!(
            decode_fixed64_at(&buf, FieldMarker::new(4, 12)),
            Ok(0x0123456789ABCDEF)
        );
        assert_eq!(
            decode_sfixed64_at(&buf, FieldMarker::new(4, 12)),
            Ok(0x0123456789ABCDEFu64 as i64)
        );

        // Floats
        let f32_val: f32 = -42.5;
        let f64_val: f64 = 123456.7890123;
        buf[0..4].copy_from_slice(&f32_val.to_bits().to_le_bytes());
        buf[4..12].copy_from_slice(&f64_val.to_bits().to_le_bytes());
        assert_eq!(decode_f32_at(&buf, FieldMarker::new(0, 4)), Ok(f32_val));
        assert_eq!(decode_f64_at(&buf, FieldMarker::new(4, 12)), Ok(f64_val));

        // Wrong length or absent markers return UnexpectedEof
        assert_eq!(
            decode_fixed32_at(&buf, FieldMarker::new(0, 3)),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_fixed32_at(&buf, FieldMarker::new(0, 5)),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_fixed64_at(&buf, FieldMarker::new(4, 11)),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_fixed32_at(&buf, FieldMarker::none()),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_fixed64_at(&buf, FieldMarker::none()),
            Err(DecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn bytes_and_str_decoding() {
        let buf = b"hello\xFFworld";
        assert_eq!(
            decode_bytes_at(buf, FieldMarker::new(0, 5)),
            Ok(&b"hello"[..])
        );
        assert_eq!(decode_str_at(buf, FieldMarker::new(0, 5)), Ok("hello"));
        assert_eq!(decode_str_at(buf, FieldMarker::new(6, 11)), Ok("world"));
        assert_eq!(
            decode_str_at(buf, FieldMarker::new(0, 6)),
            Err(DecodeError::InvalidUtf8)
        );
        assert_eq!(
            decode_bytes_at(buf, FieldMarker::none()),
            Err(DecodeError::UnexpectedEof)
        );
        assert_eq!(
            decode_str_at(buf, FieldMarker::new(0, 100)),
            Err(DecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn field_iter_empty_buffer() {
        let mut iter = FieldIter::new(&[]);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn field_iter_multi_field_buffer() {
        // Field 1: tag=1, Varint (0), tag byte = 0x08, value = 300 ([0xAC, 0x02]) -> 0..3
        // Field 2: tag=2, Fixed64 (1), tag byte = 0x11, value = 8 bytes -> 3..12
        // Field 3: tag=3, LengthDelimited (2), tag byte = 0x1A, len = 4 (0x04), payload = b"roto" -> 12..18
        // Field 4: tag=16, Fixed32 (5), tag = (16 << 3) | 5 = 133 ([0x85, 0x01]), value = 4 bytes -> 18..24
        let buf: &[u8] = &[
            0x08, 0xAC, 0x02, // 0..3
            0x11, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // 3..12
            0x1A, 0x04, b'r', b'o', b't', b'o', // 12..18
            0x85, 0x01, 0xDE, 0xAD, 0xBE, 0xEF, // 18..24
        ];

        let mut iter = FieldIter::new(buf);

        let f1 = iter.next().unwrap().unwrap();
        assert_eq!(
            f1,
            Field {
                tag: 1,
                wire_type: WireType::Varint,
                marker: FieldMarker::new(1, 3),
                raw_span: FieldMarker::new(0, 3),
            }
        );
        assert_eq!(f1.marker.slice(buf), &[0xAC, 0x02]);
        assert_eq!(f1.raw_span.slice(buf), &[0x08, 0xAC, 0x02]);
        assert_eq!(decode_varint_at(buf, f1.marker), Ok(300));

        let f2 = iter.next().unwrap().unwrap();
        assert_eq!(
            f2,
            Field {
                tag: 2,
                wire_type: WireType::Fixed64,
                marker: FieldMarker::new(4, 12),
                raw_span: FieldMarker::new(3, 12),
            }
        );
        assert_eq!(
            f2.marker.slice(buf),
            &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
        assert_eq!(f2.raw_span.slice(buf), &buf[3..12]);
        assert_eq!(decode_fixed64_at(buf, f2.marker), Ok(0x0807060504030201));

        let f3 = iter.next().unwrap().unwrap();
        assert_eq!(
            f3,
            Field {
                tag: 3,
                wire_type: WireType::LengthDelimited,
                marker: FieldMarker::new(14, 18),
                raw_span: FieldMarker::new(12, 18),
            }
        );
        assert_eq!(f3.marker.slice(buf), b"roto");
        assert_eq!(
            f3.raw_span.slice(buf),
            &[0x1A, 0x04, b'r', b'o', b't', b'o']
        );
        assert_eq!(decode_str_at(buf, f3.marker), Ok("roto"));

        let f4 = iter.next().unwrap().unwrap();
        assert_eq!(
            f4,
            Field {
                tag: 16,
                wire_type: WireType::Fixed32,
                marker: FieldMarker::new(20, 24),
                raw_span: FieldMarker::new(18, 24),
            }
        );
        assert_eq!(f4.marker.slice(buf), &[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(
            f4.raw_span.slice(buf),
            &[0x85, 0x01, 0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert_eq!(decode_fixed32_at(buf, f4.marker), Ok(0xEFBEADDE));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn field_iter_nested_length_delimited_messages() {
        // Inner message: field 1 varint 42 ([0x08, 0x2A]), field 2 string "hi" ([0x12, 0x02, b'h', b'i']) -> 6 bytes
        // Outer message: field 5 LengthDelimited (0x2A, len 0x06, inner), followed by field 6 varint 7 ([0x30, 0x07])
        let buf: &[u8] = &[
            0x2A, 0x06, // outer tag=5, len=6 (0..2)
            0x08, 0x2A, // inner field 1 = 42 (2..4)
            0x12, 0x02, b'h', b'i', // inner field 2 = "hi" (4..8)
            0x30, 0x07, // outer field 6 = 7 (8..10)
        ];

        let mut outer_iter = FieldIter::new(buf);
        let outer_f1 = outer_iter.next().unwrap().unwrap();
        assert_eq!(outer_f1.tag, 5);
        assert_eq!(outer_f1.wire_type, WireType::LengthDelimited);
        assert_eq!(outer_f1.marker, FieldMarker::new(2, 8));
        assert_eq!(outer_f1.raw_span, FieldMarker::new(0, 8));

        let inner_buf = outer_f1.marker.slice(buf);
        assert_eq!(inner_buf, &[0x08, 0x2A, 0x12, 0x02, b'h', b'i']);

        let mut inner_iter = FieldIter::new(inner_buf);
        let inner_f1 = inner_iter.next().unwrap().unwrap();
        assert_eq!(inner_f1.tag, 1);
        assert_eq!(decode_varint_at(inner_buf, inner_f1.marker), Ok(42));
        let inner_f2 = inner_iter.next().unwrap().unwrap();
        assert_eq!(inner_f2.tag, 2);
        assert_eq!(decode_str_at(inner_buf, inner_f2.marker), Ok("hi"));
        assert_eq!(inner_iter.next(), None);

        let outer_f2 = outer_iter.next().unwrap().unwrap();
        assert_eq!(outer_f2.tag, 6);
        assert_eq!(decode_varint_at(buf, outer_f2.marker), Ok(7));
        assert_eq!(outer_iter.next(), None);
    }

    #[test]
    fn field_iter_groups() {
        // 1. Empty group: tag=1 StartGroup (0x0B), tag=1 EndGroup (0x0C)
        let empty_group = &[0x0B, 0x0C];
        let mut iter = FieldIter::new(empty_group);
        let f = iter.next().unwrap().unwrap();
        assert_eq!(
            f,
            Field {
                tag: 1,
                wire_type: WireType::StartGroup,
                marker: FieldMarker::new(1, 1),
                raw_span: FieldMarker::new(0, 2),
            }
        );
        assert_eq!(f.marker.slice(empty_group), &[]);
        assert_eq!(f.raw_span.slice(empty_group), &[0x0B, 0x0C]);
        assert_eq!(iter.next(), None);

        // 2. Single and nested groups with inner fields of all wire types
        // Outer group tag=1 StartGroup (0x0B) at 0
        //   Inner varint tag=2 (0x10), val=5 (0x05) at 1..3
        //   Inner nested group tag=3 StartGroup (0x1B) at 3
        //     Inner fixed32 tag=4 (0x25), val=[1,2,3,4] at 4..9
        //     Inner length-delimited tag=5 (0x2A), len=2 (0x02), val=b"ok" at 9..13
        //     Inner fixed64 tag=6 (0x31), val=[0; 8] at 13..22
        //   Inner nested group tag=3 EndGroup (0x1C) at 22..23
        // Outer group tag=1 EndGroup (0x0C) at 23..24
        // Trailing field tag=7 Varint (0x38), val=99 (0x63) at 24..26
        let nested_buf: &[u8] = &[
            0x0B, // 0..1: StartGroup(1)
            0x10, 0x05, // 1..3: Varint(2) = 5
            0x1B, // 3..4: StartGroup(3)
            0x25, 1, 2, 3, 4, // 4..9: Fixed32(4)
            0x2A, 0x02, b'o', b'k', // 9..13: LengthDelimited(5)
            0x31, 0, 0, 0, 0, 0, 0, 0, 0,    // 13..22: Fixed64(6)
            0x1C, // 22..23: EndGroup(3)
            0x0C, // 23..24: EndGroup(1)
            0x38, 0x63, // 24..26: Varint(7) = 99
        ];
        let mut iter = FieldIter::new(nested_buf);
        let g = iter.next().unwrap().unwrap();
        assert_eq!(
            g,
            Field {
                tag: 1,
                wire_type: WireType::StartGroup,
                marker: FieldMarker::new(1, 23),
                raw_span: FieldMarker::new(0, 24),
            }
        );
        assert_eq!(g.marker.slice(nested_buf), &nested_buf[1..23]);
        assert_eq!(g.raw_span.slice(nested_buf), &nested_buf[0..24]);

        // Scan interior of outer group
        let mut group_iter = FieldIter::new(g.marker.slice(nested_buf));
        let gf1 = group_iter.next().unwrap().unwrap();
        assert_eq!(gf1.tag, 2);
        assert_eq!(gf1.wire_type, WireType::Varint);
        let gf2 = group_iter.next().unwrap().unwrap();
        assert_eq!(gf2.tag, 3);
        assert_eq!(gf2.wire_type, WireType::StartGroup);
        assert_eq!(gf2.marker, FieldMarker::new(3, 21));
        assert_eq!(gf2.raw_span, FieldMarker::new(2, 22));
        assert_eq!(group_iter.next(), None);

        let trailing = iter.next().unwrap().unwrap();
        assert_eq!(trailing.tag, 7);
        assert_eq!(decode_varint_at(nested_buf, trailing.marker), Ok(99));
        assert_eq!(iter.next(), None);

        // 3. Mismatched group end tag (StartGroup(1) closed by EndGroup(2) = 0x14)
        let mismatched = &[0x0B, 0x14, 0x08, 0x01];
        let mut iter = FieldIter::new(mismatched);
        assert_eq!(iter.next(), Some(Err(DecodeError::InvalidWireType(4))));
        assert_eq!(iter.next(), None);

        // 4. Mismatched inner group end tag (StartGroup(1), StartGroup(2), EndGroup(1))
        let mismatched_inner = &[0x0B, 0x13, 0x0C];
        let mut iter = FieldIter::new(mismatched_inner);
        assert_eq!(iter.next(), Some(Err(DecodeError::InvalidWireType(4))));
        assert_eq!(iter.next(), None);

        // 5. Unclosed group EOF
        let unclosed = &[0x0B, 0x10, 0x05];
        let mut iter = FieldIter::new(unclosed);
        assert_eq!(iter.next(), Some(Err(DecodeError::UnexpectedEof)));
        assert_eq!(iter.next(), None);

        // 6. Unmatched top-level EndGroup (tag=1, EndGroup = 0x0C)
        let top_end_group = &[0x0C, 0x08, 0x01];
        let mut iter = FieldIter::new(top_end_group);
        assert_eq!(iter.next(), Some(Err(DecodeError::InvalidWireType(4))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn field_iter_errors_and_fusing() {
        let cases: &[(&[u8], DecodeError)] = &[
            // Invalid tag 0 (raw = 0x00 -> field_number = 0, wire_type = 0)
            (&[0x00, 0x01], DecodeError::InvalidVarint),
            // Tag > u32::MAX: raw = ((u32::MAX as u64 + 1) << 3) | 0 = 1 << 35
            // 35 bits -> 6 bytes: 5 continuation bytes of 0x80 followed by 0x01
            (
                &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01, 0x00],
                DecodeError::InvalidVarint,
            ),
            // Invalid wire type 6 (tag=1, wt=6 -> 0x0E)
            (&[0x0E, 0x00], DecodeError::InvalidWireType(6)),
            // Invalid wire type 7 (tag=1, wt=7 -> 0x0F)
            (&[0x0F, 0x00], DecodeError::InvalidWireType(7)),
            // Truncated varint tag
            (&[0x80], DecodeError::UnexpectedEof),
            // Overflowing 10-byte varint tag
            (
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02],
                DecodeError::InvalidVarint,
            ),
            // Truncated varint value
            (&[0x08, 0x80], DecodeError::UnexpectedEof),
            // Overflowing varint value
            (
                &[
                    0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02,
                ],
                DecodeError::InvalidVarint,
            ),
            // Truncated fixed32 (only 3 bytes after tag 0x0D)
            (&[0x0D, 1, 2, 3], DecodeError::UnexpectedEof),
            // Truncated fixed64 (only 7 bytes after tag 0x09)
            (&[0x09, 1, 2, 3, 4, 5, 6, 7], DecodeError::UnexpectedEof),
            // Truncated length-delimited payload (claims 5 bytes, only 3 available)
            (&[0x12, 0x05, 1, 2, 3], DecodeError::UnexpectedEof),
            // Truncated length varint in length-delimited field
            (&[0x12, 0x80], DecodeError::UnexpectedEof),
            // Length varint overflow (u64::MAX length)
            (
                &[
                    0x12, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00,
                ],
                DecodeError::UnexpectedEof,
            ),
        ];

        for &(buf, expected_err) in cases {
            let mut iter = FieldIter::new(buf);
            assert_eq!(iter.next(), Some(Err(expected_err)));
            // Must fuse and return None on subsequent calls
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestEnum {
        Zero,
        One,
        Two,
        Unknown(i32),
    }

    impl VarintValue for TestEnum {
        fn from_raw_varint(val: u64) -> Self {
            match val as i32 {
                0 => Self::Zero,
                1 => Self::One,
                2 => Self::Two,
                other => Self::Unknown(other),
            }
        }
    }

    #[test]
    fn repeated_varint_iter_unpacked_packed_mixed_and_types() {
        // Interleaved buffer for tag 1:
        // - tag 1 Varint: 10 (0x08, 0x0A)
        // - tag 2 Varint (ignored): 99 (0x10, 0x63)
        // - tag 1 LengthDelimited empty packed blob: (0x0A, 0x00)
        // - tag 1 LengthDelimited packed [300, 1]: (0x0A, 0x03, 0xAC, 0x02, 0x01)
        // - tag 1 Varint: 0 (0x08, 0x00)
        // - tag 1 LengthDelimited packed [2]: (0x0A, 0x01, 0x02)
        let buf: &[u8] = &[
            0x08, 0x0A, // unpacked 10
            0x10, 0x63, // tag 2 = 99
            0x0A, 0x00, // empty packed
            0x0A, 0x03, 0xAC, 0x02, 0x01, // packed [300, 1]
            0x08, 0x00, // unpacked 0
            0x0A, 0x01, 0x02, // packed [2]
        ];

        // u32
        let mut iter_u32 = RepeatedVarintIter::<u32>::new(buf, 1);
        assert_eq!(iter_u32.next(), Some(Ok(10)));
        assert_eq!(iter_u32.next(), Some(Ok(300)));
        assert_eq!(iter_u32.next(), Some(Ok(1)));
        assert_eq!(iter_u32.next(), Some(Ok(0)));
        assert_eq!(iter_u32.next(), Some(Ok(2)));
        assert_eq!(iter_u32.next(), None);
        assert_eq!(iter_u32.next(), None);

        // u64
        let mut iter_u64 = RepeatedVarintIter::<u64>::new(buf, 1);
        assert_eq!(iter_u64.next(), Some(Ok(10)));
        assert_eq!(iter_u64.next(), Some(Ok(300)));
        assert_eq!(iter_u64.next(), Some(Ok(1)));
        assert_eq!(iter_u64.next(), Some(Ok(0)));
        assert_eq!(iter_u64.next(), Some(Ok(2)));
        assert_eq!(iter_u64.next(), None);

        // bool
        let mut iter_bool = RepeatedVarintIter::<bool>::new(buf, 1);
        assert_eq!(iter_bool.next(), Some(Ok(true)));
        assert_eq!(iter_bool.next(), Some(Ok(true)));
        assert_eq!(iter_bool.next(), Some(Ok(true)));
        assert_eq!(iter_bool.next(), Some(Ok(false)));
        assert_eq!(iter_bool.next(), Some(Ok(true)));
        assert_eq!(iter_bool.next(), None);

        // Custom enum implementing VarintValue
        let mut iter_enum = RepeatedVarintIter::<TestEnum>::new(buf, 1);
        assert_eq!(iter_enum.next(), Some(Ok(TestEnum::Unknown(10))));
        assert_eq!(iter_enum.next(), Some(Ok(TestEnum::Unknown(300))));
        assert_eq!(iter_enum.next(), Some(Ok(TestEnum::One)));
        assert_eq!(iter_enum.next(), Some(Ok(TestEnum::Zero)));
        assert_eq!(iter_enum.next(), Some(Ok(TestEnum::Two)));
        assert_eq!(iter_enum.next(), None);

        // Negative i32 / i64 (10-byte sign-extended -1 followed by positive 42 in packed)
        let neg_buf: &[u8] = &[
            0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, // unpacked -1
            0x0A, 0x0B, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
            0x2A, // packed [-2, 42]
        ];
        let mut iter_i32 = RepeatedVarintIter::<i32>::new(neg_buf, 1);
        assert_eq!(iter_i32.next(), Some(Ok(-1)));
        assert_eq!(iter_i32.next(), Some(Ok(-2)));
        assert_eq!(iter_i32.next(), Some(Ok(42)));
        assert_eq!(iter_i32.next(), None);

        let mut iter_i64 = RepeatedVarintIter::<i64>::new(neg_buf, 1);
        assert_eq!(iter_i64.next(), Some(Ok(-1)));
        assert_eq!(iter_i64.next(), Some(Ok(-2)));
        assert_eq!(iter_i64.next(), Some(Ok(42)));
        assert_eq!(iter_i64.next(), None);
    }

    #[test]
    fn repeated_varint_iter_errors_and_fusing() {
        // Unexpected wire type (tag 1 Fixed32 = 0x0D) followed by valid tag 1 Varint
        let bad_wt: &[u8] = &[0x0D, 1, 2, 3, 4, 0x08, 0x01];
        let mut iter = RepeatedVarintIter::<u32>::new(bad_wt, 1);
        assert_eq!(
            iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::Varint,
                actual: WireType::Fixed32,
            }))
        );
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Truncated varint inside packed payload: first element 5, second element 0x80 (truncated)
        // followed by valid unpacked tag 1 = 7
        let truncated_packed: &[u8] = &[0x0A, 0x02, 0x05, 0x80, 0x08, 0x07];
        let mut iter = RepeatedVarintIter::<u32>::new(truncated_packed, 1);
        assert_eq!(iter.next(), Some(Ok(5)));
        assert_eq!(iter.next(), Some(Err(DecodeError::UnexpectedEof)));
        assert_eq!(iter.next(), None);

        // Overflowing 10-byte varint inside packed payload
        let overflow_packed: &[u8] = &[
            0x0A, 0x0A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0x08, 0x01,
        ];
        let mut iter = RepeatedVarintIter::<u64>::new(overflow_packed, 1);
        assert_eq!(iter.next(), Some(Err(DecodeError::InvalidVarint)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn repeated_zigzag_iter_unpacked_packed_and_errors() {
        // sint32 / sint64 on tag 2:
        // - unpacked tag 2 (0x10): 1 -> -1
        // - empty packed tag 2 (0x12, 0x00)
        // - packed tag 2 (0x12, len=3): [0 -> 0, 2 -> 1, 3 -> -2]
        // - unpacked tag 2 (0x10): 4 -> 2
        let buf: &[u8] = &[
            0x10, 0x01, // -1
            0x12, 0x00, // empty packed
            0x12, 0x03, 0x00, 0x02, 0x03, // [0, 1, -2]
            0x10, 0x04, // 2
        ];

        let mut iter32 = RepeatedZigZagIter::<i32>::new(buf, 2);
        assert_eq!(iter32.next(), Some(Ok(-1)));
        assert_eq!(iter32.next(), Some(Ok(0)));
        assert_eq!(iter32.next(), Some(Ok(1)));
        assert_eq!(iter32.next(), Some(Ok(-2)));
        assert_eq!(iter32.next(), Some(Ok(2)));
        assert_eq!(iter32.next(), None);
        assert_eq!(iter32.next(), None);

        let mut iter64 = RepeatedZigZagIter::<i64>::new(buf, 2);
        assert_eq!(iter64.next(), Some(Ok(-1)));
        assert_eq!(iter64.next(), Some(Ok(0)));
        assert_eq!(iter64.next(), Some(Ok(1)));
        assert_eq!(iter64.next(), Some(Ok(-2)));
        assert_eq!(iter64.next(), Some(Ok(2)));
        assert_eq!(iter64.next(), None);

        // Unexpected wire type Fixed64 on tag 2 (0x11)
        let bad_wt: &[u8] = &[0x11, 1, 2, 3, 4, 5, 6, 7, 8, 0x10, 0x02];
        let mut iter = RepeatedZigZagIter::<i32>::new(bad_wt, 2);
        assert_eq!(
            iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::Varint,
                actual: WireType::Fixed64,
            }))
        );
        assert_eq!(iter.next(), None);

        // Truncated varint inside packed zigzag blob
        let bad_packed: &[u8] = &[0x12, 0x02, 0x02, 0x80, 0x10, 0x04];
        let mut iter = RepeatedZigZagIter::<i64>::new(bad_packed, 2);
        assert_eq!(iter.next(), Some(Ok(1)));
        assert_eq!(iter.next(), Some(Err(DecodeError::UnexpectedEof)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn repeated_fixed32_iter_unpacked_packed_and_errors() {
        let v1: u32 = 0x11223344;
        let v2: u32 = 0xAABBCCDD;
        let v3: u32 = 0x00000001;
        let mut buf = [0u8; 24];
        // Unpacked tag 1 Fixed32 (0x0D) + v1 (0..5)
        buf[0] = 0x0D;
        buf[1..5].copy_from_slice(&v1.to_le_bytes());
        // Empty packed tag 1 LengthDelimited (0x0A, 0x00) (5..7)
        buf[5] = 0x0A;
        buf[6] = 0x00;
        // Packed tag 1 LengthDelimited (0x0A, 0x08) + v2 + v3 (7..17)
        buf[7] = 0x0A;
        buf[8] = 0x08;
        buf[9..13].copy_from_slice(&v2.to_le_bytes());
        buf[13..17].copy_from_slice(&v3.to_le_bytes());
        // Unrelated tag 2 Varint (0x10, 0x05) (17..19)
        buf[17] = 0x10;
        buf[18] = 0x05;
        // Unpacked tag 1 Fixed32 (0x0D) + (-42i32) (19..24)
        buf[19] = 0x0D;
        buf[20..24].copy_from_slice(&(-42i32).to_le_bytes());

        let mut iter_u32 = RepeatedFixed32Iter::<u32>::new(&buf, 1);
        assert_eq!(iter_u32.next(), Some(Ok(v1)));
        assert_eq!(iter_u32.next(), Some(Ok(v2)));
        assert_eq!(iter_u32.next(), Some(Ok(v3)));
        assert_eq!(iter_u32.next(), Some(Ok(-42i32 as u32)));
        assert_eq!(iter_u32.next(), None);
        assert_eq!(iter_u32.next(), None);

        let mut iter_i32 = RepeatedFixed32Iter::<i32>::new(&buf, 1);
        assert_eq!(iter_i32.next(), Some(Ok(v1 as i32)));
        assert_eq!(iter_i32.next(), Some(Ok(v2 as i32)));
        assert_eq!(iter_i32.next(), Some(Ok(1)));
        assert_eq!(iter_i32.next(), Some(Ok(-42)));
        assert_eq!(iter_i32.next(), None);

        // Float f32
        let f1: f32 = 3.5;
        let f2: f32 = -12.25;
        let mut fbuf = [0u8; 15];
        fbuf[0] = 0x0D;
        fbuf[1..5].copy_from_slice(&f1.to_le_bytes());
        fbuf[5] = 0x0A;
        fbuf[6] = 0x08;
        fbuf[7..11].copy_from_slice(&f2.to_le_bytes());
        fbuf[11..15].copy_from_slice(&f1.to_le_bytes());
        let mut iter_f32 = RepeatedFixed32Iter::<f32>::new(&fbuf, 1);
        assert_eq!(iter_f32.next(), Some(Ok(f1)));
        assert_eq!(iter_f32.next(), Some(Ok(f2)));
        assert_eq!(iter_f32.next(), Some(Ok(f1)));
        assert_eq!(iter_f32.next(), None);

        // Misaligned packed slice (6 bytes = one valid u32 + 2 trailing bytes), followed by valid unpacked Fixed32
        let misaligned: &[u8] = &[
            0x0A, 0x06, 1, 0, 0, 0, 99, 99, // packed 6 bytes
            0x0D, 2, 0, 0, 0, // unpacked 2
        ];
        let mut iter = RepeatedFixed32Iter::<u32>::new(misaligned, 1);
        assert_eq!(iter.next(), Some(Ok(1)));
        assert_eq!(iter.next(), Some(Err(DecodeError::UnexpectedEof)));
        assert_eq!(iter.next(), None);

        // Unexpected wire type (Varint on tag 1)
        let bad_wt: &[u8] = &[0x08, 0x01, 0x0D, 1, 0, 0, 0];
        let mut iter = RepeatedFixed32Iter::<u32>::new(bad_wt, 1);
        assert_eq!(
            iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::Fixed32,
                actual: WireType::Varint,
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn repeated_fixed64_iter_unpacked_packed_and_errors() {
        let v1: u64 = 0x0102030405060708;
        let v2: i64 = -123456789012345;
        let f1: f64 = 98765.432109875;
        let mut buf = [0u8; 38];
        // Unpacked tag 1 Fixed64 (0x09) + v1 (0..9)
        buf[0] = 0x09;
        buf[1..9].copy_from_slice(&v1.to_le_bytes());
        // Empty packed tag 1 LengthDelimited (0x0A, 0x00) (9..11)
        buf[9] = 0x0A;
        buf[10] = 0x00;
        // Packed tag 1 LengthDelimited (0x0A, 0x10) + v2 + f1 (11..29)
        buf[11] = 0x0A;
        buf[12] = 0x10;
        buf[13..21].copy_from_slice(&v2.to_le_bytes());
        buf[21..29].copy_from_slice(&f1.to_le_bytes());
        // Unpacked tag 1 Fixed64 (0x09) + f1 (29..38)
        buf[29] = 0x09;
        buf[30..38].copy_from_slice(&f1.to_le_bytes());

        let mut iter_u64 = RepeatedFixed64Iter::<u64>::new(&buf, 1);
        assert_eq!(iter_u64.next(), Some(Ok(v1)));
        assert_eq!(iter_u64.next(), Some(Ok(v2 as u64)));
        assert_eq!(iter_u64.next(), Some(Ok(f1.to_bits())));
        assert_eq!(iter_u64.next(), Some(Ok(f1.to_bits())));
        assert_eq!(iter_u64.next(), None);
        assert_eq!(iter_u64.next(), None);

        let mut iter_i64 = RepeatedFixed64Iter::<i64>::new(&buf, 1);
        assert_eq!(iter_i64.next(), Some(Ok(v1 as i64)));
        assert_eq!(iter_i64.next(), Some(Ok(v2)));
        assert_eq!(iter_i64.next(), Some(Ok(f1.to_bits() as i64)));
        assert_eq!(iter_i64.next(), Some(Ok(f1.to_bits() as i64)));
        assert_eq!(iter_i64.next(), None);

        let mut iter_f64 = RepeatedFixed64Iter::<f64>::new(&buf[29..], 1);
        assert_eq!(iter_f64.next(), Some(Ok(f1)));
        assert_eq!(iter_f64.next(), None);

        // Misaligned packed slice (10 bytes = one 8-byte u64 + 2 trailing bytes)
        let misaligned: &[u8] = &[
            0x0A, 0x0A, 1, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, // packed 10 bytes
            0x09, 2, 0, 0, 0, 0, 0, 0, 0, // unpacked 2
        ];
        let mut iter = RepeatedFixed64Iter::<u64>::new(misaligned, 1);
        assert_eq!(iter.next(), Some(Ok(1)));
        assert_eq!(iter.next(), Some(Err(DecodeError::UnexpectedEof)));
        assert_eq!(iter.next(), None);

        // Unexpected wire type (Fixed32 on tag 1 = 0x0D)
        let bad_wt: &[u8] = &[0x0D, 1, 2, 3, 4];
        let mut iter = RepeatedFixed64Iter::<u64>::new(bad_wt, 1);
        assert_eq!(
            iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::Fixed64,
                actual: WireType::Fixed32,
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn repeated_bytes_and_str_iter() {
        // Tag 3 (0x1A): "hello", empty "", "roto"
        // Interleaved with tag 1 Varint (0x08, 0x01)
        let buf: &[u8] = &[
            0x1A, 0x05, b'h', b'e', b'l', b'l', b'o', // "hello"
            0x08, 0x01, // tag 1 = 1
            0x1A, 0x00, // ""
            0x1A, 0x04, b'r', b'o', b't', b'o', // "roto"
        ];

        let mut bytes_iter = RepeatedBytesIter::new(buf, 3);
        assert_eq!(bytes_iter.next(), Some(Ok(&b"hello"[..])));
        assert_eq!(bytes_iter.next(), Some(Ok(&b""[..])));
        assert_eq!(bytes_iter.next(), Some(Ok(&b"roto"[..])));
        assert_eq!(bytes_iter.next(), None);
        assert_eq!(bytes_iter.next(), None);

        let mut str_iter = RepeatedStrIter::new(buf, 3);
        assert_eq!(str_iter.next(), Some(Ok("hello")));
        assert_eq!(str_iter.next(), Some(Ok("")));
        assert_eq!(str_iter.next(), Some(Ok("roto")));
        assert_eq!(str_iter.next(), None);
        assert_eq!(str_iter.next(), None);

        // Invalid UTF-8 in second element must return Err(InvalidUtf8) and fuse even when third element is valid
        let bad_utf8: &[u8] = &[
            0x1A, 0x02, b'o', b'k', // "ok"
            0x1A, 0x02, 0xFF, 0xFE, // invalid UTF-8
            0x1A, 0x05, b'a', b'f', b't', b'e', b'r', // "after"
        ];
        let mut str_iter = RepeatedStrIter::new(bad_utf8, 3);
        assert_eq!(str_iter.next(), Some(Ok("ok")));
        assert_eq!(str_iter.next(), Some(Err(DecodeError::InvalidUtf8)));
        assert_eq!(str_iter.next(), None);
        assert_eq!(str_iter.next(), None);

        // Unexpected wire type on tag 3 (Varint 0x18)
        let bad_wt: &[u8] = &[0x18, 0x01, 0x1A, 0x02, b'o', b'k'];
        let mut bytes_iter = RepeatedBytesIter::new(bad_wt, 3);
        assert_eq!(
            bytes_iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::LengthDelimited,
                actual: WireType::Varint,
            }))
        );
        assert_eq!(bytes_iter.next(), None);

        let mut str_iter = RepeatedStrIter::new(bad_wt, 3);
        assert_eq!(
            str_iter.next(),
            Some(Err(DecodeError::UnexpectedWireType {
                expected: WireType::LengthDelimited,
                actual: WireType::Varint,
            }))
        );
        assert_eq!(str_iter.next(), None);
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct DummySubMessageView<'a> {
        raw: &'a [u8],
        id: u32,
    }

    impl<'a> MessageView<'a> for DummySubMessageView<'a> {
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut id = None;
            for res in FieldIter::new(buf) {
                let field = res?;
                if field.tag == 1 {
                    if field.wire_type != WireType::Varint {
                        return Err(DecodeError::UnexpectedWireType {
                            expected: WireType::Varint,
                            actual: field.wire_type,
                        });
                    }
                    id = Some(decode_varint_at(buf, field.marker)? as u32);
                }
            }
            let id = id.ok_or(DecodeError::MissingRequiredField("DummySubMessage.id"))?;
            Ok(Self { raw: buf, id })
        }
    }

    #[test]
    fn repeated_message_iter_and_fusing() {
        // Outer message with repeated DummySubMessageView on tag 4 (0x22):
        // - submessage 1: id = 10 ([0x08, 0x0A])
        // - submessage 2: id = 20 ([0x08, 0x14])
        let buf: &[u8] = &[
            0x22, 0x02, 0x08, 0x0A, // submsg 1
            0x08, 0x99, 0x01, // unrelated outer field 1
            0x22, 0x02, 0x08, 0x14, // submsg 2
        ];

        let mut iter = RepeatedMessageIter::<DummySubMessageView<'_>>::new(buf, 4);
        assert_eq!(
            iter.next(),
            Some(Ok(DummySubMessageView {
                raw: &[0x08, 0x0A],
                id: 10,
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Ok(DummySubMessageView {
                raw: &[0x08, 0x14],
                id: 20,
            }))
        );
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Submessage missing required field causes error and fuses iterator before 3rd valid submessage
        let buf_err: &[u8] = &[
            0x22, 0x02, 0x08, 0x0A, // valid (id=10)
            0x22, 0x00, // empty submsg -> MissingRequiredField
            0x22, 0x02, 0x08, 0x1E, // valid (id=30), must not be yielded after fuse
        ];
        let mut iter = RepeatedMessageIter::<DummySubMessageView<'_>>::new(buf_err, 4);
        assert_eq!(
            iter.next(),
            Some(Ok(DummySubMessageView {
                raw: &[0x08, 0x0A],
                id: 10,
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Err(DecodeError::MissingRequiredField(
                "DummySubMessage.id"
            )))
        );
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unknown_fields_iter_preserves_all_unrecognized_wire_types() {
        // Known tags: &[1, 3]
        // Buffer contains:
        // - tag 1 (known, Varint): [0x08, 0x2A]
        // - tag 2 (unknown, Varint): [0x10, 0xAC, 0x02]
        // - tag 3 (known, LengthDelimited): [0x1A, 0x02, b'h', b'i']
        // - tag 4 (unknown, Fixed32): [0x25, 1, 2, 3, 4]
        // - tag 5 (unknown, LengthDelimited): [0x2A, 0x03, b'x', b'y', b'z']
        // - tag 6 (unknown, Group): [0x33, 0x08, 0x01, 0x34]
        // - tag 7 (unknown, Fixed64): [0x39, 1, 2, 3, 4, 5, 6, 7, 8]
        let buf: &[u8] = &[
            0x08, 0x2A, // tag 1 (known)
            0x10, 0xAC, 0x02, // tag 2 (unknown varint)
            0x1A, 0x02, b'h', b'i', // tag 3 (known)
            0x25, 1, 2, 3, 4, // tag 4 (unknown fixed32)
            0x2A, 0x03, b'x', b'y', b'z', // tag 5 (unknown length-delimited)
            0x33, 0x08, 0x01, 0x34, // tag 6 (unknown group)
            0x39, 1, 2, 3, 4, 5, 6, 7, 8, // tag 7 (unknown fixed64)
        ];

        let mut unknown = UnknownFieldsIter::new(buf, &[1, 3]);
        assert_eq!(unknown.next(), Some(&[0x10, 0xAC, 0x02][..]));
        assert_eq!(unknown.next(), Some(&[0x25, 1, 2, 3, 4][..]));
        assert_eq!(unknown.next(), Some(&[0x2A, 0x03, b'x', b'y', b'z'][..]));
        assert_eq!(unknown.next(), Some(&[0x33, 0x08, 0x01, 0x34][..]));
        assert_eq!(unknown.next(), Some(&[0x39, 1, 2, 3, 4, 5, 6, 7, 8][..]));
        assert_eq!(unknown.next(), None);
        assert_eq!(unknown.next(), None);
    }

    #[test]
    fn zigzag_encoding_and_roundtrip() {
        let cases32 = [0i32, -1, 1, -2, 2, 123456, -123456, i32::MAX, i32::MIN];
        assert_eq!(encode_zigzag32(0), 0);
        assert_eq!(encode_zigzag32(-1), 1);
        assert_eq!(encode_zigzag32(1), 2);
        assert_eq!(encode_zigzag32(-2), 3);
        assert_eq!(encode_zigzag32(2), 4);
        assert_eq!(encode_zigzag32(i32::MAX), 0xFFFFFFFE);
        assert_eq!(encode_zigzag32(i32::MIN), 0xFFFFFFFF);
        for val in cases32 {
            assert_eq!(decode_zigzag32(encode_zigzag32(val)), val);
        }

        let cases64 = [
            0i64,
            -1,
            1,
            -2,
            2,
            1234567890123,
            -1234567890123,
            i64::MAX,
            i64::MIN,
        ];
        assert_eq!(encode_zigzag64(0), 0);
        assert_eq!(encode_zigzag64(-1), 1);
        assert_eq!(encode_zigzag64(1), 2);
        assert_eq!(encode_zigzag64(-2), 3);
        assert_eq!(encode_zigzag64(2), 4);
        assert_eq!(encode_zigzag64(i64::MAX), 0xFFFFFFFFFFFFFFFE);
        assert_eq!(encode_zigzag64(i64::MIN), 0xFFFFFFFFFFFFFFFF);
        for val in cases64 {
            assert_eq!(decode_zigzag64(encode_zigzag64(val)), val);
        }
    }

    #[test]
    fn backward_writer_cursor_and_basic_bytes() {
        let mut backing = [0u8; 8];
        let mut writer = BackwardWriter::new(&mut backing);
        assert_eq!(writer.cursor(), 8);
        assert_eq!(writer.written_len(), 0);
        assert_eq!(writer.remaining_capacity(), 8);
        assert_eq!(writer.as_written_slice(), &[]);

        // Empty write
        assert_eq!(writer.write_bytes(&[]), Ok(()));
        assert_eq!(writer.cursor(), 8);
        assert_eq!(writer.written_len(), 0);

        // Write "to" then "ro"
        assert_eq!(writer.write_bytes(b"to"), Ok(()));
        assert_eq!(writer.cursor(), 6);
        assert_eq!(writer.written_len(), 2);
        assert_eq!(writer.remaining_capacity(), 6);
        assert_eq!(writer.as_written_slice(), b"to");

        assert_eq!(writer.write_bytes(b"ro"), Ok(()));
        assert_eq!(writer.cursor(), 4);
        assert_eq!(writer.written_len(), 4);
        assert_eq!(writer.remaining_capacity(), 4);
        assert_eq!(writer.as_written_slice(), b"roto");

        let finished = writer.finish();
        assert_eq!(finished, b"roto");
    }

    #[test]
    fn backward_writer_varint_and_zigzag() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (300, &[0xAC, 0x02]),
            (16383, &[0xFF, 0x7F]),
            (16384, &[0x80, 0x80, 0x01]),
            (u32::MAX as u64, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
            (
                u64::MAX,
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
            ),
        ];

        for &(val, expected_bytes) in cases {
            let mut buf = [0u8; 16];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_varint(val), Ok(()));
            let slice = writer.finish();
            assert_eq!(slice, expected_bytes);
            assert_eq!(decode_varint(slice), Ok((val, expected_bytes.len())));
        }

        // Verify write_zigzag32 zero-extends u32 so -1i32 is 1 byte (0x01), not 10 bytes
        {
            let mut buf = [0u8; 16];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_zigzag32(-1), Ok(()));
            let slice = writer.finish();
            assert_eq!(slice, &[0x01]);
            assert_eq!(decode_sint32_at(slice, FieldMarker::new(0, 1)), Ok(-1));
        }
        {
            let mut buf = [0u8; 16];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_zigzag32(i32::MIN), Ok(()));
            let slice = writer.finish();
            assert_eq!(slice, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
            assert_eq!(
                decode_sint32_at(slice, FieldMarker::new(0, slice.len())),
                Ok(i32::MIN)
            );
        }
        {
            let mut buf = [0u8; 16];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_zigzag64(-1), Ok(()));
            let slice = writer.finish();
            assert_eq!(slice, &[0x01]);
            assert_eq!(decode_sint64_at(slice, FieldMarker::new(0, 1)), Ok(-1));
        }
        {
            let mut buf = [0u8; 16];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_zigzag64(i64::MIN), Ok(()));
            let slice = writer.finish();
            assert_eq!(slice.len(), 10);
            assert_eq!(
                decode_sint64_at(slice, FieldMarker::new(0, slice.len())),
                Ok(i64::MIN)
            );
        }
    }

    #[test]
    fn backward_writer_all_field_types_roundtrip_with_field_iter() {
        let mut backing = [0u8; 256];
        let mut writer = BackwardWriter::new(&mut backing);

        // Write fields in reverse order so they appear in tag order 1..=11 on the wire:
        // Field 11: length-delimited empty bytes
        writer.write_length_delimited(&[]).unwrap();
        writer.write_tag(11, WireType::LengthDelimited).unwrap();

        // Field 10: length-delimited string "roto-runtime"
        writer.write_length_delimited(b"roto-runtime").unwrap();
        writer.write_tag(10, WireType::LengthDelimited).unwrap();

        // Field 9: f64
        let f64_val: f64 = -98765.43210987;
        writer.write_f64(f64_val).unwrap();
        writer.write_tag(9, WireType::Fixed64).unwrap();

        // Field 8: f32
        let f32_val: f32 = 3.125;
        writer.write_f32(f32_val).unwrap();
        writer.write_tag(8, WireType::Fixed32).unwrap();

        // Field 7: sfixed64
        writer.write_sfixed64(-0x1122334455667788).unwrap();
        writer.write_tag(7, WireType::Fixed64).unwrap();

        // Field 6: sfixed32
        writer.write_sfixed32(-0x12345678).unwrap();
        writer.write_tag(6, WireType::Fixed32).unwrap();

        // Field 5: fixed64
        writer.write_fixed64(0xCAFEBABEDEADBEEF).unwrap();
        writer.write_tag(5, WireType::Fixed64).unwrap();

        // Field 4: fixed32
        writer.write_fixed32(0xDEADBEEF).unwrap();
        writer.write_tag(4, WireType::Fixed32).unwrap();

        // Field 3: zigzag64
        writer.write_zigzag64(-999_999_999_999).unwrap();
        writer.write_tag(3, WireType::Varint).unwrap();

        // Field 2: zigzag32
        writer.write_zigzag32(-42_000).unwrap();
        writer.write_tag(2, WireType::Varint).unwrap();

        // Field 1: bool true
        writer.write_bool(true).unwrap();
        writer.write_tag(1, WireType::Varint).unwrap();

        // Field 100 (multi-byte tag): bool false
        writer.write_bool(false).unwrap();
        writer.write_tag(100, WireType::Varint).unwrap();

        let wire = writer.finish();
        let mut iter = FieldIter::new(wire);

        let f100 = iter.next().unwrap().unwrap();
        assert_eq!((f100.tag, f100.wire_type), (100, WireType::Varint));
        assert_eq!(decode_bool_at(wire, f100.marker), Ok(false));

        let f1 = iter.next().unwrap().unwrap();
        assert_eq!((f1.tag, f1.wire_type), (1, WireType::Varint));
        assert_eq!(decode_bool_at(wire, f1.marker), Ok(true));

        let f2 = iter.next().unwrap().unwrap();
        assert_eq!((f2.tag, f2.wire_type), (2, WireType::Varint));
        assert_eq!(decode_sint32_at(wire, f2.marker), Ok(-42_000));

        let f3 = iter.next().unwrap().unwrap();
        assert_eq!((f3.tag, f3.wire_type), (3, WireType::Varint));
        assert_eq!(decode_sint64_at(wire, f3.marker), Ok(-999_999_999_999));

        let f4 = iter.next().unwrap().unwrap();
        assert_eq!((f4.tag, f4.wire_type), (4, WireType::Fixed32));
        assert_eq!(decode_fixed32_at(wire, f4.marker), Ok(0xDEADBEEF));

        let f5 = iter.next().unwrap().unwrap();
        assert_eq!((f5.tag, f5.wire_type), (5, WireType::Fixed64));
        assert_eq!(decode_fixed64_at(wire, f5.marker), Ok(0xCAFEBABEDEADBEEF));

        let f6 = iter.next().unwrap().unwrap();
        assert_eq!((f6.tag, f6.wire_type), (6, WireType::Fixed32));
        assert_eq!(decode_sfixed32_at(wire, f6.marker), Ok(-0x12345678));

        let f7 = iter.next().unwrap().unwrap();
        assert_eq!((f7.tag, f7.wire_type), (7, WireType::Fixed64));
        assert_eq!(decode_sfixed64_at(wire, f7.marker), Ok(-0x1122334455667788));

        let f8 = iter.next().unwrap().unwrap();
        assert_eq!((f8.tag, f8.wire_type), (8, WireType::Fixed32));
        assert_eq!(decode_f32_at(wire, f8.marker), Ok(f32_val));

        let f9 = iter.next().unwrap().unwrap();
        assert_eq!((f9.tag, f9.wire_type), (9, WireType::Fixed64));
        assert_eq!(decode_f64_at(wire, f9.marker), Ok(f64_val));

        let f10 = iter.next().unwrap().unwrap();
        assert_eq!((f10.tag, f10.wire_type), (10, WireType::LengthDelimited));
        assert_eq!(decode_str_at(wire, f10.marker), Ok("roto-runtime"));

        let f11 = iter.next().unwrap().unwrap();
        assert_eq!((f11.tag, f11.wire_type), (11, WireType::LengthDelimited));
        assert_eq!(decode_bytes_at(wire, f11.marker), Ok(&[][..]));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn backward_writer_buffer_overflow_edge_cases() {
        // 0-byte buffer
        {
            let mut empty = [0u8; 0];
            let mut writer = BackwardWriter::new(&mut empty);
            assert_eq!(writer.cursor(), 0);
            assert_eq!(writer.remaining_capacity(), 0);
            assert_eq!(writer.written_len(), 0);
            assert_eq!(writer.write_bytes(&[]), Ok(()));
            assert_eq!(writer.write_bytes(&[1]), Err(EncodeError::BufferOverflow));
            assert_eq!(writer.write_varint(0), Err(EncodeError::BufferOverflow));
            assert_eq!(
                writer.write_tag(1, WireType::Varint),
                Err(EncodeError::BufferOverflow)
            );
            assert_eq!(writer.write_bool(false), Err(EncodeError::BufferOverflow));
            assert_eq!(writer.write_fixed32(0), Err(EncodeError::BufferOverflow));
            assert_eq!(writer.write_fixed64(0), Err(EncodeError::BufferOverflow));
            assert_eq!(
                writer.write_length_delimited(&[]),
                Err(EncodeError::BufferOverflow)
            );
            assert_eq!(writer.finish(), &[]);
        }

        // Exact capacity boundary and overflow on next byte
        {
            let mut buf = [0u8; 4];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_fixed32(0x11223344), Ok(()));
            assert_eq!(writer.cursor(), 0);
            assert_eq!(writer.remaining_capacity(), 0);
            assert_eq!(writer.written_len(), 4);
            assert_eq!(writer.write_bytes(&[]), Ok(()));
            assert_eq!(
                writer.write_tag(1, WireType::Fixed32),
                Err(EncodeError::BufferOverflow)
            );
            assert_eq!(writer.finish(), &0x11223344u32.to_le_bytes());
        }

        // Multi-byte varint overflow when buffer is 1 byte too short
        {
            let mut buf = [0u8; 1];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_varint(128), Err(EncodeError::BufferOverflow));
            // Cursor remains unchanged after failed write_bytes
            assert_eq!(writer.cursor(), 1);
        }

        // write_length_delimited: payload fits, but length varint overflows
        {
            let mut buf = [0u8; 3];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(
                writer.write_length_delimited(b"abc"),
                Err(EncodeError::BufferOverflow)
            );
        }

        // write_length_delimited: payload + length varint fit, but tag overflows
        {
            let mut buf = [0u8; 4];
            let mut writer = BackwardWriter::new(&mut buf);
            assert_eq!(writer.write_length_delimited(b"abc"), Ok(()));
            assert_eq!(writer.cursor(), 0);
            assert_eq!(
                writer.write_tag(1, WireType::LengthDelimited),
                Err(EncodeError::BufferOverflow)
            );
        }
    }
}
