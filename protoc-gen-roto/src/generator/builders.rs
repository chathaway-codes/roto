use std::collections::BTreeSet;
use std::fmt::Write as _;

use google_protobuf_roto::google::protobuf::{
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    DescriptorProtoView, FieldDescriptorProtoView,
};

use super::config::GeneratorConfig;
use super::ident::{sanitize_type_ident, to_snake_case};
use super::symbols::SymbolTable;

/// Generates `#![no_std]` single-pass backward `<Msg>Builder<'a, 'b>` struct and
/// `impl<'a, 'b> <Msg>Builder<'a, 'b>` methods for a Protobuf `DescriptorProtoView`.
pub fn generate_builder(
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

    let msg_ident = sanitize_type_ident(raw_msg_name);
    let builder_ident = format!("{msg_ident}Builder");
    let view_ident = format!("{msg_ident}View");
    let effective_scope = compute_message_scope(current_scope, raw_msg_name, symbols);

    let mut fields: Vec<FieldDescriptorProtoView<'_>> = Vec::new();
    let mut field_names: BTreeSet<String> = BTreeSet::new();

    for field_res in msg_desc.field() {
        let field = field_res
            .map_err(|e| format!("invalid FieldDescriptorProto in '{raw_msg_name}': {e}"))?;
        let raw_field_name = field
            .name()
            .map_err(|e| format!("invalid FieldDescriptorProto.name in '{raw_msg_name}': {e}"))?;
        if raw_field_name.is_empty() {
            return Err(format!(
                "FieldDescriptorProto.name cannot be empty in message '{raw_msg_name}'"
            ));
        }
        let field_snake = to_snake_case(raw_field_name);
        if !field_names.insert(field_snake.clone()) {
            return Err(format!(
                "duplicate field name '{raw_field_name}' in message '{raw_msg_name}'"
            ));
        }
        fields.push(field);
    }

    let mut emitted_aliases: BTreeSet<String> = BTreeSet::new();
    let pad = "    ".repeat(indent);

    writeln!(out, "{pad}pub struct {builder_ident}<'a, 'b> {{").unwrap();
    writeln!(
        out,
        "{pad}    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,"
    )
    .unwrap();
    writeln!(out, "{pad}}}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "{pad}impl<'a, 'b> {builder_ident}<'a, 'b> {{").unwrap();
    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "{pad}        Self {{ writer }}").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {{"
    )
    .unwrap();
    writeln!(out, "{pad}        self.writer.check()").unwrap();
    writeln!(out, "{pad}    }}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "{pad}    #[inline]").unwrap();
    writeln!(
        out,
        "{pad}    pub fn copy_unknown_fields(&mut self, view: &{view_ident}<'_>) -> &mut Self {{"
    )
    .unwrap();
    writeln!(out, "{pad}        for raw_field in view.unknown_fields() {{").unwrap();
    writeln!(out, "{pad}            let _ = self.writer.write_bytes(raw_field);").unwrap();
    writeln!(out, "{pad}        }}").unwrap();
    writeln!(out, "{pad}        self").unwrap();
    writeln!(out, "{pad}    }}").unwrap();

    for field in &fields {
        let raw_field_name = field
            .name()
            .map_err(|e| format!("invalid FieldDescriptorProto.name in '{raw_msg_name}': {e}"))?;
        let field_snake = to_snake_case(raw_field_name);
        let number = field.number().map_err(|e| {
            format!("invalid FieldDescriptorProto.number in '{raw_msg_name}.{raw_field_name}': {e}")
        })?;
        if number <= 0 {
            return Err(format!(
                "invalid non-positive field number {number} on '{raw_msg_name}.{raw_field_name}'"
            ));
        }
        let tag = number as u32;

        let label = field.label().map_err(|e| {
            format!("invalid FieldDescriptorProto.label in '{raw_msg_name}.{raw_field_name}': {e}")
        })?;
        let field_type = field.r#type().map_err(|e| {
            format!("invalid FieldDescriptorProto.type in '{raw_msg_name}.{raw_field_name}': {e}")
        })?;

        let writer_spec = resolve_field_writer_spec(
            field,
            field_type,
            raw_msg_name,
            raw_field_name,
            &effective_scope,
            symbols,
            config,
        )?;

        writeln!(out).unwrap();
        emit_single_writer(out, &pad, &field_snake, tag, &writer_spec);

        if label == FieldLabel::LabelRepeated {
            if let Some(singular) = singularize_repeated_field_name(&field_snake) {
                if !field_names.contains(singular)
                    && emitted_aliases.insert(singular.to_string())
                {
                    writeln!(out).unwrap();
                    emit_singular_alias(out, &pad, singular, &field_snake, &writer_spec);
                }
            }

            if writer_spec.is_packable() {
                writeln!(out).unwrap();
                emit_packed_writer(out, &pad, &field_snake, tag, &writer_spec);
            }
        }
    }

    writeln!(out, "{pad}}}").unwrap();

    Ok(())
}

/// Returns `Some(singular)` if `name` is a plural repeated field name ending in `'s'`
/// (with `name.len() > 1 && !name.ends_with("ss")`).
pub fn singularize_repeated_field_name(name: &str) -> Option<&str> {
    if name.len() > 1 && name.ends_with('s') && !name.ends_with("ss") {
        Some(&name[..name.len() - 1])
    } else {
        None
    }
}

enum FieldWriterSpec {
    Scalar {
        param_type: String,
        write_expr: &'static str,
        wire_type: &'static str,
        packable: bool,
    },
    NestedMessage {
        sub_builder_path: String,
    },
}

impl FieldWriterSpec {
    #[inline]
    fn is_packable(&self) -> bool {
        match self {
            Self::Scalar { packable, .. } => *packable,
            Self::NestedMessage { .. } => false,
        }
    }
}

fn resolve_field_writer_spec(
    field: &FieldDescriptorProtoView<'_>,
    field_type: FieldType,
    msg_name: &str,
    field_name: &str,
    effective_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
) -> Result<FieldWriterSpec, String> {
    let spec = match field_type {
        FieldType::TypeInt32 => FieldWriterSpec::Scalar {
            param_type: "i32".to_string(),
            write_expr: "let _ = self.writer.write_varint(val as u64);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeInt64 => FieldWriterSpec::Scalar {
            param_type: "i64".to_string(),
            write_expr: "let _ = self.writer.write_varint(val as u64);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeUint32 => FieldWriterSpec::Scalar {
            param_type: "u32".to_string(),
            write_expr: "let _ = self.writer.write_varint(val as u64);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeUint64 => FieldWriterSpec::Scalar {
            param_type: "u64".to_string(),
            write_expr: "let _ = self.writer.write_varint(val);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeBool => FieldWriterSpec::Scalar {
            param_type: "bool".to_string(),
            write_expr: "let _ = self.writer.write_bool(val);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeEnum => {
            let resolved = symbols
                .resolve_field(effective_scope, field, config)?
                .ok_or_else(|| {
                    format!(
                        "missing type_name on enum field '{field_name}' in message '{msg_name}'"
                    )
                })?;
            if !resolved.is_enum() {
                return Err(format!(
                    "field '{field_name}' in message '{msg_name}' has TypeEnum but resolves to non-enum '{}'",
                    resolved.fq_proto_name
                ));
            }
            FieldWriterSpec::Scalar {
                param_type: resolved.enum_path().to_string(),
                write_expr: "let _ = self.writer.write_varint(val.as_i32() as u64);",
                wire_type: "::roto::runtime::WireType::Varint",
                packable: true,
            }
        }
        FieldType::TypeSint32 => FieldWriterSpec::Scalar {
            param_type: "i32".to_string(),
            write_expr: "let _ = self.writer.write_zigzag32(val);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeSint64 => FieldWriterSpec::Scalar {
            param_type: "i64".to_string(),
            write_expr: "let _ = self.writer.write_zigzag64(val);",
            wire_type: "::roto::runtime::WireType::Varint",
            packable: true,
        },
        FieldType::TypeFixed32 => FieldWriterSpec::Scalar {
            param_type: "u32".to_string(),
            write_expr: "let _ = self.writer.write_fixed32(val);",
            wire_type: "::roto::runtime::WireType::Fixed32",
            packable: true,
        },
        FieldType::TypeSfixed32 => FieldWriterSpec::Scalar {
            param_type: "i32".to_string(),
            write_expr: "let _ = self.writer.write_sfixed32(val);",
            wire_type: "::roto::runtime::WireType::Fixed32",
            packable: true,
        },
        FieldType::TypeFloat => FieldWriterSpec::Scalar {
            param_type: "f32".to_string(),
            write_expr: "let _ = self.writer.write_f32(val);",
            wire_type: "::roto::runtime::WireType::Fixed32",
            packable: true,
        },
        FieldType::TypeFixed64 => FieldWriterSpec::Scalar {
            param_type: "u64".to_string(),
            write_expr: "let _ = self.writer.write_fixed64(val);",
            wire_type: "::roto::runtime::WireType::Fixed64",
            packable: true,
        },
        FieldType::TypeSfixed64 => FieldWriterSpec::Scalar {
            param_type: "i64".to_string(),
            write_expr: "let _ = self.writer.write_sfixed64(val);",
            wire_type: "::roto::runtime::WireType::Fixed64",
            packable: true,
        },
        FieldType::TypeDouble => FieldWriterSpec::Scalar {
            param_type: "f64".to_string(),
            write_expr: "let _ = self.writer.write_f64(val);",
            wire_type: "::roto::runtime::WireType::Fixed64",
            packable: true,
        },
        FieldType::TypeString => FieldWriterSpec::Scalar {
            param_type: "&str".to_string(),
            write_expr: "let _ = self.writer.write_length_delimited(val.as_bytes());",
            wire_type: "::roto::runtime::WireType::LengthDelimited",
            packable: false,
        },
        FieldType::TypeBytes => FieldWriterSpec::Scalar {
            param_type: "&[u8]".to_string(),
            write_expr: "let _ = self.writer.write_length_delimited(val);",
            wire_type: "::roto::runtime::WireType::LengthDelimited",
            packable: false,
        },
        FieldType::TypeMessage | FieldType::TypeGroup => {
            let resolved = symbols
                .resolve_field(effective_scope, field, config)?
                .ok_or_else(|| {
                    format!(
                        "missing type_name on message field '{field_name}' in message '{msg_name}'"
                    )
                })?;
            if !resolved.is_message() {
                return Err(format!(
                    "field '{field_name}' in message '{msg_name}' has {field_type:?} but resolves to non-message '{}'",
                    resolved.fq_proto_name
                ));
            }
            FieldWriterSpec::NestedMessage {
                sub_builder_path: resolved.builder_ident_path,
            }
        }
    };

    Ok(spec)
}

fn emit_single_writer(
    out: &mut String,
    pad: &str,
    field_snake: &str,
    tag: u32,
    spec: &FieldWriterSpec,
) {
    match spec {
        FieldWriterSpec::Scalar {
            param_type,
            write_expr,
            wire_type,
            ..
        } => {
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub fn write_{field_snake}(&mut self, val: {param_type}) -> &mut Self {{"
            )
            .unwrap();
            writeln!(out, "{pad}        {write_expr}").unwrap();
            writeln!(
                out,
                "{pad}        let _ = self.writer.write_tag({tag}, {wire_type});"
            )
            .unwrap();
            writeln!(out, "{pad}        self").unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
        FieldWriterSpec::NestedMessage { sub_builder_path } => {
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub fn write_{field_snake}<F>(&mut self, f: F) -> &mut Self"
            )
            .unwrap();
            writeln!(out, "{pad}    where").unwrap();
            writeln!(
                out,
                "{pad}        F: FnOnce(&mut {sub_builder_path}<'_, 'b>),"
            )
            .unwrap();
            writeln!(out, "{pad}    {{").unwrap();
            writeln!(out, "{pad}        let end_pos = self.writer.cursor();").unwrap();
            writeln!(out, "{pad}        {{").unwrap();
            writeln!(
                out,
                "{pad}            let mut sub_builder = {sub_builder_path}::new(self.writer);"
            )
            .unwrap();
            writeln!(out, "{pad}            f(&mut sub_builder);").unwrap();
            writeln!(out, "{pad}        }}").unwrap();
            writeln!(
                out,
                "{pad}        let nested_len = end_pos - self.writer.cursor();"
            )
            .unwrap();
            writeln!(
                out,
                "{pad}        let _ = self.writer.write_varint(nested_len as u64);"
            )
            .unwrap();
            writeln!(
                out,
                "{pad}        let _ = self.writer.write_tag({tag}, ::roto::runtime::WireType::LengthDelimited);"
            )
            .unwrap();
            writeln!(out, "{pad}        self").unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
    }
}

fn emit_singular_alias(
    out: &mut String,
    pad: &str,
    singular: &str,
    field_snake: &str,
    spec: &FieldWriterSpec,
) {
    match spec {
        FieldWriterSpec::Scalar { param_type, .. } => {
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub fn write_{singular}(&mut self, val: {param_type}) -> &mut Self {{"
            )
            .unwrap();
            writeln!(out, "{pad}        self.write_{field_snake}(val)").unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
        FieldWriterSpec::NestedMessage { sub_builder_path } => {
            writeln!(out, "{pad}    #[inline]").unwrap();
            writeln!(
                out,
                "{pad}    pub fn write_{singular}<F>(&mut self, f: F) -> &mut Self"
            )
            .unwrap();
            writeln!(out, "{pad}    where").unwrap();
            writeln!(
                out,
                "{pad}        F: FnOnce(&mut {sub_builder_path}<'_, 'b>),"
            )
            .unwrap();
            writeln!(out, "{pad}    {{").unwrap();
            writeln!(out, "{pad}        self.write_{field_snake}(f)").unwrap();
            writeln!(out, "{pad}    }}").unwrap();
        }
    }
}

fn emit_packed_writer(
    out: &mut String,
    pad: &str,
    field_snake: &str,
    tag: u32,
    spec: &FieldWriterSpec,
) {
    if let FieldWriterSpec::Scalar {
        param_type,
        write_expr,
        packable: true,
        ..
    } = spec
    {
        writeln!(out, "{pad}    #[inline]").unwrap();
        writeln!(
            out,
            "{pad}    pub fn write_packed_{field_snake}(&mut self, vals: &[{param_type}]) -> &mut Self {{"
        )
        .unwrap();
        writeln!(out, "{pad}        let end_pos = self.writer.cursor();").unwrap();
        writeln!(out, "{pad}        for &val in vals.iter().rev() {{").unwrap();
        writeln!(out, "{pad}            {write_expr}").unwrap();
        writeln!(out, "{pad}        }}").unwrap();
        writeln!(
            out,
            "{pad}        let packed_len = end_pos - self.writer.cursor();"
        )
        .unwrap();
        writeln!(
            out,
            "{pad}        let _ = self.writer.write_varint(packed_len as u64);"
        )
        .unwrap();
        writeln!(
            out,
            "{pad}        let _ = self.writer.write_tag({tag}, ::roto::runtime::WireType::LengthDelimited);"
        )
        .unwrap();
        writeln!(out, "{pad}        self").unwrap();
        writeln!(out, "{pad}    }}").unwrap();
    }
}

fn compute_message_scope(current_scope: &str, msg_name: &str, symbols: &SymbolTable) -> String {
    let trimmed = current_scope.trim().trim_matches('.');
    if trimmed.is_empty() {
        return format!(".{msg_name}");
    }
    let candidate = format!(".{trimmed}.{msg_name}");
    if symbols.get(&candidate).is_some() {
        return candidate;
    }
    let last_seg = trimmed.rsplit('.').next().unwrap_or(trimmed);
    if last_seg == msg_name {
        format!(".{trimmed}")
    } else {
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    use google_protobuf_roto::google::protobuf::{
        FileDescriptorProtoBuilder, FileDescriptorProtoView, FileDescriptorSetView,
    };
    use roto::runtime::BackwardWriter;

    fn load_descriptor_and_plugin_fds() -> Vec<u8> {
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
    fn generates_readme_location_and_sensor_reading_builders() {
        let mut buf = [0u8; 1024];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut fb = FileDescriptorProtoBuilder::new(&mut writer);
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

        let loc_msg = file_view
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("Location"))
            .unwrap();
        let mut loc_out = String::new();
        generate_builder(&loc_msg, ".telemetry", &symbols, &config, 0, &mut loc_out).unwrap();

        let expected_loc = "\
pub struct LocationBuilder<'a, 'b> {
    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
}

impl<'a, 'b> LocationBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
        for raw_field in view.unknown_fields() {
            let _ = self.writer.write_bytes(raw_field);
        }
        self
    }

    #[inline]
    pub fn write_lat_e7(&mut self, val: i32) -> &mut Self {
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_lon_e7(&mut self, val: i32) -> &mut Self {
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }
}
";
        assert_eq!(loc_out, expected_loc);

        let sr_msg = file_view
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("SensorReading"))
            .unwrap();
        let mut sr_out = String::new();
        generate_builder(&sr_msg, ".telemetry", &symbols, &config, 0, &mut sr_out).unwrap();

        let expected_sr = "\
pub struct SensorReadingBuilder<'a, 'b> {
    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
}

impl<'a, 'b> SensorReadingBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
        self.writer.check()
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &SensorReadingView<'_>) -> &mut Self {
        for raw_field in view.unknown_fields() {
            let _ = self.writer.write_bytes(raw_field);
        }
        self
    }

    #[inline]
    pub fn write_timestamp_ms(&mut self, val: u64) -> &mut Self {
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_device_id(&mut self, val: &str) -> &mut Self {
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::telemetry::LocationBuilder<'_, 'b>),
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = crate::telemetry::LocationBuilder::new(self.writer);
            f(&mut sub_builder);
        }
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_samples(&mut self, val: u32) -> &mut Self {
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_sample(&mut self, val: u32) -> &mut Self {
        self.write_samples(val)
    }

    #[inline]
    pub fn write_packed_samples(&mut self, vals: &[u32]) -> &mut Self {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}
";
        assert_eq!(sr_out, expected_sr);
    }

    #[test]
    fn generates_all_18_field_types_packed_writers_and_singular_aliases() {
        let mut buf = [0u8; 4096];
        let mut writer = BackwardWriter::new(&mut buf);
        {
            let mut fb = FileDescriptorProtoBuilder::new(&mut writer);
            fb.write_message_type(|msg| {
                // Collision check: `items` is repeated, but `item` already exists as field 20,
                // and `status` ends in "ss" (`address`), so neither gets a conflicting alias.
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeString)
                        .write_label(FieldLabel::LabelRepeated)
                        .write_number(22)
                        .write_name("address");
                })
                .write_field(|f| {
                    f.write_type(FieldType::TypeUint32)
                        .write_label(FieldLabel::LabelRepeated)
                        .write_number(21)
                        .write_name("items");
                })
                .write_field(|f| {
                    f.write_type(FieldType::TypeUint32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(20)
                        .write_name("item");
                })
                // Oneof field (field 19)
                .write_oneof_decl(|oo| {
                    oo.write_name("choice");
                })
                .write_field(|f| {
                    f.write_oneof_index(0)
                        .write_type(FieldType::TypeString)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(19)
                        .write_name("oneof_str");
                });

                // All 18 field types as repeated to verify both single and packed writers
                let all_types = [
                    (1, "f_double", FieldType::TypeDouble, ""),
                    (2, "f_float", FieldType::TypeFloat, ""),
                    (3, "f_int64", FieldType::TypeInt64, ""),
                    (4, "f_uint64", FieldType::TypeUint64, ""),
                    (5, "f_int32", FieldType::TypeInt32, ""),
                    (6, "f_fixed64", FieldType::TypeFixed64, ""),
                    (7, "f_fixed32", FieldType::TypeFixed32, ""),
                    (8, "f_bool", FieldType::TypeBool, ""),
                    (9, "f_string", FieldType::TypeString, ""),
                    (10, "f_group", FieldType::TypeGroup, ".pkg.SubGroup"),
                    (11, "f_messages", FieldType::TypeMessage, ".pkg.SubMsg"),
                    (12, "f_bytes", FieldType::TypeBytes, ""),
                    (13, "f_uint32", FieldType::TypeUint32, ""),
                    (14, "f_enum", FieldType::TypeEnum, ".pkg.Mode"),
                    (15, "f_sfixed32", FieldType::TypeSfixed32, ""),
                    (16, "f_sfixed64", FieldType::TypeSfixed64, ""),
                    (17, "f_sint32", FieldType::TypeSint32, ""),
                    (18, "f_sint64", FieldType::TypeSint64, ""),
                ];
                for &(num, name, ty, type_name) in all_types.iter().rev() {
                    msg.write_field(|f| {
                        if !type_name.is_empty() {
                            f.write_type_name(type_name);
                        }
                        f.write_type(ty)
                            .write_label(FieldLabel::LabelRepeated)
                            .write_number(num)
                            .write_name(name);
                    });
                }
                msg.write_name("AllTypes");
            })
            .write_message_type(|msg| {
                msg.write_name("SubMsg");
            })
            .write_message_type(|msg| {
                msg.write_name("SubGroup");
            })
            .write_enum_type(|en| {
                en.write_value(|v| {
                    v.write_number(0).write_name("MODE_A");
                })
                .write_name("Mode");
            })
            .write_syntax("proto2")
            .write_package("pkg")
            .write_name("all_types.proto");
        }

        let file_view = FileDescriptorProtoView::from_bytes(writer.finish().unwrap()).unwrap();
        let symbols = SymbolTable::from_files([file_view]).unwrap();
        let config = GeneratorConfig::default();

        let all_msg = file_view
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("AllTypes"))
            .unwrap();
        let mut out = String::new();
        generate_builder(&all_msg, ".pkg.AllTypes", &symbols, &config, 0, &mut out).unwrap();

        // Check all 18 single writers
        assert!(out.contains("pub fn write_f_double(&mut self, val: f64) -> &mut Self {\n        let _ = self.writer.write_f64(val);\n        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Fixed64);\n        self"));
        assert!(out.contains("pub fn write_f_float(&mut self, val: f32) -> &mut Self {\n        let _ = self.writer.write_f32(val);\n        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Fixed32);\n        self"));
        assert!(out.contains("pub fn write_f_int64(&mut self, val: i64) -> &mut Self {\n        let _ = self.writer.write_varint(val as u64);\n        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_uint64(&mut self, val: u64) -> &mut Self {\n        let _ = self.writer.write_varint(val);\n        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_int32(&mut self, val: i32) -> &mut Self {\n        let _ = self.writer.write_varint(val as u64);\n        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_fixed64(&mut self, val: u64) -> &mut Self {\n        let _ = self.writer.write_fixed64(val);\n        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Fixed64);\n        self"));
        assert!(out.contains("pub fn write_f_fixed32(&mut self, val: u32) -> &mut Self {\n        let _ = self.writer.write_fixed32(val);\n        let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Fixed32);\n        self"));
        assert!(out.contains("pub fn write_f_bool(&mut self, val: bool) -> &mut Self {\n        let _ = self.writer.write_bool(val);\n        let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_string(&mut self, val: &str) -> &mut Self {\n        let _ = self.writer.write_length_delimited(val.as_bytes());\n        let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);\n        self"));
        assert!(out.contains("pub fn write_f_group<F>(&mut self, f: F) -> &mut Self\n    where\n        F: FnOnce(&mut crate::pkg::SubGroupBuilder<'_, 'b>),"));
        assert!(out.contains("pub fn write_f_messages<F>(&mut self, f: F) -> &mut Self\n    where\n        F: FnOnce(&mut crate::pkg::SubMsgBuilder<'_, 'b>),"));
        // Singular alias for `f_messages` -> `f_message`
        assert!(out.contains("pub fn write_f_message<F>(&mut self, f: F) -> &mut Self\n    where\n        F: FnOnce(&mut crate::pkg::SubMsgBuilder<'_, 'b>),\n    {\n        self.write_f_messages(f)\n    }"));
        assert!(out.contains("pub fn write_f_bytes(&mut self, val: &[u8]) -> &mut Self {\n        let _ = self.writer.write_length_delimited(val);\n        let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);\n        self"));
        assert!(out.contains("pub fn write_f_uint32(&mut self, val: u32) -> &mut Self {\n        let _ = self.writer.write_varint(val as u64);\n        let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_enum(&mut self, val: crate::pkg::Mode) -> &mut Self {\n        let _ = self.writer.write_varint(val.as_i32() as u64);\n        let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_sfixed32(&mut self, val: i32) -> &mut Self {\n        let _ = self.writer.write_sfixed32(val);\n        let _ = self.writer.write_tag(15, ::roto::runtime::WireType::Fixed32);\n        self"));
        assert!(out.contains("pub fn write_f_sfixed64(&mut self, val: i64) -> &mut Self {\n        let _ = self.writer.write_sfixed64(val);\n        let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Fixed64);\n        self"));
        assert!(out.contains("pub fn write_f_sint32(&mut self, val: i32) -> &mut Self {\n        let _ = self.writer.write_zigzag32(val);\n        let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_f_sint64(&mut self, val: i64) -> &mut Self {\n        let _ = self.writer.write_zigzag64(val);\n        let _ = self.writer.write_tag(18, ::roto::runtime::WireType::Varint);\n        self"));
        assert!(out.contains("pub fn write_oneof_str(&mut self, val: &str) -> &mut Self"));

        // Check all 14 packed repeated scalar writers
        for packed_sig in [
            "pub fn write_packed_f_double(&mut self, vals: &[f64]) -> &mut Self",
            "pub fn write_packed_f_float(&mut self, vals: &[f32]) -> &mut Self",
            "pub fn write_packed_f_int64(&mut self, vals: &[i64]) -> &mut Self",
            "pub fn write_packed_f_uint64(&mut self, vals: &[u64]) -> &mut Self",
            "pub fn write_packed_f_int32(&mut self, vals: &[i32]) -> &mut Self",
            "pub fn write_packed_f_fixed64(&mut self, vals: &[u64]) -> &mut Self",
            "pub fn write_packed_f_fixed32(&mut self, vals: &[u32]) -> &mut Self",
            "pub fn write_packed_f_bool(&mut self, vals: &[bool]) -> &mut Self",
            "pub fn write_packed_f_uint32(&mut self, vals: &[u32]) -> &mut Self",
            "pub fn write_packed_f_enum(&mut self, vals: &[crate::pkg::Mode]) -> &mut Self",
            "pub fn write_packed_f_sfixed32(&mut self, vals: &[i32]) -> &mut Self",
            "pub fn write_packed_f_sfixed64(&mut self, vals: &[i64]) -> &mut Self",
            "pub fn write_packed_f_sint32(&mut self, vals: &[i32]) -> &mut Self",
            "pub fn write_packed_f_sint64(&mut self, vals: &[i64]) -> &mut Self",
        ] {
            assert!(out.contains(packed_sig), "missing {packed_sig}");
        }

        // Non-packable repeated fields must NOT have write_packed_*
        assert!(!out.contains("write_packed_f_string"));
        assert!(!out.contains("write_packed_f_bytes"));
        assert!(!out.contains("write_packed_f_messages"));
        assert!(!out.contains("write_packed_f_group"));

        // `items` must not emit duplicate `write_item` because field 20 is already `item`
        assert_eq!(out.matches("pub fn write_item(").count(), 1);
        // `address` ends in "ss" so it must not emit `write_addres`
        assert!(!out.contains("write_addres("));
    }

    #[test]
    fn generates_builders_for_real_descriptor_and_plugin_protos() {
        let fds_bytes = load_descriptor_and_plugin_fds();
        let fds = FileDescriptorSetView::from_bytes(&fds_bytes).unwrap();
        let symbols = SymbolTable::from_files(fds.file().map(|f| f.unwrap())).unwrap();
        let config = GeneratorConfig::default();

        let desc_file = fds
            .file()
            .map(|f| f.unwrap())
            .find(|f| f.name() == Ok("google/protobuf/descriptor.proto"))
            .unwrap();
        let plugin_file = fds
            .file()
            .map(|f| f.unwrap())
            .find(|f| f.name() == Ok("google/protobuf/compiler/plugin.proto"))
            .unwrap();

        // 1. SourceCodeInfo.Location (nested message with packed path/span)
        let sci_msg = desc_file
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("SourceCodeInfo"))
            .unwrap();
        let loc_msg = sci_msg
            .nested_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("Location"))
            .unwrap();
        let mut loc_out = String::new();
        generate_builder(
            &loc_msg,
            ".google.protobuf.SourceCodeInfo.Location",
            &symbols,
            &config,
            1,
            &mut loc_out,
        )
        .unwrap();
        assert!(loc_out.contains("    pub struct LocationBuilder<'a, 'b> {"));
        assert!(loc_out.contains("        pub fn write_path(&mut self, val: i32) -> &mut Self"));
        assert!(loc_out.contains("        pub fn write_packed_path(&mut self, vals: &[i32]) -> &mut Self"));
        assert!(loc_out.contains("        pub fn write_span(&mut self, val: i32) -> &mut Self"));
        assert!(loc_out.contains("        pub fn write_packed_span(&mut self, vals: &[i32]) -> &mut Self"));
        assert!(loc_out.contains("        pub fn write_leading_detached_comments(&mut self, val: &str) -> &mut Self"));
        assert!(loc_out.contains("        pub fn write_leading_detached_comment(&mut self, val: &str) -> &mut Self"));

        // 2. CodeGeneratorRequest (singular alias `write_source_file_descriptor`)
        let req_msg = plugin_file
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("CodeGeneratorRequest"))
            .unwrap();
        let mut req_out = String::new();
        generate_builder(
            &req_msg,
            ".google.protobuf.compiler.CodeGeneratorRequest",
            &symbols,
            &config,
            0,
            &mut req_out,
        )
        .unwrap();
        assert!(req_out.contains("pub struct CodeGeneratorRequestBuilder<'a, 'b>"));
        assert!(req_out.contains("pub fn write_file_to_generate(&mut self, val: &str) -> &mut Self"));
        assert!(req_out.contains("pub fn write_proto_file<F>(&mut self, f: F) -> &mut Self"));
        assert!(req_out.contains("pub fn write_source_file_descriptors<F>(&mut self, f: F) -> &mut Self"));
        assert!(req_out.contains("pub fn write_source_file_descriptor<F>(&mut self, f: F) -> &mut Self"));
        assert!(req_out.contains("self.write_source_file_descriptors(f)"));
        assert!(req_out.contains("pub fn write_compiler_version<F>(&mut self, f: F) -> &mut Self"));

        // 3. Generate builders for EVERY message in descriptor.proto and plugin.proto
        for file in [desc_file, plugin_file] {
            let pkg = file.package().unwrap();
            let scope = format!(".{pkg}");
            for msg_res in file.message_type() {
                let msg = msg_res.unwrap();
                let mut out = String::new();
                generate_builder(&msg, &scope, &symbols, &config, 0, &mut out).unwrap();
                assert!(out.contains("Builder<'a, 'b>"));
                assert!(out.contains("pub const fn check(&self)"));
                assert!(out.contains("pub fn copy_unknown_fields("));
            }
        }
    }
}
