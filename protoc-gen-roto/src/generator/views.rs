use std::collections::BTreeSet;
use std::fmt::Write as _;

use google_protobuf_roto::google::protobuf::{
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    DescriptorProtoView, FieldDescriptorProtoView,
};

use super::config::GeneratorConfig;
use super::defaults::resolve_field_default;
use super::enums::{collect_message_oneofs, generate_enum, generate_oneof, OneofInfo};
use super::ident::{escape_ident, escape_mod_ident, sanitize_type_ident, to_snake_case};
use super::symbols::{ResolvedType, SymbolTable};

struct ResolvedField<'a> {
    proto_field: FieldDescriptorProtoView<'a>,
    raw_name: &'a str,
    accessor_ident: String,
    has_ident: String,
    marker_ident: String,
    tag: u32,
    label: FieldLabel,
    field_type: FieldType,
    resolved_type: Option<ResolvedType>,
    real_oneof_index: Option<i32>,
}

/// Generates the `#![no_std]` zero-copy `<Msg>View<'a>` struct, its `impl<'a> <Msg>View<'a>`
/// methods, its `::roto::runtime::MessageView<'a>` implementation, and (when the message
/// declares nested enums, real `oneof`s, or nested messages) the `pub mod <msg_snake> { ... }`
/// submodule containing all nested definitions.
pub fn generate_message_view(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    generate_view_only(msg_desc, current_scope, symbols, config, indent, out)?;

    let mut nested_out = String::new();
    generate_nested_module(
        msg_desc,
        current_scope,
        symbols,
        config,
        indent,
        &mut nested_out,
    )?;
    if !nested_out.is_empty() {
        writeln!(out).unwrap();
        out.push_str(&nested_out);
    }

    Ok(())
}

/// Alias for [`generate_message_view`] emitting `<Msg>View<'a>` and any nested submodule.
pub fn generate_message(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    generate_message_view(msg_desc, current_scope, symbols, config, indent, out)
}

/// Generates only the `<Msg>View<'a>` struct, `impl<'a> <Msg>View<'a>`, and
/// `impl<'a> ::roto::runtime::MessageView<'a> for <Msg>View<'a>` without the nested module.
pub fn generate_view_only(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let raw_msg_name = msg_desc
        .name()
        .map_err(|e| format!("invalid DescriptorProto.name: {e}"))?;
    if raw_msg_name.is_empty() {
        return Err("DescriptorProto.name cannot be empty".to_string());
    }

    let msg_scope = compute_message_scope(current_scope, raw_msg_name, symbols);
    let msg_ident = sanitize_type_ident(raw_msg_name);
    let view_ident = format!("{msg_ident}View");
    let msg_mod_ident = escape_mod_ident(raw_msg_name);

    let real_oneofs = collect_message_oneofs(msg_desc, &msg_scope, symbols, config)?;
    let real_oneof_indices: BTreeSet<i32> = real_oneofs.iter().map(|o| o.oneof_index).collect();

    let fields = resolve_message_fields(
        msg_desc,
        raw_msg_name,
        &msg_scope,
        &real_oneof_indices,
        symbols,
        config,
    )?;

    let singular_fields: Vec<&ResolvedField<'_>> = fields
        .iter()
        .filter(|f| matches!(f.label, FieldLabel::LabelRequired | FieldLabel::LabelOptional))
        .collect();

    let mut used_method_names = BTreeSet::new();
    used_method_names.insert("from_bytes".to_string());
    used_method_names.insert("as_bytes".to_string());
    used_method_names.insert("unknown_fields".to_string());
    for f in &fields {
        used_method_names.insert(f.accessor_ident.clone());
    }

    let pad = "    ".repeat(indent);

    // 1. Struct definition
    writeln!(out, "{pad}#[derive(Clone, Copy, Debug)]").unwrap();
    writeln!(out, "{pad}pub struct {view_ident}<'a> {{").unwrap();
    writeln!(out, "{pad}    buf: &'a [u8],").unwrap();
    for f in &singular_fields {
        writeln!(
            out,
            "{pad}    {}: ::roto::runtime::FieldMarker,",
            f.marker_ident
        )
        .unwrap();
    }
    for oneof in &real_oneofs {
        writeln!(out, "{pad}    oneof_{}: u32,", oneof.oneof_index).unwrap();
    }
    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    // 2. Inherent impl
    writeln!(out, "{pad}impl<'a> {view_ident}<'a> {{").unwrap();

    // KNOWN_TAGS
    let tags_list: Vec<String> = fields.iter().map(|f| f.tag.to_string()).collect();
    writeln!(
        out,
        "{pad}    pub const KNOWN_TAGS: &'static [u32] = &[{}];",
        tags_list.join(", ")
    )
    .unwrap();
    writeln!(out).unwrap();

    // from_bytes
    writeln!(
        out,
        "{pad}    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {{"
    )
    .unwrap();
    for f in &singular_fields {
        writeln!(
            out,
            "{pad}        let mut {} = ::roto::runtime::FieldMarker::none();",
            f.marker_ident
        )
        .unwrap();
    }
    for oneof in &real_oneofs {
        writeln!(out, "{pad}        let mut oneof_{}: u32 = 0;", oneof.oneof_index).unwrap();
    }
    if !singular_fields.is_empty() || !real_oneofs.is_empty() {
        writeln!(out).unwrap();
    }

    writeln!(
        out,
        "{pad}        for item in ::roto::runtime::FieldIter::new(buf) {{"
    )
    .unwrap();
    if singular_fields.is_empty() {
        writeln!(out, "{pad}            let _field = item?;").unwrap();
    } else {
        writeln!(out, "{pad}            let field = item?;").unwrap();
        if singular_fields.len() == 1 {
            writeln!(out, "{pad}            #[allow(clippy::single_match)]").unwrap();
        }
        writeln!(
            out,
            "{pad}            match (field.tag, field.wire_type) {{"
        )
        .unwrap();
        for f in &singular_fields {
            let wt = wire_type_variant(f.field_type);
            if let Some(oo_idx) = f.real_oneof_index {
                writeln!(
                    out,
                    "{pad}                ({}, ::roto::runtime::WireType::{wt}) => {{",
                    f.tag
                )
                .unwrap();
                writeln!(out, "{pad}                    {} = field.marker;", f.marker_ident)
                    .unwrap();
                writeln!(out, "{pad}                    oneof_{oo_idx} = {};", f.tag).unwrap();
                writeln!(out, "{pad}                }}").unwrap();
            } else {
                writeln!(
                    out,
                    "{pad}                ({}, ::roto::runtime::WireType::{wt}) => {} = field.marker,",
                    f.tag, f.marker_ident
                )
                .unwrap();
            }
        }
        writeln!(out, "{pad}                _ => {{}}").unwrap();
        writeln!(out, "{pad}            }}").unwrap();
    }
    writeln!(out, "{pad}        }}").unwrap();
    writeln!(out).unwrap();

    // Required field checks
    let required_fields: Vec<&&ResolvedField<'_>> = singular_fields
        .iter()
        .filter(|f| f.label == FieldLabel::LabelRequired)
        .collect();
    for f in &required_fields {
        writeln!(out, "{pad}        if !{}.is_present() {{", f.marker_ident).unwrap();
        writeln!(
            out,
            "{pad}            return Err(::roto::runtime::DecodeError::MissingRequiredField(\"{raw_msg_name}.{}\"));",
            f.raw_name
        )
        .unwrap();
        writeln!(out, "{pad}        }}").unwrap();
    }
    if !required_fields.is_empty() {
        writeln!(out).unwrap();
    }

    // Return Ok(Self { ... })
    if singular_fields.is_empty() && real_oneofs.is_empty() {
        writeln!(out, "{pad}        Ok(Self {{ buf }})").unwrap();
    } else {
        writeln!(out, "{pad}        Ok(Self {{").unwrap();
        writeln!(out, "{pad}            buf,").unwrap();
        for f in &singular_fields {
            writeln!(out, "{pad}            {},", f.marker_ident).unwrap();
        }
        for oneof in &real_oneofs {
            writeln!(out, "{pad}            oneof_{},", oneof.oneof_index).unwrap();
        }
        writeln!(out, "{pad}        }})").unwrap();
    }
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out).unwrap();

    // as_bytes
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(out, "{pad}    pub const fn as_bytes(&self) -> &'a [u8] {{").unwrap();
    writeln!(out, "{pad}        self.buf").unwrap();
    writeln!(out, "{pad}    }}").unwrap();

    // Field accessors
    for f in &fields {
        writeln!(out).unwrap();
        emit_field_accessors(
            f,
            &msg_scope,
            symbols,
            config,
            &mut used_method_names,
            &pad,
            out,
        )?;
    }

    // Oneof accessors
    for oneof in &real_oneofs {
        let oneof_ident = escape_ident(&to_snake_case(&oneof.proto_name));
        if used_method_names.contains(&oneof_ident) {
            continue;
        }
        used_method_names.insert(oneof_ident.clone());
        writeln!(out).unwrap();
        emit_oneof_accessor(oneof, &oneof_ident, &msg_mod_ident, &fields, &pad, out)?;
    }

    // unknown_fields
    writeln!(out).unwrap();
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {{"
    )
    .unwrap();
    writeln!(
        out,
        "{pad}        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)"
    )
    .unwrap();
    writeln!(out, "{pad}    }}").unwrap();

    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    // 3. MessageView trait impl
    writeln!(
        out,
        "{pad}impl<'a> ::roto::runtime::MessageView<'a> for {view_ident}<'a> {{"
    )
    .unwrap();
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {{"
    )
    .unwrap();
    writeln!(out, "{pad}        Self::from_bytes(buf)").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out, "{pad}}}").unwrap();

    Ok(())
}

/// Emits `pub mod <msg_snake> { ... }` containing nested enums, real oneof enums, and
/// nested message views when `msg_desc` has any nested items.
pub fn generate_nested_module(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let raw_msg_name = msg_desc
        .name()
        .map_err(|e| format!("invalid DescriptorProto.name: {e}"))?;
    let msg_scope = compute_message_scope(current_scope, raw_msg_name, symbols);
    let msg_mod_ident = escape_mod_ident(raw_msg_name);

    let mut nested_enums = Vec::new();
    for enum_res in msg_desc.enum_type() {
        let en = enum_res
            .map_err(|e| format!("invalid nested EnumDescriptorProto in '{raw_msg_name}': {e}"))?;
        nested_enums.push(en);
    }

    let real_oneofs = collect_message_oneofs(msg_desc, &msg_scope, symbols, config)?;

    let mut nested_msgs = Vec::new();
    for nested_res in msg_desc.nested_type() {
        let nested = nested_res
            .map_err(|e| format!("invalid nested DescriptorProto in '{raw_msg_name}': {e}"))?;
        nested_msgs.push(nested);
    }

    if nested_enums.is_empty() && real_oneofs.is_empty() && nested_msgs.is_empty() {
        return Ok(());
    }

    let pad = "    ".repeat(indent);
    writeln!(out, "{pad}pub mod {msg_mod_ident} {{").unwrap();

    let mut first = true;
    for en in &nested_enums {
        if !first {
            writeln!(out).unwrap();
        }
        first = false;
        generate_enum(en, indent + 1, out)?;
    }

    for oneof in &real_oneofs {
        if !first {
            writeln!(out).unwrap();
        }
        first = false;
        generate_oneof(oneof, indent + 1, out);
    }

    for nested in &nested_msgs {
        if !first {
            writeln!(out).unwrap();
        }
        first = false;
        generate_message_view(nested, &msg_scope, symbols, config, indent + 1, out)?;
    }

    writeln!(out, "{pad}}}").unwrap();
    Ok(())
}

fn compute_message_scope(
    current_scope: &str,
    raw_msg_name: &str,
    symbols: &SymbolTable,
) -> String {
    let trimmed = current_scope.trim().trim_matches('.');
    if trimmed.is_empty() {
        return format!(".{raw_msg_name}");
    }

    let appended = format!(".{trimmed}.{raw_msg_name}");
    if symbols.get(&appended).is_some() {
        return appended;
    }

    if trimmed == raw_msg_name || trimmed.ends_with(&format!(".{raw_msg_name}")) {
        return format!(".{trimmed}");
    }

    appended
}

fn resolve_message_fields<'a>(
    msg_desc: &DescriptorProtoView<'a>,
    raw_msg_name: &str,
    msg_scope: &str,
    real_oneof_indices: &BTreeSet<i32>,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<Vec<ResolvedField<'a>>, String> {
    let mut resolved_fields = Vec::new();
    let mut used_markers = BTreeSet::new();
    used_markers.insert("buf".to_string());
    used_markers.insert("item".to_string());
    used_markers.insert("field".to_string());

    for field_res in msg_desc.field() {
        let proto_field = field_res
            .map_err(|e| format!("invalid FieldDescriptorProto in '{raw_msg_name}': {e}"))?;
        let raw_name = proto_field
            .name()
            .map_err(|e| format!("invalid FieldDescriptorProto.name in '{raw_msg_name}': {e}"))?;
        if raw_name.is_empty() {
            return Err(format!(
                "FieldDescriptorProto.name cannot be empty in message '{raw_msg_name}'"
            ));
        }

        let number = proto_field
            .number()
            .map_err(|e| format!("invalid FieldDescriptorProto.number in '{raw_name}': {e}"))?;
        if number <= 0 {
            return Err(format!(
                "invalid non-positive field number {number} on '{raw_msg_name}.{raw_name}'"
            ));
        }
        let tag = number as u32;

        let label = proto_field
            .label()
            .map_err(|e| format!("invalid FieldDescriptorProto.label in '{raw_name}': {e}"))?;
        let field_type = proto_field
            .r#type()
            .map_err(|e| format!("invalid FieldDescriptorProto.type in '{raw_name}': {e}"))?;

        let resolved_type = symbols.resolve_field(msg_scope, &proto_field, config)?;

        let real_oneof_index = if proto_field.has_oneof_index() {
            let idx = proto_field
                .oneof_index()
                .map_err(|e| format!("invalid oneof_index in '{raw_name}': {e}"))?;
            let is_proto3_opt = proto_field
                .proto3_optional()
                .map_err(|e| format!("invalid proto3_optional in '{raw_name}': {e}"))?;
            if !is_proto3_opt && real_oneof_indices.contains(&idx) {
                Some(idx)
            } else {
                None
            }
        } else {
            None
        };

        let snake_name = to_snake_case(raw_name);
        let mut accessor_ident = escape_ident(&snake_name);
        if matches!(
            accessor_ident.as_str(),
            "from_bytes" | "as_bytes" | "unknown_fields"
        ) {
            accessor_ident.push('_');
        }

        let has_ident = format!("has_{snake_name}");

        let candidate_marker = escape_ident(&snake_name);
        let marker_ident = if used_markers.contains(&candidate_marker)
            || candidate_marker.starts_with("oneof_")
            || candidate_marker.starts_with("f_")
        {
            format!("f_{tag}")
        } else {
            candidate_marker
        };
        used_markers.insert(marker_ident.clone());

        resolved_fields.push(ResolvedField {
            proto_field,
            raw_name,
            accessor_ident,
            has_ident,
            marker_ident,
            tag,
            label,
            field_type,
            resolved_type,
            real_oneof_index,
        });
    }

    Ok(resolved_fields)
}

fn wire_type_variant(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::TypeInt32
        | FieldType::TypeInt64
        | FieldType::TypeUint32
        | FieldType::TypeUint64
        | FieldType::TypeSint32
        | FieldType::TypeSint64
        | FieldType::TypeBool
        | FieldType::TypeEnum => "Varint",
        FieldType::TypeFixed32 | FieldType::TypeSfixed32 | FieldType::TypeFloat => "Fixed32",
        FieldType::TypeFixed64 | FieldType::TypeSfixed64 | FieldType::TypeDouble => "Fixed64",
        FieldType::TypeString
        | FieldType::TypeBytes
        | FieldType::TypeMessage
        | FieldType::TypeGroup => "LengthDelimited",
    }
}

fn emit_field_accessors(
    f: &ResolvedField<'_>,
    msg_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    used_method_names: &mut BTreeSet<String>,
    pad: &str,
    out: &mut String,
) -> Result<(), String> {
    match f.label {
        FieldLabel::LabelRequired => {
            if !used_method_names.contains(&f.has_ident) {
                used_method_names.insert(f.has_ident.clone());
                writeln!(out, "{pad}    #[inline]").unwrap();
                writeln!(
                    out,
                    "{pad}    pub const fn {}(&self) -> bool {{",
                    f.has_ident
                )
                .unwrap();
                writeln!(out, "{pad}        self.{}.is_present()", f.marker_ident).unwrap();
                writeln!(out, "{pad}    }}").unwrap();
                writeln!(out).unwrap();
            }

            let ret_ty = singular_return_type(f)?;
            let decode_stmt = point_decode_return_expr(f)?;
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub fn {}(&self) -> Result<{ret_ty}, ::roto::runtime::DecodeError> {{",
                f.accessor_ident
            )
            .unwrap();
            writeln!(out, "{pad}        {decode_stmt}").unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
        FieldLabel::LabelOptional => {
            if !used_method_names.contains(&f.has_ident) {
                used_method_names.insert(f.has_ident.clone());
                writeln!(out, "{pad}    #[inline]").unwrap();
                writeln!(
                    out,
                    "{pad}    pub const fn {}(&self) -> bool {{",
                    f.has_ident
                )
                .unwrap();
                writeln!(out, "{pad}        self.{}.is_present()", f.marker_ident).unwrap();
                writeln!(out, "{pad}    }}").unwrap();
                writeln!(out).unwrap();
            }

            if matches!(f.field_type, FieldType::TypeMessage | FieldType::TypeGroup) {
                let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                    format!("missing resolved type for message field '{}'", f.raw_name)
                })?;
                let view_ty = resolved.view_type("'a");
                let view_ident = &resolved.view_ident_path;
                writeln!(out, "{pad}    #[inline]").unwrap();
                writeln!(
                    out,
                    "{pad}    pub fn {}(&self) -> Result<Option<{view_ty}>, ::roto::runtime::DecodeError> {{",
                    f.accessor_ident
                )
                .unwrap();
                writeln!(out, "{pad}        if !self.{}.is_present() {{", f.marker_ident).unwrap();
                writeln!(out, "{pad}            return Ok(None);").unwrap();
                writeln!(out, "{pad}        }}").unwrap();
                writeln!(
                    out,
                    "{pad}        {view_ident}::from_bytes(self.{}.slice(self.buf)).map(Some)",
                    f.marker_ident
                )
                .unwrap();
                writeln!(out, "{pad}    }}").unwrap();
            } else {
                let ret_ty = singular_return_type(f)?;
                let default_expr =
                    resolve_field_default(&f.proto_field, msg_scope, symbols, config)?;
                let decode_stmt = point_decode_return_expr(f)?;
                writeln!(out, "{pad}    #[inline]").unwrap();
                writeln!(
                    out,
                    "{pad}    pub fn {}(&self) -> Result<{ret_ty}, ::roto::runtime::DecodeError> {{",
                    f.accessor_ident
                )
                .unwrap();
                writeln!(out, "{pad}        if !self.{}.is_present() {{", f.marker_ident).unwrap();
                writeln!(out, "{pad}            return Ok({default_expr});").unwrap();
                writeln!(out, "{pad}        }}").unwrap();
                writeln!(out, "{pad}        {decode_stmt}").unwrap();
                writeln!(out, "{pad}    }}").unwrap();
            }
        }
        FieldLabel::LabelRepeated => {
            let (iter_ty, iter_ctor) = repeated_iter_type_and_ctor(f)?;
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub const fn {}(&self) -> {iter_ty} {{",
                f.accessor_ident
            )
            .unwrap();
            writeln!(out, "{pad}        {iter_ctor}(self.buf, {})", f.tag).unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
    }

    Ok(())
}

fn singular_return_type(f: &ResolvedField<'_>) -> Result<String, String> {
    match f.field_type {
        FieldType::TypeInt32 | FieldType::TypeSint32 | FieldType::TypeSfixed32 => {
            Ok("i32".to_string())
        }
        FieldType::TypeInt64 | FieldType::TypeSint64 | FieldType::TypeSfixed64 => {
            Ok("i64".to_string())
        }
        FieldType::TypeUint32 | FieldType::TypeFixed32 => Ok("u32".to_string()),
        FieldType::TypeUint64 | FieldType::TypeFixed64 => Ok("u64".to_string()),
        FieldType::TypeFloat => Ok("f32".to_string()),
        FieldType::TypeDouble => Ok("f64".to_string()),
        FieldType::TypeBool => Ok("bool".to_string()),
        FieldType::TypeString => Ok("&'a str".to_string()),
        FieldType::TypeBytes => Ok("&'a [u8]".to_string()),
        FieldType::TypeEnum => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for enum field '{}'", f.raw_name)
            })?;
            Ok(resolved.enum_path().to_string())
        }
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for message field '{}'", f.raw_name)
            })?;
            Ok(resolved.view_type("'a"))
        }
    }
}

fn point_decode_return_expr(f: &ResolvedField<'_>) -> Result<String, String> {
    let m = &f.marker_ident;
    match f.field_type {
        FieldType::TypeInt32 => {
            Ok(format!("Ok(::roto::runtime::decode_varint_at(self.buf, self.{m})? as i32)"))
        }
        FieldType::TypeInt64 => {
            Ok(format!("Ok(::roto::runtime::decode_varint_at(self.buf, self.{m})? as i64)"))
        }
        FieldType::TypeUint32 => {
            Ok(format!("Ok(::roto::runtime::decode_varint_at(self.buf, self.{m})? as u32)"))
        }
        FieldType::TypeUint64 => {
            Ok(format!("::roto::runtime::decode_varint_at(self.buf, self.{m})"))
        }
        FieldType::TypeSint32 => {
            Ok(format!("::roto::runtime::decode_sint32_at(self.buf, self.{m})"))
        }
        FieldType::TypeSint64 => {
            Ok(format!("::roto::runtime::decode_sint64_at(self.buf, self.{m})"))
        }
        FieldType::TypeFixed32 => {
            Ok(format!("::roto::runtime::decode_fixed32_at(self.buf, self.{m})"))
        }
        FieldType::TypeFixed64 => {
            Ok(format!("::roto::runtime::decode_fixed64_at(self.buf, self.{m})"))
        }
        FieldType::TypeSfixed32 => {
            Ok(format!("::roto::runtime::decode_sfixed32_at(self.buf, self.{m})"))
        }
        FieldType::TypeSfixed64 => {
            Ok(format!("::roto::runtime::decode_sfixed64_at(self.buf, self.{m})"))
        }
        FieldType::TypeFloat => {
            Ok(format!("::roto::runtime::decode_f32_at(self.buf, self.{m})"))
        }
        FieldType::TypeDouble => {
            Ok(format!("::roto::runtime::decode_f64_at(self.buf, self.{m})"))
        }
        FieldType::TypeBool => {
            Ok(format!("::roto::runtime::decode_bool_at(self.buf, self.{m})"))
        }
        FieldType::TypeString => {
            Ok(format!("::roto::runtime::decode_str_at(self.buf, self.{m})"))
        }
        FieldType::TypeBytes => {
            Ok(format!("::roto::runtime::decode_bytes_at(self.buf, self.{m})"))
        }
        FieldType::TypeEnum => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for enum field '{}'", f.raw_name)
            })?;
            let enum_path = resolved.enum_path();
            Ok(format!(
                "Ok(<{enum_path} as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.{m})?))"
            ))
        }
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for message field '{}'", f.raw_name)
            })?;
            let view_ident = &resolved.view_ident_path;
            Ok(format!("{view_ident}::from_bytes(self.{m}.slice(self.buf))"))
        }
    }
}

fn point_decode_unwrapped_expr(f: &ResolvedField<'_>) -> Result<String, String> {
    let m = &f.marker_ident;
    match f.field_type {
        FieldType::TypeInt32 => {
            Ok(format!("::roto::runtime::decode_varint_at(self.buf, self.{m})? as i32"))
        }
        FieldType::TypeInt64 => {
            Ok(format!("::roto::runtime::decode_varint_at(self.buf, self.{m})? as i64"))
        }
        FieldType::TypeUint32 => {
            Ok(format!("::roto::runtime::decode_varint_at(self.buf, self.{m})? as u32"))
        }
        FieldType::TypeUint64 => {
            Ok(format!("::roto::runtime::decode_varint_at(self.buf, self.{m})?"))
        }
        FieldType::TypeSint32 => {
            Ok(format!("::roto::runtime::decode_sint32_at(self.buf, self.{m})?"))
        }
        FieldType::TypeSint64 => {
            Ok(format!("::roto::runtime::decode_sint64_at(self.buf, self.{m})?"))
        }
        FieldType::TypeFixed32 => {
            Ok(format!("::roto::runtime::decode_fixed32_at(self.buf, self.{m})?"))
        }
        FieldType::TypeFixed64 => {
            Ok(format!("::roto::runtime::decode_fixed64_at(self.buf, self.{m})?"))
        }
        FieldType::TypeSfixed32 => {
            Ok(format!("::roto::runtime::decode_sfixed32_at(self.buf, self.{m})?"))
        }
        FieldType::TypeSfixed64 => {
            Ok(format!("::roto::runtime::decode_sfixed64_at(self.buf, self.{m})?"))
        }
        FieldType::TypeFloat => {
            Ok(format!("::roto::runtime::decode_f32_at(self.buf, self.{m})?"))
        }
        FieldType::TypeDouble => {
            Ok(format!("::roto::runtime::decode_f64_at(self.buf, self.{m})?"))
        }
        FieldType::TypeBool => {
            Ok(format!("::roto::runtime::decode_bool_at(self.buf, self.{m})?"))
        }
        FieldType::TypeString => {
            Ok(format!("::roto::runtime::decode_str_at(self.buf, self.{m})?"))
        }
        FieldType::TypeBytes => {
            Ok(format!("::roto::runtime::decode_bytes_at(self.buf, self.{m})?"))
        }
        FieldType::TypeEnum => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for enum field '{}'", f.raw_name)
            })?;
            let enum_path = resolved.enum_path();
            Ok(format!(
                "<{enum_path} as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.{m})?)"
            ))
        }
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for message field '{}'", f.raw_name)
            })?;
            let view_ident = &resolved.view_ident_path;
            Ok(format!("{view_ident}::from_bytes(self.{m}.slice(self.buf))?"))
        }
    }
}

fn repeated_iter_type_and_ctor(f: &ResolvedField<'_>) -> Result<(String, &'static str), String> {
    match f.field_type {
        FieldType::TypeInt32 => Ok((
            "::roto::runtime::RepeatedVarintIter<'a, i32>".to_string(),
            "::roto::runtime::RepeatedVarintIter::new",
        )),
        FieldType::TypeInt64 => Ok((
            "::roto::runtime::RepeatedVarintIter<'a, i64>".to_string(),
            "::roto::runtime::RepeatedVarintIter::new",
        )),
        FieldType::TypeUint32 => Ok((
            "::roto::runtime::RepeatedVarintIter<'a, u32>".to_string(),
            "::roto::runtime::RepeatedVarintIter::new",
        )),
        FieldType::TypeUint64 => Ok((
            "::roto::runtime::RepeatedVarintIter<'a, u64>".to_string(),
            "::roto::runtime::RepeatedVarintIter::new",
        )),
        FieldType::TypeBool => Ok((
            "::roto::runtime::RepeatedVarintIter<'a, bool>".to_string(),
            "::roto::runtime::RepeatedVarintIter::new",
        )),
        FieldType::TypeEnum => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for repeated enum field '{}'", f.raw_name)
            })?;
            let enum_path = resolved.enum_path();
            Ok((
                format!("::roto::runtime::RepeatedVarintIter<'a, {enum_path}>"),
                "::roto::runtime::RepeatedVarintIter::new",
            ))
        }
        FieldType::TypeSint32 => Ok((
            "::roto::runtime::RepeatedZigZagIter<'a, i32>".to_string(),
            "::roto::runtime::RepeatedZigZagIter::new",
        )),
        FieldType::TypeSint64 => Ok((
            "::roto::runtime::RepeatedZigZagIter<'a, i64>".to_string(),
            "::roto::runtime::RepeatedZigZagIter::new",
        )),
        FieldType::TypeFixed32 => Ok((
            "::roto::runtime::RepeatedFixed32Iter<'a, u32>".to_string(),
            "::roto::runtime::RepeatedFixed32Iter::new",
        )),
        FieldType::TypeSfixed32 => Ok((
            "::roto::runtime::RepeatedFixed32Iter<'a, i32>".to_string(),
            "::roto::runtime::RepeatedFixed32Iter::new",
        )),
        FieldType::TypeFloat => Ok((
            "::roto::runtime::RepeatedFixed32Iter<'a, f32>".to_string(),
            "::roto::runtime::RepeatedFixed32Iter::new",
        )),
        FieldType::TypeFixed64 => Ok((
            "::roto::runtime::RepeatedFixed64Iter<'a, u64>".to_string(),
            "::roto::runtime::RepeatedFixed64Iter::new",
        )),
        FieldType::TypeSfixed64 => Ok((
            "::roto::runtime::RepeatedFixed64Iter<'a, i64>".to_string(),
            "::roto::runtime::RepeatedFixed64Iter::new",
        )),
        FieldType::TypeDouble => Ok((
            "::roto::runtime::RepeatedFixed64Iter<'a, f64>".to_string(),
            "::roto::runtime::RepeatedFixed64Iter::new",
        )),
        FieldType::TypeString => Ok((
            "::roto::runtime::RepeatedStrIter<'a>".to_string(),
            "::roto::runtime::RepeatedStrIter::new",
        )),
        FieldType::TypeBytes => Ok((
            "::roto::runtime::RepeatedBytesIter<'a>".to_string(),
            "::roto::runtime::RepeatedBytesIter::new",
        )),
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = f.resolved_type.as_ref().ok_or_else(|| {
                format!("missing resolved type for repeated message field '{}'", f.raw_name)
            })?;
            let view_ty = resolved.view_type("'a");
            Ok((
                format!("::roto::runtime::RepeatedMessageIter<'a, {view_ty}>"),
                "::roto::runtime::RepeatedMessageIter::new",
            ))
        }
    }
}

fn emit_oneof_accessor(
    oneof: &OneofInfo,
    oneof_ident: &str,
    msg_mod_ident: &str,
    fields: &[ResolvedField<'_>],
    pad: &str,
    out: &mut String,
) -> Result<(), String> {
    let oneof_ty = if oneof.has_lifetime {
        format!("{msg_mod_ident}::{}<'a>", oneof.enum_ident)
    } else {
        format!("{msg_mod_ident}::{}", oneof.enum_ident)
    };

    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    pub fn {oneof_ident}(&self) -> Result<Option<{oneof_ty}>, ::roto::runtime::DecodeError> {{"
    )
    .unwrap();
    writeln!(out, "{pad}        match self.oneof_{} {{", oneof.oneof_index).unwrap();

    for variant in &oneof.variants {
        let tag = variant.field_number as u32;
        let field = fields
            .iter()
            .find(|f| f.tag == tag)
            .ok_or_else(|| format!("missing field for oneof variant '{}'", variant.proto_field_name))?;
        let val_expr = point_decode_unwrapped_expr(field)?;
        writeln!(
            out,
            "{pad}            {tag} => Ok(Some({msg_mod_ident}::{}::{}({val_expr}))),",
            oneof.enum_ident, variant.variant_ident
        )
        .unwrap();
    }

    writeln!(out, "{pad}            _ => Ok(None),").unwrap();
    writeln!(out, "{pad}        }}").unwrap();
    writeln!(out, "{pad}    }}").unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    use google_protobuf_roto::google::protobuf::{
        FileDescriptorProtoBuilder, FileDescriptorProtoView, FileDescriptorSetView,
    };
    use roto::runtime::BackwardWriter;

    fn load_descriptor_fds() -> Vec<u8> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let proto_root = format!("{manifest_dir}/../google-protobuf-roto/proto");

        let out = Command::new("protoc")
            .arg(format!("--proto_path={proto_root}"))
            .arg("--include_imports")
            .arg("--descriptor_set_out=/dev/stdout")
            .arg("google/protobuf/compiler/plugin.proto")
            .output()
            .expect("failed to invoke protoc on plugin.proto");
        assert!(
            out.status.success(),
            "protoc failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    }

    #[test]
    fn generates_location_and_sensor_reading_views_matching_readme_architecture() {
        let mut buf = [0u8; 2048];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut fb = FileDescriptorProtoBuilder::new(&mut writer);

            // message SensorReading
            fb.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeUint32)
                        .write_label(FieldLabel::LabelRepeated)
                        .write_number(4)
                        .write_name("samples");
                })
                .write_field(|f| {
                    f.write_type_name(".telemetry.Location")
                        .write_type(FieldType::TypeMessage)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(3)
                        .write_name("location");
                })
                .write_field(|f| {
                    f.write_default_value("unknown")
                        .write_type(FieldType::TypeString)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(2)
                        .write_name("device_id");
                })
                .write_field(|f| {
                    f.write_type(FieldType::TypeUint64)
                        .write_label(FieldLabel::LabelRequired)
                        .write_number(1)
                        .write_name("timestamp_ms");
                })
                .write_name("SensorReading");
            })
            // message Location
            .write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeInt32)
                        .write_label(FieldLabel::LabelRequired)
                        .write_number(2)
                        .write_name("lon_e7");
                })
                .write_field(|f| {
                    f.write_type(FieldType::TypeInt32)
                        .write_label(FieldLabel::LabelRequired)
                        .write_number(1)
                        .write_name("lat_e7");
                })
                .write_name("Location");
            })
            .write_syntax("proto2")
            .write_package("telemetry")
            .write_name("sensor.proto");
        }

        let file_view = FileDescriptorProtoView::from_bytes(writer.finish().unwrap()).unwrap();
        let symbols = SymbolTable::from_files([file_view]).unwrap();
        let config = GeneratorConfig::default();

        let mut msgs = file_view.message_type();
        let loc_desc = msgs.next().unwrap().unwrap();
        let sr_desc = msgs.next().unwrap().unwrap();

        let mut loc_out = String::new();
        generate_message_view(&loc_desc, ".telemetry", &symbols, &config, 0, &mut loc_out).unwrap();
        let expected_loc = "\
#[derive(Clone, Copy, Debug)]
pub struct LocationView<'a> {
    buf: &'a [u8],
    lat_e7: ::roto::runtime::FieldMarker,
    lon_e7: ::roto::runtime::FieldMarker,
}

impl<'a> LocationView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut lat_e7 = ::roto::runtime::FieldMarker::none();
        let mut lon_e7 = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => lat_e7 = field.marker,
                (2, ::roto::runtime::WireType::Varint) => lon_e7 = field.marker,
                _ => {}
            }
        }

        if !lat_e7.is_present() {
            return Err(::roto::runtime::DecodeError::MissingRequiredField(\"Location.lat_e7\"));
        }
        if !lon_e7.is_present() {
            return Err(::roto::runtime::DecodeError::MissingRequiredField(\"Location.lon_e7\"));
        }

        Ok(Self {
            buf,
            lat_e7,
            lon_e7,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_lat_e7(&self) -> bool {
        self.lat_e7.is_present()
    }

    #[inline]
    pub fn lat_e7(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(::roto::runtime::decode_varint_at(self.buf, self.lat_e7)? as i32)
    }

    #[inline]
    pub const fn has_lon_e7(&self) -> bool {
        self.lon_e7.is_present()
    }

    #[inline]
    pub fn lon_e7(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(::roto::runtime::decode_varint_at(self.buf, self.lon_e7)? as i32)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for LocationView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}
";
        assert_eq!(loc_out, expected_loc);

        let mut sr_out = String::new();
        generate_message_view(&sr_desc, ".telemetry", &symbols, &config, 0, &mut sr_out).unwrap();
        let expected_sr = "\
#[derive(Clone, Copy, Debug)]
pub struct SensorReadingView<'a> {
    buf: &'a [u8],
    timestamp_ms: ::roto::runtime::FieldMarker,
    device_id: ::roto::runtime::FieldMarker,
    location: ::roto::runtime::FieldMarker,
}

impl<'a> SensorReadingView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut timestamp_ms = ::roto::runtime::FieldMarker::none();
        let mut device_id = ::roto::runtime::FieldMarker::none();
        let mut location = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => timestamp_ms = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => device_id = field.marker,
                (3, ::roto::runtime::WireType::LengthDelimited) => location = field.marker,
                _ => {}
            }
        }

        if !timestamp_ms.is_present() {
            return Err(::roto::runtime::DecodeError::MissingRequiredField(\"SensorReading.timestamp_ms\"));
        }

        Ok(Self {
            buf,
            timestamp_ms,
            device_id,
            location,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_timestamp_ms(&self) -> bool {
        self.timestamp_ms.is_present()
    }

    #[inline]
    pub fn timestamp_ms(&self) -> Result<u64, ::roto::runtime::DecodeError> {
        ::roto::runtime::decode_varint_at(self.buf, self.timestamp_ms)
    }

    #[inline]
    pub const fn has_device_id(&self) -> bool {
        self.device_id.is_present()
    }

    #[inline]
    pub fn device_id(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.device_id.is_present() {
            return Ok(\"unknown\");
        }
        ::roto::runtime::decode_str_at(self.buf, self.device_id)
    }

    #[inline]
    pub const fn has_location(&self) -> bool {
        self.location.is_present()
    }

    #[inline]
    pub fn location(&self) -> Result<Option<crate::telemetry::LocationView<'a>>, ::roto::runtime::DecodeError> {
        if !self.location.is_present() {
            return Ok(None);
        }
        crate::telemetry::LocationView::from_bytes(self.location.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn samples(&self) -> ::roto::runtime::RepeatedVarintIter<'a, u32> {
        ::roto::runtime::RepeatedVarintIter::new(self.buf, 4)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for SensorReadingView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}
";
        assert_eq!(sr_out, expected_sr);
    }

    #[test]
    fn generates_all_18_field_types_oneofs_nested_types_and_buf_collision_avoidance() {
        let mut buf = [0u8; 4096];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut fb = FileDescriptorProtoBuilder::new(&mut writer);

            fb.write_message_type(|msg| {
                // Real oneof 1 (`threshold`, scalar/enum-only -> no lifetime parameter)
                msg.write_oneof_decl(|oo| {
                    oo.write_name("threshold");
                })
                // Real oneof 0 (`choice`, borrowed -> has `<'a>` lifetime parameter)
                .write_oneof_decl(|oo| {
                    oo.write_name("choice");
                })
                // Nested message `Inner`
                .write_nested_type(|inner| {
                    inner
                        .write_field(|f| {
                            f.write_type(FieldType::TypeInt32)
                                .write_label(FieldLabel::LabelOptional)
                                .write_number(1)
                                .write_name("val");
                        })
                        .write_name("Inner");
                })
                // Nested enum `Kind`
                .write_enum_type(|en| {
                    en.write_value(|v| {
                        v.write_number(1).write_name("KIND_B");
                    })
                    .write_value(|v| {
                        v.write_number(0).write_name("KIND_A");
                    })
                    .write_name("Kind");
                })
                // Oneof 1 fields (tags 39, 40)
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type_name("Kind")
                        .write_type(FieldType::TypeEnum)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(40)
                        .write_name("thresh_kind");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeSint32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(39)
                        .write_name("thresh_code");
                })
                // Oneof 0 fields (tags 37, 38)
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type_name("Inner")
                        .write_type(FieldType::TypeMessage)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(38)
                        .write_name("choice_inner");
                })
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type(FieldType::TypeString)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(37)
                        .write_name("choice_str");
                });

                // Repeated fields for all 18 types (tags 19..=36)
                let all_types = [
                    (1, FieldType::TypeDouble, ""),
                    (2, FieldType::TypeFloat, ""),
                    (3, FieldType::TypeInt64, ""),
                    (4, FieldType::TypeUint64, ""),
                    (5, FieldType::TypeInt32, ""),
                    (6, FieldType::TypeFixed64, ""),
                    (7, FieldType::TypeFixed32, ""),
                    (8, FieldType::TypeBool, ""),
                    (9, FieldType::TypeString, ""),
                    (10, FieldType::TypeGroup, "Inner"),
                    (11, FieldType::TypeMessage, "Inner"),
                    (12, FieldType::TypeBytes, ""),
                    (13, FieldType::TypeUint32, ""),
                    (14, FieldType::TypeEnum, "Kind"),
                    (15, FieldType::TypeSfixed32, ""),
                    (16, FieldType::TypeSfixed64, ""),
                    (17, FieldType::TypeSint32, ""),
                    (18, FieldType::TypeSint64, ""),
                ];

                for &(idx, fty, tname) in all_types.iter().rev() {
                    let rep_tag = idx + 18;
                    msg.write_field(|f| {
                        if !tname.is_empty() {
                            f.write_type_name(tname);
                        }
                        f.write_type(fty)
                            .write_label(FieldLabel::LabelRepeated)
                            .write_number(rep_tag)
                            .write_name(&format!("rep_{idx}"));
                    });
                }

                // Singular fields for all 18 types (tags 1..=18), with tag 1 named `buf` to test collision avoidance
                for &(idx, fty, tname) in all_types.iter().rev() {
                    msg.write_field(|f| {
                        if !tname.is_empty() {
                            f.write_type_name(tname);
                        }
                        f.write_type(fty)
                            .write_label(FieldLabel::LabelOptional)
                            .write_number(idx);
                        if idx == 1 {
                            f.write_name("buf");
                        } else {
                            f.write_name(&format!("opt_{idx}"));
                        }
                    });
                }

                msg.write_name("AllTypes");
            })
            .write_syntax("proto2")
            .write_package("demo")
            .write_name("all_types.proto");
        }

        let file_view = FileDescriptorProtoView::from_bytes(writer.finish().unwrap()).unwrap();
        let symbols = SymbolTable::from_files([file_view]).unwrap();
        let config = GeneratorConfig::default();

        let msg_desc = file_view.message_type().next().unwrap().unwrap();
        let mut out = String::new();
        generate_message_view(&msg_desc, ".demo", &symbols, &config, 0, &mut out).unwrap();

        // Check `buf` collision avoidance: struct field is `f_1`, accessor is `buf(&self)` and `has_buf(&self)`
        assert!(out.contains("    buf: &'a [u8],\n    f_1: ::roto::runtime::FieldMarker,"));
        assert!(out.contains("pub const fn has_buf(&self) -> bool {\n        self.f_1.is_present()\n    }"));
        assert!(out.contains("pub fn buf(&self) -> Result<f64, ::roto::runtime::DecodeError>"));

        // Check oneof tracking fields and match arms (both borrowed `<'a>` and scalar-only without `<'a>`)
        assert!(out.contains("    oneof_0: u32,\n    oneof_1: u32,"));
        assert!(out.contains("(37, ::roto::runtime::WireType::LengthDelimited) => {\n                    choice_str = field.marker;\n                    oneof_0 = 37;\n                }"));
        assert!(out.contains("pub fn choice(&self) -> Result<Option<all_types::Choice<'a>>, ::roto::runtime::DecodeError>"));
        assert!(out.contains("37 => Ok(Some(all_types::Choice::ChoiceStr(::roto::runtime::decode_str_at(self.buf, self.choice_str)?))),"));
        assert!(out.contains("38 => Ok(Some(all_types::Choice::ChoiceInner(crate::demo::all_types::InnerView::from_bytes(self.choice_inner.slice(self.buf))?))),"));
        assert!(out.contains("pub fn threshold(&self) -> Result<Option<all_types::Threshold>, ::roto::runtime::DecodeError>"));
        assert!(out.contains("39 => Ok(Some(all_types::Threshold::ThreshCode(::roto::runtime::decode_sint32_at(self.buf, self.thresh_code)?))),"));
        assert!(out.contains("40 => Ok(Some(all_types::Threshold::ThreshKind(<crate::demo::all_types::Kind as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.thresh_kind)?)))),"));

        // Check point decoders for all scalar/composite types
        assert!(out.contains("::roto::runtime::decode_f64_at(self.buf, self.f_1)"));
        assert!(out.contains("::roto::runtime::decode_f32_at(self.buf, self.opt_2)"));
        assert!(out.contains("Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_3)? as i64)"));
        assert!(out.contains("::roto::runtime::decode_varint_at(self.buf, self.opt_4)"));
        assert!(out.contains("Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_5)? as i32)"));
        assert!(out.contains("::roto::runtime::decode_fixed64_at(self.buf, self.opt_6)"));
        assert!(out.contains("::roto::runtime::decode_fixed32_at(self.buf, self.opt_7)"));
        assert!(out.contains("::roto::runtime::decode_bool_at(self.buf, self.opt_8)"));
        assert!(out.contains("::roto::runtime::decode_str_at(self.buf, self.opt_9)"));
        assert!(out.contains("crate::demo::all_types::InnerView::from_bytes(self.opt_10.slice(self.buf)).map(Some)"));
        assert!(out.contains("crate::demo::all_types::InnerView::from_bytes(self.opt_11.slice(self.buf)).map(Some)"));
        assert!(out.contains("::roto::runtime::decode_bytes_at(self.buf, self.opt_12)"));
        assert!(out.contains("Ok(::roto::runtime::decode_varint_at(self.buf, self.opt_13)? as u32)"));
        assert!(out.contains("Ok(<crate::demo::all_types::Kind as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.opt_14)?))"));
        assert!(out.contains("::roto::runtime::decode_sfixed32_at(self.buf, self.opt_15)"));
        assert!(out.contains("::roto::runtime::decode_sfixed64_at(self.buf, self.opt_16)"));
        assert!(out.contains("::roto::runtime::decode_sint32_at(self.buf, self.opt_17)"));
        assert!(out.contains("::roto::runtime::decode_sint64_at(self.buf, self.opt_18)"));

        // Check repeated iterators for all 18 types
        assert!(out.contains("pub const fn rep_1(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, f64>"));
        assert!(out.contains("pub const fn rep_2(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, f32>"));
        assert!(out.contains("pub const fn rep_3(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i64>"));
        assert!(out.contains("pub const fn rep_4(&self) -> ::roto::runtime::RepeatedVarintIter<'a, u64>"));
        assert!(out.contains("pub const fn rep_5(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32>"));
        assert!(out.contains("pub const fn rep_6(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, u64>"));
        assert!(out.contains("pub const fn rep_7(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, u32>"));
        assert!(out.contains("pub const fn rep_8(&self) -> ::roto::runtime::RepeatedVarintIter<'a, bool>"));
        assert!(out.contains("pub const fn rep_9(&self) -> ::roto::runtime::RepeatedStrIter<'a>"));
        assert!(out.contains("pub const fn rep_10(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::demo::all_types::InnerView<'a>>"));
        assert!(out.contains("pub const fn rep_11(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::demo::all_types::InnerView<'a>>"));
        assert!(out.contains("pub const fn rep_12(&self) -> ::roto::runtime::RepeatedBytesIter<'a>"));
        assert!(out.contains("pub const fn rep_13(&self) -> ::roto::runtime::RepeatedVarintIter<'a, u32>"));
        assert!(out.contains("pub const fn rep_14(&self) -> ::roto::runtime::RepeatedVarintIter<'a, crate::demo::all_types::Kind>"));
        assert!(out.contains("pub const fn rep_15(&self) -> ::roto::runtime::RepeatedFixed32Iter<'a, i32>"));
        assert!(out.contains("pub const fn rep_16(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, i64>"));
        assert!(out.contains("pub const fn rep_17(&self) -> ::roto::runtime::RepeatedZigZagIter<'a, i32>"));
        assert!(out.contains("pub const fn rep_18(&self) -> ::roto::runtime::RepeatedZigZagIter<'a, i64>"));

        // Check nested module `pub mod all_types`
        assert!(out.contains("pub mod all_types {"));
        assert!(out.contains("    pub enum Kind {"));
        assert!(out.contains("    pub enum Choice<'a> {"));
        assert!(out.contains("    pub enum Threshold {"));
        assert!(out.contains("    pub struct InnerView<'a> {"));
    }

    #[test]
    fn generates_views_for_all_real_descriptor_and_plugin_proto_messages() {
        let fds_bytes = load_descriptor_fds();
        let fds = FileDescriptorSetView::from_bytes(&fds_bytes).unwrap();
        let symbols = SymbolTable::from_files(fds.file().map(|f| f.unwrap())).unwrap();
        let config = GeneratorConfig::default();

        for file_res in fds.file() {
            let file = file_res.unwrap();
            let pkg = file.package().unwrap();
            let scope = if pkg.is_empty() {
                String::new()
            } else {
                format!(".{pkg}")
            };

            for msg_res in file.message_type() {
                let msg = msg_res.unwrap();
                let mut out = String::new();
                generate_message_view(&msg, &scope, &symbols, &config, 0, &mut out).unwrap();
                let msg_name = msg.name().unwrap();
                assert!(out.contains(&format!("pub struct {msg_name}View<'a>")));
            }
        }
    }
}
