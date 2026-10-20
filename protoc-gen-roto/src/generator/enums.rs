use std::fmt::Write as _;

use google_protobuf_roto::google::protobuf::{
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    DescriptorProtoView, EnumDescriptorProtoView, FieldDescriptorProtoView,
    OneofDescriptorProtoView,
};

use super::config::GeneratorConfig;
use super::ident::{escape_ident, sanitize_type_ident, to_pascal_case};
use super::symbols::SymbolTable;

/// Metadata for a single member field of a Protobuf `oneof`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneofVariantInfo {
    /// Original Protobuf field name (e.g. `"device_id"`).
    pub proto_field_name: String,
    /// Escaped Rust `PascalCase` variant identifier (e.g. `"DeviceId"`).
    pub variant_ident: String,
    /// Protobuf field tag number (e.g. `2`).
    pub field_number: i32,
    /// Protobuf field descriptor type.
    pub field_type: FieldType,
    /// Resolved Rust view type for the variant payload (e.g. `&'a str`, `i32`, `crate::...::SubView<'a>`).
    pub rust_type: String,
    /// True if this variant's payload borrows from `'a`.
    pub borrows_lifetime: bool,
}

/// Resolved metadata for a real (non-`proto3_optional`) Protobuf `oneof` declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneofInfo {
    /// Index of this `oneof` in `DescriptorProtoView::oneof_decl()`.
    pub oneof_index: i32,
    /// Original Protobuf `oneof` name (e.g. `"payload"`).
    pub proto_name: String,
    /// Escaped Rust `PascalCase` enum identifier (e.g. `"Payload"`).
    pub enum_ident: String,
    /// True if at least one member variant borrows from `'a`.
    pub has_lifetime: bool,
    /// True if at least one member variant holds a message or group `*View<'a>`.
    pub has_message_variant: bool,
    /// Non-`proto3_optional` member variants in declaration order.
    pub variants: Vec<OneofVariantInfo>,
}

/// Generates `#![no_std]` Rust code for a Protobuf `EnumDescriptorProtoView`.
///
/// Handles standard enums as well as `allow_alias = true` enums with duplicate numeric
/// discriminants by emitting primary variants on the `#[repr(i32)]` `enum` and associated
/// `pub const <AliasVariant>: Self = Self::<PrimaryVariant>;` constants on `impl <EnumName>`.
pub fn generate_enum(
    enum_desc: &EnumDescriptorProtoView<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let raw_name = enum_desc
        .name()
        .map_err(|e| format!("invalid EnumDescriptorProto.name: {e}"))?;
    if raw_name.is_empty() {
        return Err("EnumDescriptorProto.name cannot be empty".to_string());
    }

    let enum_ident = sanitize_type_ident(raw_name);

    let allow_alias = match enum_desc
        .options()
        .map_err(|e| format!("invalid EnumOptions in '{raw_name}': {e}"))?
    {
        Some(opts) => opts
            .allow_alias()
            .map_err(|e| format!("invalid EnumOptions.allow_alias in '{raw_name}': {e}"))?,
        None => false,
    };

    let mut primary_variants: Vec<(String, i32)> = Vec::new();
    let mut alias_variants: Vec<(String, String)> = Vec::new();

    for val_res in enum_desc.value() {
        let val = val_res
            .map_err(|e| format!("invalid EnumValueDescriptorProto in '{raw_name}': {e}"))?;
        let val_name = val
            .name()
            .map_err(|e| format!("invalid EnumValueDescriptorProto.name in '{raw_name}': {e}"))?;
        if val_name.is_empty() {
            return Err(format!(
                "EnumValueDescriptorProto.name cannot be empty in '{raw_name}'"
            ));
        }
        let number = val
            .number()
            .map_err(|e| format!("invalid EnumValueDescriptorProto.number in '{raw_name}': {e}"))?;
        let variant_ident = escape_ident(&to_pascal_case(val_name));

        if let Some((primary_ident, _)) = primary_variants
            .iter()
            .find(|(_, existing_num)| *existing_num == number)
        {
            if !allow_alias {
                return Err(format!(
                    "duplicate enum discriminant {number} for variant '{val_name}' in enum '{raw_name}' without allow_alias = true"
                ));
            }
            if &variant_ident != primary_ident
                && !alias_variants
                    .iter()
                    .any(|(existing_alias, _)| existing_alias == &variant_ident)
            {
                alias_variants.push((variant_ident, primary_ident.clone()));
            }
        } else {
            if primary_variants
                .iter()
                .any(|(existing_ident, _)| existing_ident == &variant_ident)
            {
                return Err(format!(
                    "conflicting PascalCase enum variant identifier '{variant_ident}' in enum '{raw_name}'"
                ));
            }
            primary_variants.push((variant_ident, number));
        }
    }

    let Some((first_variant_ident, _)) = primary_variants.first() else {
        return Err(format!("enum '{raw_name}' must have at least one value"));
    };

    let pad = "    ".repeat(indent);

    writeln!(out, "{pad}#[derive(Clone, Copy, Debug, PartialEq, Eq)]").unwrap();
    writeln!(out, "{pad}#[repr(i32)]").unwrap();
    writeln!(out, "{pad}pub enum {enum_ident} {{").unwrap();
    for (variant_ident, number) in &primary_variants {
        writeln!(out, "{pad}    {variant_ident} = {number},").unwrap();
    }
    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "{pad}impl {enum_ident} {{").unwrap();
    if !alias_variants.is_empty() {
        for (alias_ident, primary_ident) in &alias_variants {
            writeln!(out, "{pad}    #[allow(non_upper_case_globals)]").unwrap();
            writeln!(
                out,
                "{pad}    pub const {alias_ident}: Self = Self::{primary_ident};"
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(out, "{pad}    pub const fn from_i32(val: i32) -> Option<Self> {{").unwrap();
    writeln!(out, "{pad}        match val {{").unwrap();
    for (variant_ident, number) in &primary_variants {
        writeln!(out, "{pad}            {number} => Some(Self::{variant_ident}),").unwrap();
    }
    writeln!(out, "{pad}            _ => None,").unwrap();
    writeln!(out, "{pad}        }}").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(out, "{pad}    pub const fn as_i32(self) -> i32 {{").unwrap();
    writeln!(out, "{pad}        self as i32").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    writeln!(
        out,
        "{pad}impl core::convert::TryFrom<i32> for {enum_ident} {{"
    )
    .unwrap();
    writeln!(out, "{pad}    type Error = i32;").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    fn try_from(val: i32) -> Result<Self, Self::Error> {{"
    )
    .unwrap();
    writeln!(out, "{pad}        Self::from_i32(val).ok_or(val)").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    writeln!(
        out,
        "{pad}impl ::roto::runtime::VarintValue for {enum_ident} {{"
    )
    .unwrap();
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(out, "{pad}    fn from_raw_varint(val: u64) -> Self {{").unwrap();
    writeln!(
        out,
        "{pad}        Self::from_i32(val as i32).unwrap_or(Self::{first_variant_ident})"
    )
    .unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out, "{pad}}}").unwrap();

    Ok(())
}

/// Resolves a single `OneofDescriptorProtoView` at `oneof_index` on `msg_desc`.
///
/// Returns `Ok(None)` if the `oneof` is a proto3 synthetic optional oneof (or has no
/// non-`proto3_optional` member fields).
pub fn resolve_oneof(
    msg_desc: &DescriptorProtoView<'_>,
    oneof_index: i32,
    oneof_desc: &OneofDescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<Option<OneofInfo>, String> {
    let msg_name = msg_desc
        .name()
        .map_err(|e| format!("invalid DescriptorProto.name: {e}"))?;
    let proto_name = oneof_desc
        .name()
        .map_err(|e| format!("invalid OneofDescriptorProto.name in '{msg_name}': {e}"))?;
    if proto_name.is_empty() {
        return Err(format!(
            "OneofDescriptorProto.name cannot be empty in message '{msg_name}'"
        ));
    }

    let enum_ident = sanitize_type_ident(proto_name);
    let mut variants = Vec::new();
    let mut has_lifetime = false;
    let mut has_message_variant = false;

    for field_res in msg_desc.field() {
        let field =
            field_res.map_err(|e| format!("invalid FieldDescriptorProto in '{msg_name}': {e}"))?;
        if !field.has_oneof_index() {
            continue;
        }
        let field_oneof_idx = field
            .oneof_index()
            .map_err(|e| format!("invalid FieldDescriptorProto.oneof_index in '{msg_name}': {e}"))?;
        if field_oneof_idx != oneof_index {
            continue;
        }
        let is_proto3_optional = field.proto3_optional().map_err(|e| {
            format!("invalid FieldDescriptorProto.proto3_optional in '{msg_name}': {e}")
        })?;
        if is_proto3_optional {
            continue;
        }

        let variant = resolve_oneof_variant(&field, msg_name, current_scope, symbols, config)?;
        if variant.borrows_lifetime {
            has_lifetime = true;
        }
        if matches!(
            variant.field_type,
            FieldType::TypeMessage | FieldType::TypeGroup
        ) {
            has_message_variant = true;
        }
        variants.push(variant);
    }

    if variants.is_empty() {
        return Ok(None);
    }

    Ok(Some(OneofInfo {
        oneof_index,
        proto_name: proto_name.to_string(),
        enum_ident,
        has_lifetime,
        has_message_variant,
        variants,
    }))
}

/// Inspects all `oneof_decl` entries on `msg_desc` and returns all real (non-`proto3_optional`)
/// oneofs with their resolved member variants.
pub fn collect_message_oneofs(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<Vec<OneofInfo>, String> {
    let msg_name = msg_desc
        .name()
        .map_err(|e| format!("invalid DescriptorProto.name: {e}"))?;

    let mut oneof_decls = Vec::new();
    for oneof_res in msg_desc.oneof_decl() {
        let oneof =
            oneof_res.map_err(|e| format!("invalid OneofDescriptorProto in '{msg_name}': {e}"))?;
        oneof_decls.push(oneof);
    }

    // Validate that every field's `oneof_index` is in bounds of `oneof_decl`.
    for field_res in msg_desc.field() {
        let field =
            field_res.map_err(|e| format!("invalid FieldDescriptorProto in '{msg_name}': {e}"))?;
        if field.has_oneof_index() {
            let idx = field.oneof_index().map_err(|e| {
                format!("invalid FieldDescriptorProto.oneof_index in '{msg_name}': {e}")
            })?;
            if idx < 0 || (idx as usize) >= oneof_decls.len() {
                let field_name = field.name().unwrap_or("<unnamed>");
                return Err(format!(
                    "field '{field_name}' in message '{msg_name}' has out-of-range oneof_index {idx} (oneof_decl len = {})",
                    oneof_decls.len()
                ));
            }
        }
    }

    let mut real_oneofs = Vec::new();
    for (idx, oneof_desc) in oneof_decls.iter().enumerate() {
        if let Some(info) = resolve_oneof(
            msg_desc,
            idx as i32,
            oneof_desc,
            current_scope,
            symbols,
            config,
        )? {
            real_oneofs.push(info);
        }
    }

    Ok(real_oneofs)
}

/// Emits the Rust `pub enum <OneofPascal><'a>` or `pub enum <OneofPascal>` definition
/// for a resolved [`OneofInfo`].
pub fn generate_oneof(oneof: &OneofInfo, indent: usize, out: &mut String) {
    let pad = "    ".repeat(indent);

    if oneof.has_message_variant {
        writeln!(out, "{pad}#[derive(Clone, Copy, Debug)]").unwrap();
    } else {
        writeln!(out, "{pad}#[derive(Clone, Copy, Debug, PartialEq)]").unwrap();
    }

    if oneof.has_lifetime {
        writeln!(out, "{pad}pub enum {}<'a> {{", oneof.enum_ident).unwrap();
    } else {
        writeln!(out, "{pad}pub enum {} {{", oneof.enum_ident).unwrap();
    }

    for variant in &oneof.variants {
        writeln!(
            out,
            "{pad}    {}({}),",
            variant.variant_ident, variant.rust_type
        )
        .unwrap();
    }

    writeln!(out, "{pad}}}").unwrap();
}

/// Inspects and generates all real (non-`proto3_optional`) oneof enums declared on `msg_desc`.
pub fn generate_message_oneofs(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let oneofs = collect_message_oneofs(msg_desc, current_scope, symbols, config)?;
    for (i, oneof) in oneofs.iter().enumerate() {
        if i > 0 {
            writeln!(out).unwrap();
        }
        generate_oneof(oneof, indent, out);
    }
    Ok(())
}

fn resolve_oneof_variant(
    field: &FieldDescriptorProtoView<'_>,
    msg_name: &str,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<OneofVariantInfo, String> {
    let field_name = field
        .name()
        .map_err(|e| format!("invalid FieldDescriptorProto.name in '{msg_name}': {e}"))?;
    if field_name.is_empty() {
        return Err(format!(
            "FieldDescriptorProto.name cannot be empty in message '{msg_name}'"
        ));
    }
    let field_number = field
        .number()
        .map_err(|e| format!("invalid FieldDescriptorProto.number in '{field_name}': {e}"))?;
    let label = field
        .label()
        .map_err(|e| format!("invalid FieldDescriptorProto.label in '{field_name}': {e}"))?;
    if label == FieldLabel::LabelRepeated {
        return Err(format!(
            "oneof field '{field_name}' in message '{msg_name}' cannot be repeated"
        ));
    }

    let field_type = field
        .r#type()
        .map_err(|e| format!("invalid FieldDescriptorProto.type in '{field_name}': {e}"))?;

    let (rust_type, borrows_lifetime) = match field_type {
        FieldType::TypeInt32 | FieldType::TypeSint32 | FieldType::TypeSfixed32 => {
            ("i32".to_string(), false)
        }
        FieldType::TypeInt64 | FieldType::TypeSint64 | FieldType::TypeSfixed64 => {
            ("i64".to_string(), false)
        }
        FieldType::TypeUint32 | FieldType::TypeFixed32 => ("u32".to_string(), false),
        FieldType::TypeUint64 | FieldType::TypeFixed64 => ("u64".to_string(), false),
        FieldType::TypeFloat => ("f32".to_string(), false),
        FieldType::TypeDouble => ("f64".to_string(), false),
        FieldType::TypeBool => ("bool".to_string(), false),
        FieldType::TypeString => ("&'a str".to_string(), true),
        FieldType::TypeBytes => ("&'a [u8]".to_string(), true),
        FieldType::TypeEnum => {
            let resolved = symbols
                .resolve_field(current_scope, field, config)?
                .ok_or_else(|| {
                    format!("missing type_name on enum field '{field_name}' in '{current_scope}'")
                })?;
            if !resolved.is_enum() {
                return Err(format!(
                    "field '{field_name}' in '{current_scope}' has TypeEnum but resolves to non-enum '{}'",
                    resolved.fq_proto_name
                ));
            }
            (resolved.enum_path().to_string(), false)
        }
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = symbols
                .resolve_field(current_scope, field, config)?
                .ok_or_else(|| {
                    format!(
                        "missing type_name on message field '{field_name}' in '{current_scope}'"
                    )
                })?;
            if !resolved.is_message() {
                return Err(format!(
                    "field '{field_name}' in '{current_scope}' has {field_type:?} but resolves to non-message '{}'",
                    resolved.fq_proto_name
                ));
            }
            (resolved.view_type("'a"), true)
        }
    };

    let variant_ident = escape_ident(&to_pascal_case(field_name));

    Ok(OneofVariantInfo {
        proto_field_name: field_name.to_string(),
        variant_ident,
        field_number,
        field_type,
        rust_type,
        borrows_lifetime,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    use google_protobuf_roto::google::protobuf::{
        EnumDescriptorProtoBuilder, FileDescriptorProtoBuilder, FileDescriptorProtoView,
        FileDescriptorSetView,
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
    fn generates_standard_enums_from_real_descriptor_proto() {
        let fds_bytes = load_descriptor_fds();
        let fds = FileDescriptorSetView::from_bytes(&fds_bytes).unwrap();
        let desc_file = fds
            .file()
            .map(|f| f.unwrap())
            .find(|f| f.name() == Ok("google/protobuf/descriptor.proto"))
            .unwrap();

        // 1. Top-level Edition enum
        let edition_enum = desc_file
            .enum_type()
            .map(|e| e.unwrap())
            .find(|e| e.name() == Ok("Edition"))
            .unwrap();
        let mut out = String::new();
        generate_enum(&edition_enum, 0, &mut out).unwrap();

        assert!(out.contains("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n#[repr(i32)]\npub enum Edition {"));
        assert!(out.contains("    EditionUnknown = 0,"));
        assert!(out.contains("    EditionProto2 = 998,"));
        assert!(out.contains("    EditionProto3 = 999,"));
        assert!(out.contains("    Edition2023 = 1000,"));
        assert!(out.contains("    EditionMax = 2147483647,"));
        assert!(out.contains("            0 => Some(Self::EditionUnknown),"));
        assert!(out.contains("            1000 => Some(Self::Edition2023),"));
        assert!(out.contains("impl core::convert::TryFrom<i32> for Edition {"));
        assert!(out.contains("impl ::roto::runtime::VarintValue for Edition {"));
        assert!(out.contains("Self::from_i32(val as i32).unwrap_or(Self::EditionUnknown)"));

        // 2. Nested FieldDescriptorProto.Type and FieldDescriptorProto.Label at indent = 1
        let fdp_msg = desc_file
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("FieldDescriptorProto"))
            .unwrap();
        let label_enum = fdp_msg
            .enum_type()
            .map(|e| e.unwrap())
            .find(|e| e.name() == Ok("Label"))
            .unwrap();
        let mut label_out = String::new();
        generate_enum(&label_enum, 1, &mut label_out).unwrap();

        let expected_label = concat!(
            "    #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n",
            "    #[repr(i32)]\n",
            "    pub enum Label {\n",
            "        LabelOptional = 1,\n",
            "        LabelRepeated = 3,\n",
            "        LabelRequired = 2,\n",
            "    }\n",
            "\n",
            "    impl Label {\n",
            "        #[inline]\n",
            "        pub const fn from_i32(val: i32) -> Option<Self> {\n",
            "            match val {\n",
            "                1 => Some(Self::LabelOptional),\n",
            "                3 => Some(Self::LabelRepeated),\n",
            "                2 => Some(Self::LabelRequired),\n",
            "                _ => None,\n",
            "            }\n",
            "        }\n",
            "\n",
            "        #[inline]\n",
            "        pub const fn as_i32(self) -> i32 {\n",
            "            self as i32\n",
            "        }\n",
            "    }\n",
            "\n",
            "    impl core::convert::TryFrom<i32> for Label {\n",
            "        type Error = i32;\n",
            "\n",
            "        #[inline]\n",
            "        fn try_from(val: i32) -> Result<Self, Self::Error> {\n",
            "            Self::from_i32(val).ok_or(val)\n",
            "        }\n",
            "    }\n",
            "\n",
            "    impl ::roto::runtime::VarintValue for Label {\n",
            "        #[inline]\n",
            "        fn from_raw_varint(val: u64) -> Self {\n",
            "            Self::from_i32(val as i32).unwrap_or(Self::LabelOptional)\n",
            "        }\n",
            "    }\n",
        );
        assert_eq!(label_out, expected_label);

        let type_enum = fdp_msg
            .enum_type()
            .map(|e| e.unwrap())
            .find(|e| e.name() == Ok("Type"))
            .unwrap();
        let mut type_out = String::new();
        generate_enum(&type_enum, 0, &mut type_out).unwrap();
        assert!(type_out.contains("pub enum Type {"));
        assert!(type_out.contains("    TypeDouble = 1,"));
        assert!(type_out.contains("    TypeSint64 = 18,"));
        assert!(type_out.contains("Self::from_i32(val as i32).unwrap_or(Self::TypeDouble)"));
    }

    #[test]
    fn generates_allow_alias_enum_and_rejects_unaliased_duplicates() {
        let mut buf = [0u8; 512];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut eb = EnumDescriptorProtoBuilder::new(&mut writer);
            eb.write_options(|opt| {
                opt.write_allow_alias(true);
            })
            .write_value(|v| {
                v.write_number(2).write_name("MODE_STANDBY");
            })
            .write_value(|v| {
                v.write_number(2).write_name("MODE_SLEEP");
            })
            .write_value(|v| {
                v.write_number(1).write_name("MODE_RUNNING");
            })
            .write_value(|v| {
                v.write_number(1).write_name("MODE_ACTIVE");
            })
            .write_value(|v| {
                v.write_number(0).write_name("MODE_IDLE");
            })
            .write_name("DeviceMode");
        }
        let enum_view = EnumDescriptorProtoView::from_bytes(writer.finish().unwrap()).unwrap();

        let mut out = String::new();
        generate_enum(&enum_view, 0, &mut out).unwrap();

        let expected = "\
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DeviceMode {
    ModeIdle = 0,
    ModeActive = 1,
    ModeSleep = 2,
}

impl DeviceMode {
    #[allow(non_upper_case_globals)]
    pub const ModeRunning: Self = Self::ModeActive;
    #[allow(non_upper_case_globals)]
    pub const ModeStandby: Self = Self::ModeSleep;

    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::ModeIdle),
            1 => Some(Self::ModeActive),
            2 => Some(Self::ModeSleep),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for DeviceMode {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for DeviceMode {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::ModeIdle)
    }
}
";
        assert_eq!(out, expected);

        // Without allow_alias = true, duplicate numeric discriminants must return an error
        let mut bad_buf = [0u8; 256];
        let mut bad_writer = BackwardWriter::new(&mut bad_buf);
        {
            let mut eb = EnumDescriptorProtoBuilder::new(&mut bad_writer);
            eb.write_value(|v| {
                v.write_number(1).write_name("SECOND");
            })
            .write_value(|v| {
                v.write_number(1).write_name("FIRST");
            })
            .write_name("DuplicateWithoutAlias");
        }
        let bad_view = EnumDescriptorProtoView::from_bytes(bad_writer.finish().unwrap()).unwrap();
        let mut bad_out = String::new();
        let err = generate_enum(&bad_view, 0, &mut bad_out).unwrap_err();
        assert!(err.contains("duplicate enum discriminant 1"));
    }

    #[test]
    fn generates_borrowed_and_scalar_oneofs_and_filters_proto3_optional() {
        let mut buf = [0u8; 2048];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut fb = FileDescriptorProtoBuilder::new(&mut writer);
            fb.write_message_type(|msg| {
                // Oneof 2: synthetic proto3_optional oneof `_opt_tag` (must be filtered out)
                msg.write_oneof_decl(|oo| {
                    oo.write_name("_opt_tag");
                })
                // Oneof 1: scalar/enum-only oneof `alert_threshold` (no <'a> lifetime)
                .write_oneof_decl(|oo| {
                    oo.write_name("alert_threshold");
                })
                // Oneof 0: borrowed oneof `sensor_payload` (requires <'a> lifetime)
                .write_oneof_decl(|oo| {
                    oo.write_name("sensor_payload");
                })
                // Synthetic proto3_optional field in oneof 2
                .write_field(|f| {
                    f.write_proto3_optional(true)
                        .write_oneof_index(2)
                        .write_type(FieldType::TypeUint32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(10)
                        .write_name("opt_tag");
                })
                // Scalar/enum-only fields in oneof 1
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type_name(".demo.DeviceMode")
                        .write_type(FieldType::TypeEnum)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(9)
                        .write_name("target_mode");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeBool)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(8)
                        .write_name("enabled");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeDouble)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(7)
                        .write_name("max_ratio");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeFloat)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(6)
                        .write_name("max_temp_c");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeSint64)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(5)
                        .write_name("delta_ns");
                })
                .write_field(|f| {
                    f.write_oneof_index(1)
                        .write_type(FieldType::TypeFixed32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(4)
                        .write_name("max_count");
                })
                // Borrowed + scalar fields in oneof 0
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type_name(".demo.Location")
                        .write_type(FieldType::TypeMessage)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(3)
                        .write_name("location");
                })
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type(FieldType::TypeBytes)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(2)
                        .write_name("raw_bytes");
                })
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type(FieldType::TypeString)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(1)
                        .write_name("text");
                })
                .write_name("TelemetryEvent");
            })
            .write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeInt32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(1)
                        .write_name("lat_e7");
                })
                .write_name("Location");
            })
            .write_enum_type(|en| {
                en.write_value(|v| {
                    v.write_number(0).write_name("MODE_IDLE");
                })
                .write_name("DeviceMode");
            })
            .write_syntax("proto3")
            .write_package("demo")
            .write_name("telemetry.proto");
        }

        let file_view = FileDescriptorProtoView::from_bytes(writer.finish().unwrap()).unwrap();
        let symbols = SymbolTable::from_files([file_view]).unwrap();
        let config = GeneratorConfig::default();

        let event_msg = file_view
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("TelemetryEvent"))
            .unwrap();

        let oneofs =
            collect_message_oneofs(&event_msg, ".demo.TelemetryEvent", &symbols, &config).unwrap();
        assert_eq!(oneofs.len(), 2);
        assert_eq!(oneofs[0].enum_ident, "SensorPayload");
        assert!(oneofs[0].has_lifetime);
        assert!(oneofs[0].has_message_variant);
        assert_eq!(oneofs[1].enum_ident, "AlertThreshold");
        assert!(!oneofs[1].has_lifetime);
        assert!(!oneofs[1].has_message_variant);

        let mut out = String::new();
        generate_message_oneofs(
            &event_msg,
            ".demo.TelemetryEvent",
            &symbols,
            &config,
            0,
            &mut out,
        )
        .unwrap();

        let expected = "\
#[derive(Clone, Copy, Debug)]
pub enum SensorPayload<'a> {
    Text(&'a str),
    RawBytes(&'a [u8]),
    Location(crate::demo::LocationView<'a>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertThreshold {
    MaxCount(u32),
    DeltaNs(i64),
    MaxTempC(f32),
    MaxRatio(f64),
    Enabled(bool),
    TargetMode(crate::demo::DeviceMode),
}
";
        assert_eq!(out, expected);
    }
}
