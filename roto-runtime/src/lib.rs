#![no_std]
#![forbid(unsafe_code)]

pub mod error;
pub mod wire;

pub use error::{DecodeError, EncodeError};
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
}
