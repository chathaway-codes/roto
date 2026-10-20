use std::fmt::Write as _;

use google_protobuf_roto::google::protobuf::{
    field_descriptor_proto::Type as FieldType, FieldDescriptorProtoView,
};

use super::config::GeneratorConfig;
use super::symbols::{ResolvedType, SymbolTable};

/// Resolves the Rust default value expression for a singular (`optional`) Protobuf field.
///
/// Inspects `field.default_value()` (`[default = ...]`) and falls back to the canonical
/// `proto2`/`proto3` zero/empty/first-variant default when `default_value` is absent or empty.
pub fn resolve_field_default(
    field: &FieldDescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<String, String> {
    let field_name = field
        .name()
        .map_err(|e| format!("invalid FieldDescriptorProto.name: {e}"))?;
    let field_type = field
        .r#type()
        .map_err(|e| format!("invalid FieldDescriptorProto.type in '{field_name}': {e}"))?;
    let raw_default = field
        .default_value()
        .map_err(|e| format!("invalid FieldDescriptorProto.default_value in '{field_name}': {e}"))?;

    let resolved = if field_type == FieldType::TypeEnum {
        let res = symbols
            .resolve_field(current_scope, field, config)?
            .ok_or_else(|| {
                format!("missing type_name on enum field '{field_name}' in '{current_scope}'")
            })?;
        Some(res)
    } else {
        None
    };

    parse_default_value(field_type, raw_default, resolved.as_ref())
        .map_err(|e| format!("invalid default value for field '{field_name}' in '{current_scope}': {e}"))
}

/// Parses a raw Protobuf `default_value` string into a valid `#![no_std]` Rust expression
/// for the given `field_type`.
pub fn parse_default_value(
    field_type: FieldType,
    raw_default: &str,
    resolved_type: Option<&ResolvedType>,
) -> Result<String, String> {
    match field_type {
        FieldType::TypeInt32 | FieldType::TypeSint32 | FieldType::TypeSfixed32 => {
            if raw_default.is_empty() {
                return Ok("0i32".to_string());
            }
            let val = parse_i32(raw_default)?;
            Ok(format!("{val}i32"))
        }
        FieldType::TypeInt64 | FieldType::TypeSint64 | FieldType::TypeSfixed64 => {
            if raw_default.is_empty() {
                return Ok("0i64".to_string());
            }
            let val = parse_i64(raw_default)?;
            Ok(format!("{val}i64"))
        }
        FieldType::TypeUint32 | FieldType::TypeFixed32 => {
            if raw_default.is_empty() {
                return Ok("0u32".to_string());
            }
            let val = parse_u32(raw_default)?;
            Ok(format!("{val}u32"))
        }
        FieldType::TypeUint64 | FieldType::TypeFixed64 => {
            if raw_default.is_empty() {
                return Ok("0u64".to_string());
            }
            let val = parse_u64(raw_default)?;
            Ok(format!("{val}u64"))
        }
        FieldType::TypeFloat => parse_f32_default(raw_default),
        FieldType::TypeDouble => parse_f64_default(raw_default),
        FieldType::TypeBool => match raw_default.trim() {
            "" | "false" | "0" => Ok("false".to_string()),
            "true" | "1" => Ok("true".to_string()),
            other => Err(format!("invalid bool default '{other}'")),
        },
        FieldType::TypeString => Ok(format_string_literal(raw_default)),
        FieldType::TypeBytes => {
            let bytes = unescape_c_bytes(raw_default)?;
            Ok(format_bytes_literal(&bytes))
        }
        FieldType::TypeEnum => {
            let resolved = resolved_type
                .ok_or_else(|| "TypeEnum default resolution requires a ResolvedType".to_string())?;
            resolved.resolve_enum_default(raw_default.trim())
        }
        FieldType::TypeMessage | FieldType::TypeGroup => Err(format!(
            "composite field type {field_type:?} does not have a scalar default expression"
        )),
    }
}

fn parse_i32(s: &str) -> Result<i32, String> {
    let trimmed = s.trim();
    if let Some(hex) = trimmed.strip_prefix("-0x").or_else(|| trimmed.strip_prefix("-0X")) {
        let mag = u32::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex i32 default '{s}': {e}"))?;
        if mag == 0x8000_0000 {
            return Ok(i32::MIN);
        }
        let signed = i32::try_from(mag)
            .map_err(|_| format!("hex i32 default '{s}' out of range"))?;
        return Ok(-signed);
    }
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix("+0x"))
        .or_else(|| trimmed.strip_prefix("+0X"))
    {
        return i32::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex i32 default '{s}': {e}"));
    }
    trimmed
        .parse::<i32>()
        .map_err(|e| format!("invalid i32 default '{s}': {e}"))
}

fn parse_i64(s: &str) -> Result<i64, String> {
    let trimmed = s.trim();
    if let Some(hex) = trimmed.strip_prefix("-0x").or_else(|| trimmed.strip_prefix("-0X")) {
        let mag = u64::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex i64 default '{s}': {e}"))?;
        if mag == 0x8000_0000_0000_0000 {
            return Ok(i64::MIN);
        }
        let signed = i64::try_from(mag)
            .map_err(|_| format!("hex i64 default '{s}' out of range"))?;
        return Ok(-signed);
    }
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix("+0x"))
        .or_else(|| trimmed.strip_prefix("+0X"))
    {
        return i64::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex i64 default '{s}': {e}"));
    }
    trimmed
        .parse::<i64>()
        .map_err(|e| format!("invalid i64 default '{s}': {e}"))
}

fn parse_u32(s: &str) -> Result<u32, String> {
    let trimmed = s.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix("+0x"))
        .or_else(|| trimmed.strip_prefix("+0X"))
    {
        return u32::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex u32 default '{s}': {e}"));
    }
    trimmed
        .parse::<u32>()
        .map_err(|e| format!("invalid u32 default '{s}': {e}"))
}

fn parse_u64(s: &str) -> Result<u64, String> {
    let trimmed = s.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix("+0x"))
        .or_else(|| trimmed.strip_prefix("+0X"))
    {
        return u64::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex u64 default '{s}': {e}"));
    }
    trimmed
        .parse::<u64>()
        .map_err(|e| format!("invalid u64 default '{s}': {e}"))
}

fn parse_f32_default(raw_default: &str) -> Result<String, String> {
    let trimmed = raw_default.trim();
    if trimmed.is_empty() {
        return Ok("0.0f32".to_string());
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => Ok("f32::INFINITY".to_string()),
        "-inf" | "-infinity" => Ok("f32::NEG_INFINITY".to_string()),
        "nan" | "+nan" | "-nan" => Ok("f32::NAN".to_string()),
        _ => {
            let val = trimmed
                .parse::<f32>()
                .map_err(|e| format!("invalid f32 default '{raw_default}': {e}"))?;
            if val.is_nan() {
                Ok("f32::NAN".to_string())
            } else if val.is_infinite() {
                if val.is_sign_positive() {
                    Ok("f32::INFINITY".to_string())
                } else {
                    Ok("f32::NEG_INFINITY".to_string())
                }
            } else {
                let mut lit = format!("{val:?}");
                if !lit.contains('.') && !lit.contains('e') && !lit.contains('E') {
                    lit.push_str(".0");
                }
                Ok(format!("{lit}f32"))
            }
        }
    }
}

fn parse_f64_default(raw_default: &str) -> Result<String, String> {
    let trimmed = raw_default.trim();
    if trimmed.is_empty() {
        return Ok("0.0f64".to_string());
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => Ok("f64::INFINITY".to_string()),
        "-inf" | "-infinity" => Ok("f64::NEG_INFINITY".to_string()),
        "nan" | "+nan" | "-nan" => Ok("f64::NAN".to_string()),
        _ => {
            let val = trimmed
                .parse::<f64>()
                .map_err(|e| format!("invalid f64 default '{raw_default}': {e}"))?;
            if val.is_nan() {
                Ok("f64::NAN".to_string())
            } else if val.is_infinite() {
                if val.is_sign_positive() {
                    Ok("f64::INFINITY".to_string())
                } else {
                    Ok("f64::NEG_INFINITY".to_string())
                }
            } else {
                let mut lit = format!("{val:?}");
                if !lit.contains('.') && !lit.contains('e') && !lit.contains('E') {
                    lit.push_str(".0");
                }
                Ok(format!("{lit}f64"))
            }
        }
    }
}

/// Formats a UTF-8 string as an escaped Rust string literal `"..."`.
pub fn format_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                write!(out, "\\u{{{:04x}}}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Unescapes a `protoc` C-escaped `bytes` default value string into raw bytes.
///
/// Supports:
/// - `\ooo` octal escapes (1 to 3 octal digits `'0'..='7'`)
/// - `\xHH` / `\XHH` hex escapes (1 to 2 hex digits)
/// - Standard C escapes: `\a` (`0x07`), `\b` (`0x08`), `\f` (`0x0C`), `\n` (`0x0A`),
///   `\r` (`0x0D`), `\t` (`0x09`), `\v` (`0x0B`), `\\`, `\'`, `\"`, `\?` (`0x3F`)
/// - Literal bytes
pub fn unescape_c_bytes(escaped: &str) -> Result<Vec<u8>, String> {
    let bytes = escaped.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b != b'\\' {
            out.push(b);
            i += 1;
            continue;
        }

        i += 1;
        if i >= bytes.len() {
            return Err("trailing backslash in C-escaped bytes default".to_string());
        }

        let esc = bytes[i];
        match esc {
            b'a' => {
                out.push(0x07);
                i += 1;
            }
            b'b' => {
                out.push(0x08);
                i += 1;
            }
            b'f' => {
                out.push(0x0C);
                i += 1;
            }
            b'n' => {
                out.push(b'\n');
                i += 1;
            }
            b'r' => {
                out.push(b'\r');
                i += 1;
            }
            b't' => {
                out.push(b'\t');
                i += 1;
            }
            b'v' => {
                out.push(0x0B);
                i += 1;
            }
            b'\\' => {
                out.push(b'\\');
                i += 1;
            }
            b'\'' => {
                out.push(b'\'');
                i += 1;
            }
            b'"' => {
                out.push(b'"');
                i += 1;
            }
            b'?' => {
                out.push(0x3F);
                i += 1;
            }
            b'0'..=b'7' => {
                let mut val: u32 = (esc - b'0') as u32;
                i += 1;
                for _ in 0..2 {
                    if i < bytes.len() && matches!(bytes[i], b'0'..=b'7') {
                        val = (val * 8) + (bytes[i] - b'0') as u32;
                        i += 1;
                    } else {
                        break;
                    }
                }
                if val > 0xFF {
                    return Err(format!("octal escape \\{val:o} out of u8 range"));
                }
                out.push(val as u8);
            }
            b'x' | b'X' => {
                i += 1;
                if i >= bytes.len() || !bytes[i].is_ascii_hexdigit() {
                    return Err("missing hex digits after \\x in C-escaped bytes default".to_string());
                }
                let mut val: u8 = hex_digit_val(bytes[i]);
                i += 1;
                if i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                    val = (val << 4) | hex_digit_val(bytes[i]);
                    i += 1;
                }
                out.push(val);
            }
            other => {
                return Err(format!(
                    "unknown escape sequence '\\{}' (0x{other:02x}) in C-escaped bytes default",
                    other as char
                ));
            }
        }
    }

    Ok(out)
}

fn hex_digit_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => unreachable!(),
    }
}

/// Formats a byte slice as a `#![no_std]` Rust byte slice literal `&b"..."[..]`.
pub fn format_bytes_literal(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() + 6);
    out.push_str("&b\"");
    for &b in bytes {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7E => out.push(b as char),
            other => {
                write!(out, "\\x{other:02x}").unwrap();
            }
        }
    }
    out.push_str("\"[..]");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use google_protobuf_roto::google::protobuf::{
        field_descriptor_proto::Label as FieldLabel, FieldDescriptorProtoBuilder,
        FileDescriptorProtoBuilder, FileDescriptorProtoView,
    };

    #[test]
    fn parses_all_integer_defaults_and_empty_fallbacks() {
        for ty in [
            FieldType::TypeInt32,
            FieldType::TypeSint32,
            FieldType::TypeSfixed32,
        ] {
            assert_eq!(parse_default_value(ty, "", None).unwrap(), "0i32");
            assert_eq!(parse_default_value(ty, "42", None).unwrap(), "42i32");
            assert_eq!(parse_default_value(ty, "-12345", None).unwrap(), "-12345i32");
            assert_eq!(
                parse_default_value(ty, "-2147483648", None).unwrap(),
                "-2147483648i32"
            );
            assert_eq!(
                parse_default_value(ty, "2147483647", None).unwrap(),
                "2147483647i32"
            );
            assert_eq!(parse_default_value(ty, "0x2a", None).unwrap(), "42i32");
            assert_eq!(parse_default_value(ty, "-0x2a", None).unwrap(), "-42i32");
            assert!(parse_default_value(ty, "99999999999", None).is_err());
        }

        for ty in [
            FieldType::TypeInt64,
            FieldType::TypeSint64,
            FieldType::TypeSfixed64,
        ] {
            assert_eq!(parse_default_value(ty, "", None).unwrap(), "0i64");
            assert_eq!(parse_default_value(ty, "-9999999999", None).unwrap(), "-9999999999i64");
            assert_eq!(
                parse_default_value(ty, "-9223372036854775808", None).unwrap(),
                "-9223372036854775808i64"
            );
            assert_eq!(
                parse_default_value(ty, "9223372036854775807", None).unwrap(),
                "9223372036854775807i64"
            );
            assert!(parse_default_value(ty, "not_an_int", None).is_err());
        }

        for ty in [FieldType::TypeUint32, FieldType::TypeFixed32] {
            assert_eq!(parse_default_value(ty, "", None).unwrap(), "0u32");
            assert_eq!(parse_default_value(ty, "4294967295", None).unwrap(), "4294967295u32");
            assert_eq!(parse_default_value(ty, "0xFF", None).unwrap(), "255u32");
            assert!(parse_default_value(ty, "-1", None).is_err());
        }

        for ty in [FieldType::TypeUint64, FieldType::TypeFixed64] {
            assert_eq!(parse_default_value(ty, "", None).unwrap(), "0u64");
            assert_eq!(
                parse_default_value(ty, "18446744073709551615", None).unwrap(),
                "18446744073709551615u64"
            );
            assert!(parse_default_value(ty, "-1", None).is_err());
        }
    }

    #[test]
    fn parses_float_and_double_defaults_including_inf_and_nan() {
        // f32
        assert_eq!(
            parse_default_value(FieldType::TypeFloat, "", None).unwrap(),
            "0.0f32"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeFloat, "0", None).unwrap(),
            "0.0f32"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeFloat, "5", None).unwrap(),
            "5.0f32"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeFloat, "-3.25", None).unwrap(),
            "-3.25f32"
        );
        for inf in ["inf", "+inf", "infinity", "+infinity", "INF", "Infinity"] {
            assert_eq!(
                parse_default_value(FieldType::TypeFloat, inf, None).unwrap(),
                "f32::INFINITY"
            );
        }
        for ninf in ["-inf", "-infinity", "-INF", "-Infinity"] {
            assert_eq!(
                parse_default_value(FieldType::TypeFloat, ninf, None).unwrap(),
                "f32::NEG_INFINITY"
            );
        }
        for nan in ["nan", "NaN", "+nan", "-nan"] {
            assert_eq!(
                parse_default_value(FieldType::TypeFloat, nan, None).unwrap(),
                "f32::NAN"
            );
        }
        assert!(parse_default_value(FieldType::TypeFloat, "abc", None).is_err());

        // f64
        assert_eq!(
            parse_default_value(FieldType::TypeDouble, "", None).unwrap(),
            "0.0f64"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeDouble, "42", None).unwrap(),
            "42.0f64"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeDouble, "2.5", None).unwrap(),
            "2.5f64"
        );
        for inf in ["inf", "+inf", "infinity", "+infinity"] {
            assert_eq!(
                parse_default_value(FieldType::TypeDouble, inf, None).unwrap(),
                "f64::INFINITY"
            );
        }
        for ninf in ["-inf", "-infinity"] {
            assert_eq!(
                parse_default_value(FieldType::TypeDouble, ninf, None).unwrap(),
                "f64::NEG_INFINITY"
            );
        }
        for nan in ["nan", "NaN"] {
            assert_eq!(
                parse_default_value(FieldType::TypeDouble, nan, None).unwrap(),
                "f64::NAN"
            );
        }
        assert!(parse_default_value(FieldType::TypeDouble, "abc", None).is_err());
    }

    #[test]
    fn parses_bool_string_and_c_escaped_bytes_defaults() {
        // bool
        assert_eq!(
            parse_default_value(FieldType::TypeBool, "", None).unwrap(),
            "false"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeBool, "false", None).unwrap(),
            "false"
        );
        assert_eq!(
            parse_default_value(FieldType::TypeBool, "true", None).unwrap(),
            "true"
        );
        assert!(parse_default_value(FieldType::TypeBool, "yes", None).is_err());

        // string
        assert_eq!(
            parse_default_value(FieldType::TypeString, "", None).unwrap(),
            "\"\""
        );
        assert_eq!(
            parse_default_value(FieldType::TypeString, "unknown", None).unwrap(),
            "\"unknown\""
        );
        assert_eq!(
            parse_default_value(FieldType::TypeString, "a\"b\\c\nd\r\t\0", None).unwrap(),
            "\"a\\\"b\\\\c\\nd\\r\\t\\0\""
        );

        // bytes
        assert_eq!(
            parse_default_value(FieldType::TypeBytes, "", None).unwrap(),
            "&b\"\"[..]"
        );
        let raw_c_escaped = r#"\0\001\377\x00\xfe\xFF\a\b\f\n\r\t\v\\\'\"\?Az"#;
        let unescaped = unescape_c_bytes(raw_c_escaped).unwrap();
        assert_eq!(
            unescaped,
            vec![
                0x00, 0x01, 0xFF, 0x00, 0xFE, 0xFF, 0x07, 0x08, 0x0C, b'\n', b'\r', b'\t', 0x0B,
                b'\\', b'\'', b'"', b'?', b'A', b'z',
            ]
        );
        assert_eq!(
            parse_default_value(FieldType::TypeBytes, raw_c_escaped, None).unwrap(),
            r#"&b"\x00\x01\xff\x00\xfe\xff\x07\x08\x0c\n\r\t\x0b\\'\"?Az"[..]"#
        );

        // Invalid C-escapes
        assert!(unescape_c_bytes("\\").is_err());
        assert!(unescape_c_bytes("\\x").is_err());
        assert!(unescape_c_bytes("\\xZZ").is_err());
        assert!(unescape_c_bytes("\\777").is_err());
        assert!(unescape_c_bytes("\\q").is_err());
    }

    #[test]
    fn resolves_enum_and_descriptor_field_defaults() {
        let mut buf = [0u8; 512];
        let mut fb = FileDescriptorProtoBuilder::new(&mut buf);
        fb.write_enum_type(|en| {
            en.write_value(|v| {
                v.write_number(3).write_name("LITE_RUNTIME");
            })
            .write_value(|v| {
                v.write_number(2).write_name("CODE_SIZE");
            })
            .write_value(|v| {
                v.write_number(1).write_name("SPEED");
            })
            .write_name("OptimizeMode");
        })
        .write_package("demo")
        .write_name("demo.proto");
        let file_view = FileDescriptorProtoView::from_bytes(fb.finish().unwrap()).unwrap();
        let symbols = SymbolTable::from_files([file_view]).unwrap();
        let config = GeneratorConfig::default();

        // 1. Enum field with explicit [default = CODE_SIZE]
        let mut fbuf1 = [0u8; 128];
        let mut fb1 = FieldDescriptorProtoBuilder::new(&mut fbuf1);
        fb1.write_default_value("CODE_SIZE")
            .write_type_name(".demo.OptimizeMode")
            .write_type(FieldType::TypeEnum)
            .write_label(FieldLabel::LabelOptional)
            .write_number(1)
            .write_name("optimize_for");
        let field1 = FieldDescriptorProtoView::from_bytes(fb1.finish().unwrap()).unwrap();
        assert_eq!(
            resolve_field_default(&field1, ".demo", &symbols, &config).unwrap(),
            "crate::demo::OptimizeMode::CodeSize"
        );

        // 2. Enum field without explicit default -> falls back to first variant (Speed)
        let mut fbuf2 = [0u8; 128];
        let mut fb2 = FieldDescriptorProtoBuilder::new(&mut fbuf2);
        fb2.write_type_name(".demo.OptimizeMode")
            .write_type(FieldType::TypeEnum)
            .write_label(FieldLabel::LabelOptional)
            .write_number(1)
            .write_name("optimize_for");
        let field2 = FieldDescriptorProtoView::from_bytes(fb2.finish().unwrap()).unwrap();
        assert_eq!(
            resolve_field_default(&field2, ".demo", &symbols, &config).unwrap(),
            "crate::demo::OptimizeMode::Speed"
        );
    }
}
