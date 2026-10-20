#![no_std]
#![forbid(unsafe_code)]

pub mod error;
pub mod iter;
pub mod wire;

pub use error::{DecodeError, EncodeError};
pub use iter::{Field, FieldIter};
pub use wire::{
    FieldMarker, WireType, decode_bool_at, decode_bytes_at, decode_f32_at, decode_f64_at,
    decode_fixed32_at, decode_fixed64_at, decode_sfixed32_at, decode_sfixed64_at,
    decode_sint32_at, decode_sint64_at, decode_str_at, decode_varint, decode_varint_at,
    decode_zigzag32, decode_zigzag64,
};

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
}
