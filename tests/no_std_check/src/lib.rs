#![no_std]
#![forbid(unsafe_code)]

use roto_runtime::{
    DecodeError, EncodeError, FieldMarker, WireType, decode_bool_at, decode_bytes_at,
    decode_f32_at, decode_f64_at, decode_fixed32_at, decode_fixed64_at, decode_sfixed32_at,
    decode_sfixed64_at, decode_sint32_at, decode_sint64_at, decode_str_at, decode_varint,
    decode_varint_at, decode_zigzag32, decode_zigzag64,
};

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Exercises `roto` and `roto-runtime` in a `#![no_std]`, zero-allocation environment.
pub fn verify_no_std() -> bool {
    if !(roto::runtime::runtime_ready() && roto_runtime::runtime_ready()) {
        return false;
    }

    let Ok(WireType::Varint) = WireType::from_u8(0) else {
        return false;
    };
    let Err(DecodeError::InvalidWireType(7)) = WireType::from_u8(7) else {
        return false;
    };
    let _enc_err = EncodeError::BufferOverflow;

    let none = FieldMarker::none();
    if none.is_present() || !none.is_empty() || !none.slice(&[1, 2]).is_empty() {
        return false;
    }

    let buf: &[u8] = &[
        0xAC, 0x02, // 0..2: varint 300
        0x01, // 2..3: bool true / zigzag -1
        0x78, 0x56, 0x34, 0x12, // 3..7: fixed32 0x12345678
        0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, // 7..15: fixed64
        b'r', b'o', b't', b'o', // 15..19: "roto"
    ];

    let Ok((300, 2)) = decode_varint(buf) else {
        return false;
    };
    let Ok(300) = decode_varint_at(buf, FieldMarker::new(0, 2)) else {
        return false;
    };
    let Ok(true) = decode_bool_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    if decode_zigzag32(1) != -1 || decode_zigzag64(2) != 1 {
        return false;
    }
    let Ok(-1) = decode_sint32_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    let Ok(-1) = decode_sint64_at(buf, FieldMarker::new(2, 3)) else {
        return false;
    };
    let Ok(0x12345678) = decode_fixed32_at(buf, FieldMarker::new(3, 7)) else {
        return false;
    };
    let Ok(0x12345678) = decode_sfixed32_at(buf, FieldMarker::new(3, 7)) else {
        return false;
    };
    let Ok(0x0123456789ABCDEF) = decode_fixed64_at(buf, FieldMarker::new(7, 15)) else {
        return false;
    };
    let Ok(0x0123456789ABCDEF) = decode_sfixed64_at(buf, FieldMarker::new(7, 15)) else {
        return false;
    };
    if decode_f32_at(buf, FieldMarker::new(3, 7)).is_err() {
        return false;
    }
    if decode_f64_at(buf, FieldMarker::new(7, 15)).is_err() {
        return false;
    }
    let Ok(b"roto") = decode_bytes_at(buf, FieldMarker::new(15, 19)) else {
        return false;
    };
    let Ok("roto") = decode_str_at(buf, FieldMarker::new(15, 19)) else {
        return false;
    };

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_std_harness_smoke_test() {
        assert!(verify_no_std());
    }
}
