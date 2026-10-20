/// Converts a Protobuf identifier (`SCREAMING_SNAKE_CASE`, `snake_case`, or `PascalCase`)
/// into a Rust `PascalCase` identifier.
///
/// Existing `PascalCase` or mixed-case identifiers without underscores (such as
/// `FileDescriptorProto`, `JSType`, or `CType`) preserve their internal capitalization
/// while ensuring the leading character is uppercase.
/// Leading digits are prefixed with `_` to form a valid Rust identifier.
pub fn to_pascal_case(s: &str) -> String {
    if s.is_empty() {
        return "_".to_string();
    }

    let out = if !s.contains('_') && s.chars().any(|c| c.is_ascii_lowercase()) {
        let mut chars = s.chars();
        let mut result = String::with_capacity(s.len());
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.extend(chars);
        }
        result
    } else {
        let mut result = String::with_capacity(s.len());
        for part in s.split('_').filter(|p| !p.is_empty()) {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                for ch in chars {
                    result.push(ch.to_ascii_lowercase());
                }
            }
        }
        if result.is_empty() {
            "_".to_string()
        } else {
            result
        }
    };

    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{out}")
    } else {
        out
    }
}

/// Converts a Protobuf type name into a valid, escaped Rust type identifier,
/// preserving acronyms such as `JSType` and `CType`.
pub fn sanitize_type_ident(s: &str) -> String {
    escape_ident(&to_pascal_case(s))
}

/// Converts a `PascalCase`, `camelCase`, or `SCREAMING_SNAKE_CASE` identifier
/// into `snake_case` matching `roto` module and field naming conventions
/// (e.g. `FileDescriptorProto` -> `file_descriptor_proto`, `JSType` -> `js_type`,
/// `CType` -> `c_type`).
pub fn to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return "_".to_string();
    }

    let mut result = String::with_capacity(s.len() + 4);

    for segment in s.split(['_', '-', ' ']).filter(|seg| !seg.is_empty()) {
        if !result.is_empty() {
            result.push('_');
        }
        let chars: Vec<char> = segment.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && ch.is_ascii_uppercase() {
                let prev = chars[i - 1];
                let next_is_lower = chars.get(i + 1).is_some_and(|c| c.is_ascii_lowercase());
                if prev.is_ascii_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_ascii_uppercase() && next_is_lower)
                {
                    result.push('_');
                }
            }
            result.push(ch.to_ascii_lowercase());
        }
    }

    if result.is_empty() {
        return "_".to_string();
    }

    if result.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{result}")
    } else {
        result
    }
}

/// Converts a `PascalCase`, `camelCase`, or `snake_case` identifier into
/// `SCREAMING_SNAKE_CASE` for constant names (e.g. `SayHello` -> `SAY_HELLO`).
pub fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_ascii_uppercase()
}

/// Escapes a Rust identifier if it collides with a strict, reserved, or weak Rust keyword.
///
/// Keywords that cannot be raw-escaped (`self`, `super`, `crate`, `Self`) are suffixed
/// with a trailing underscore (`self_`, `super_`, `crate_`, `Self_`). All other Rust
/// keywords are prefixed with `r#`.
pub fn escape_ident(s: &str) -> String {
    if s.is_empty() {
        return "_".to_string();
    }
    if s.starts_with("r#") {
        return s.to_string();
    }
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return format!("_{s}");
    }

    match s {
        "self" | "super" | "crate" | "Self" => format!("{s}_"),
        "as" | "break" | "const" | "continue" | "else" | "enum" | "extern" | "false" | "fn"
        | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true" | "type" | "unsafe"
        | "use" | "where" | "while" | "async" | "await" | "dyn" | "abstract" | "become" | "box"
        | "do" | "final" | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual"
        | "yield" | "try" | "gen" => format!("r#{s}"),
        _ => s.to_string(),
    }
}

/// Converts a package segment or enclosing message name into a `snake_case` Rust module
/// identifier with keyword escaping applied (e.g. `type` -> `r#type`,
/// `FieldDescriptorProto` -> `field_descriptor_proto`).
pub fn escape_mod_ident(s: &str) -> String {
    let stripped = s.strip_prefix("r#").unwrap_or(s);
    escape_ident(&to_snake_case(stripped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_case_converts_variants_and_oneofs_while_preserving_type_acronyms() {
        assert_eq!(to_pascal_case("EDITION_UNKNOWN"), "EditionUnknown");
        assert_eq!(to_pascal_case("EDITION_1_TEST_ONLY"), "Edition1TestOnly");
        assert_eq!(
            to_pascal_case("EDITION_99997_TEST_ONLY"),
            "Edition99997TestOnly"
        );
        assert_eq!(to_pascal_case("TYPE_SFIXED32"), "TypeSfixed32");
        assert_eq!(to_pascal_case("JS_NORMAL"), "JsNormal");
        assert_eq!(to_pascal_case("SPEED"), "Speed");
        assert_eq!(to_pascal_case("CODE_SIZE"), "CodeSize");
        assert_eq!(to_pascal_case("LITE_RUNTIME"), "LiteRuntime");
        assert_eq!(to_pascal_case("STRING_PIECE"), "StringPiece");
        assert_eq!(
            to_pascal_case("FEATURE_PROTO3_OPTIONAL"),
            "FeatureProto3Optional"
        );
        assert_eq!(to_pascal_case("my_oneof"), "MyOneof");
        assert_eq!(to_pascal_case("payload"), "Payload");

        // Preserve existing PascalCase and acronym capitalization when no underscores
        assert_eq!(to_pascal_case("FileDescriptorProto"), "FileDescriptorProto");
        assert_eq!(to_pascal_case("JSType"), "JSType");
        assert_eq!(to_pascal_case("CType"), "CType");
        assert_eq!(sanitize_type_ident("JSType"), "JSType");
        assert_eq!(sanitize_type_ident("CType"), "CType");
        assert_eq!(sanitize_type_ident("Self"), "Self_");

        // Leading digits and empty inputs
        assert_eq!(to_pascal_case("1_test"), "_1Test");
        assert_eq!(to_pascal_case("2023Edition"), "_2023Edition");
        assert_eq!(to_pascal_case(""), "_");
    }

    #[test]
    fn snake_case_handles_pascal_camel_acronyms_and_screaming_snake() {
        assert_eq!(
            to_snake_case("FileDescriptorProto"),
            "file_descriptor_proto"
        );
        assert_eq!(to_snake_case("EnumReservedRange"), "enum_reserved_range");
        assert_eq!(
            to_snake_case("CodeGeneratorResponse"),
            "code_generator_response"
        );
        assert_eq!(to_snake_case("FieldOptions"), "field_options");
        assert_eq!(to_snake_case("JSType"), "js_type");
        assert_eq!(to_snake_case("CType"), "c_type");
        assert_eq!(
            to_snake_case("SCREAMING_SNAKE_CASE"),
            "screaming_snake_case"
        );
        assert_eq!(to_snake_case("UTF8Validation"), "utf8_validation");
        assert_eq!(to_snake_case("proto3_optional"), "proto3_optional");
        assert_eq!(to_snake_case("lat_e7"), "lat_e7");
        assert_eq!(to_snake_case("123Field"), "_123_field");
        assert_eq!(to_snake_case(""), "_");
    }

    #[test]
    fn keyword_escaping_covers_strict_reserved_weak_and_path_keywords() {
        for kw in [
            "as", "break", "const", "continue", "else", "enum", "extern", "false", "fn", "for",
            "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "static", "struct", "trait", "true", "type", "unsafe", "use", "where",
            "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
            "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
        ] {
            assert_eq!(escape_ident(kw), format!("r#{kw}"));
            assert_eq!(escape_mod_ident(kw), format!("r#{kw}"));
        }

        assert_eq!(escape_ident("self"), "self_");
        assert_eq!(escape_ident("super"), "super_");
        assert_eq!(escape_ident("crate"), "crate_");
        assert_eq!(escape_ident("Self"), "Self_");

        assert_eq!(escape_mod_ident("self"), "self_");
        assert_eq!(escape_mod_ident("super"), "super_");
        assert_eq!(escape_mod_ident("crate"), "crate_");

        // Idempotent when already escaped
        assert_eq!(escape_ident("r#type"), "r#type");
        assert_eq!(escape_mod_ident("r#type"), "r#type");

        // Normal identifiers remain untouched
        assert_eq!(escape_ident("google"), "google");
        assert_eq!(escape_mod_ident("DescriptorProto"), "descriptor_proto");
    }
}
