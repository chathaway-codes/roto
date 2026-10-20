#![forbid(unsafe_code)]

pub mod generator;

use std::fmt;
use std::io::{Read, Write};

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::{
        code_generator_response::Feature, CodeGeneratorRequestView, CodeGeneratorResponseBuilder,
    },
    DescriptorProtoView, EnumDescriptorProtoView, EnumOptionsView, EnumValueDescriptorProtoView,
    EnumValueOptionsView, ExtensionRangeOptionsView, FeatureSetView, FieldDescriptorProtoView,
    FieldOptionsView, FileDescriptorProtoView, FileOptionsView, MessageOptionsView,
    MethodDescriptorProtoView, MethodOptionsView, OneofDescriptorProtoView, OneofOptionsView,
    ServiceDescriptorProtoView, ServiceOptionsView, SourceCodeInfoView, UninterpretedOptionView,
};
use roto::runtime::{DecodeError, EncodeError};

use crate::generator::config::GeneratorConfig;

/// Output file entry produced by the code generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFile {
    pub name: String,
    pub content: String,
}

/// Top-level framing or I/O error that prevents reading the request or writing the response.
#[derive(Debug)]
pub enum PluginError {
    Framing(DecodeError),
    Encode(EncodeError),
    Io(std::io::Error),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framing(err) => {
                write!(f, "failed to decode CodeGeneratorRequest from stdin: {err}")
            }
            Self::Encode(err) => write!(f, "failed to encode CodeGeneratorResponse: {err}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Reads a `CodeGeneratorRequest` from `reader`, processes it, and writes the
/// serialized `CodeGeneratorResponse` to `writer`.
pub fn run_stdio<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<(), PluginError> {
    let mut stdin_buf = Vec::new();
    reader.read_to_end(&mut stdin_buf)?;

    let response_bytes = process_bytes(&stdin_buf)?;
    writer.write_all(&response_bytes)?;
    writer.flush()?;
    Ok(())
}

/// Parses `stdin_buf` as a `CodeGeneratorRequestView`, executes the request handler,
/// and serializes the `CodeGeneratorResponse` using `CodeGeneratorResponseBuilder`.
pub fn process_bytes(stdin_buf: &[u8]) -> Result<Vec<u8>, PluginError> {
    let request = CodeGeneratorRequestView::from_bytes(stdin_buf).map_err(PluginError::Framing)?;

    let mut capacity = stdin_buf.len().saturating_mul(2).max(4096);
    loop {
        let mut backing = vec![0u8; capacity];
        match handle_request(&request, &mut backing) {
            Ok(bytes) => return Ok(bytes.to_vec()),
            Err(EncodeError::BufferOverflow) => {
                capacity = capacity
                    .checked_mul(2)
                    .ok_or(PluginError::Encode(EncodeError::BufferOverflow))?;
            }
        }
    }
}

/// Processes a parsed `CodeGeneratorRequestView` and writes the `CodeGeneratorResponse`
/// into `buf` using `CodeGeneratorResponseBuilder`.
///
/// Schema or generator errors are reported via `CodeGeneratorResponseBuilder::write_error`
/// with exit status `0` per the `plugin.proto` specification.
pub fn handle_request<'b>(
    request: &CodeGeneratorRequestView<'_>,
    buf: &'b mut [u8],
) -> Result<&'b [u8], EncodeError> {
    let mut resp = CodeGeneratorResponseBuilder::new(buf);

    match generate_request(request) {
        Ok(files) => {
            for file in files.iter().rev() {
                resp.write_file(|fb| {
                    fb.write_content(&file.content).write_name(&file.name);
                })
                .check()?;
            }
        }
        Err(err_msg) => {
            resp.write_error(&err_msg);
        }
    }

    resp.write_supported_features(Feature::FeatureProto3Optional as u64);
    resp.finish()
}

/// Validates all descriptors in `request` and invokes the full `roto` code generator.
pub fn generate_request(
    request: &CodeGeneratorRequestView<'_>,
) -> Result<Vec<GeneratedFile>, String> {
    crate::generator::generate(request)
}

/// Validates a `CodeGeneratorRequestView`, walking all `proto_file` and `source_file_descriptors`
/// and verifying every requested `file_to_generate` exists in `proto_file`.
pub fn validate_request(request: &CodeGeneratorRequestView<'_>) -> Result<(), String> {
    let _config = GeneratorConfig::from_request(request)?;
    validate_request_descriptors(request)
}

/// Validates all descriptors and `file_to_generate` entries in `request` without parsing
/// `CodeGeneratorRequest.parameter` (allowing plugins with extended parameter syntax to reuse
/// descriptor validation).
pub fn validate_request_descriptors(request: &CodeGeneratorRequestView<'_>) -> Result<(), String> {
    if let Some(ver) = request
        .compiler_version()
        .map_err(|e| format!("invalid CodeGeneratorRequest.compiler_version: {e}"))?
    {
        ver.major()
            .map_err(|e| format!("invalid Version.major: {e}"))?;
        ver.minor()
            .map_err(|e| format!("invalid Version.minor: {e}"))?;
        ver.patch()
            .map_err(|e| format!("invalid Version.patch: {e}"))?;
        ver.suffix()
            .map_err(|e| format!("invalid Version.suffix: {e}"))?;
    }

    let mut proto_file_names = Vec::new();
    for file_res in request.proto_file() {
        let file =
            file_res.map_err(|e| format!("invalid FileDescriptorProto in proto_file: {e}"))?;
        validate_file_descriptor(&file)?;
        let name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
        proto_file_names.push(name);
    }

    for file_res in request.source_file_descriptors() {
        let file = file_res
            .map_err(|e| format!("invalid FileDescriptorProto in source_file_descriptors: {e}"))?;
        validate_file_descriptor(&file)?;
    }

    for target_res in request.file_to_generate() {
        let target = target_res
            .map_err(|e| format!("invalid CodeGeneratorRequest.file_to_generate: {e}"))?;
        if target.is_empty() {
            return Err("file_to_generate entry cannot be empty".to_string());
        }
        if !proto_file_names.contains(&target) {
            return Err(format!(
                "file_to_generate '{target}' not found in proto_file descriptors"
            ));
        }
    }

    Ok(())
}

/// Walks and validates all fields, messages, nested messages, enums, oneofs, extensions,
/// services, methods, options, and source code info of a `FileDescriptorProtoView`.
pub fn validate_file_descriptor(file: &FileDescriptorProtoView<'_>) -> Result<(), String> {
    let file_name = file
        .name()
        .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
    if file.has_name() && file_name.is_empty() {
        return Err("FileDescriptorProto.name cannot be empty when present".to_string());
    }
    file.package()
        .map_err(|e| format!("invalid FileDescriptorProto.package in '{file_name}': {e}"))?;
    file.syntax()
        .map_err(|e| format!("invalid FileDescriptorProto.syntax in '{file_name}': {e}"))?;
    file.edition()
        .map_err(|e| format!("invalid FileDescriptorProto.edition in '{file_name}': {e}"))?;

    for dep in file.dependency() {
        dep.map_err(|e| format!("invalid FileDescriptorProto.dependency in '{file_name}': {e}"))?;
    }
    for pub_dep in file.public_dependency() {
        pub_dep.map_err(|e| {
            format!("invalid FileDescriptorProto.public_dependency in '{file_name}': {e}")
        })?;
    }
    for weak_dep in file.weak_dependency() {
        weak_dep.map_err(|e| {
            format!("invalid FileDescriptorProto.weak_dependency in '{file_name}': {e}")
        })?;
    }
    for opt_dep in file.option_dependency() {
        opt_dep.map_err(|e| {
            format!("invalid FileDescriptorProto.option_dependency in '{file_name}': {e}")
        })?;
    }

    for msg_res in file.message_type() {
        let msg = msg_res.map_err(|e| format!("invalid DescriptorProto in '{file_name}': {e}"))?;
        validate_message(&msg)?;
    }

    for enum_res in file.enum_type() {
        let en =
            enum_res.map_err(|e| format!("invalid EnumDescriptorProto in '{file_name}': {e}"))?;
        validate_enum(&en)?;
    }

    for svc_res in file.service() {
        let svc =
            svc_res.map_err(|e| format!("invalid ServiceDescriptorProto in '{file_name}': {e}"))?;
        validate_service(&svc)?;
    }

    for ext_res in file.extension() {
        let ext = ext_res
            .map_err(|e| format!("invalid extension FieldDescriptorProto in '{file_name}': {e}"))?;
        validate_field(&ext)?;
    }

    if let Some(opts) = file
        .options()
        .map_err(|e| format!("invalid FileOptions in '{file_name}': {e}"))?
    {
        validate_file_options(&opts)?;
    }

    if let Some(sci) = file
        .source_code_info()
        .map_err(|e| format!("invalid SourceCodeInfo in '{file_name}': {e}"))?
    {
        validate_source_code_info(&sci)?;
    }

    Ok(())
}

fn validate_message(msg: &DescriptorProtoView<'_>) -> Result<(), String> {
    let msg_name = msg
        .name()
        .map_err(|e| format!("invalid DescriptorProto.name: {e}"))?;
    msg.visibility()
        .map_err(|e| format!("invalid DescriptorProto.visibility in '{msg_name}': {e}"))?;

    for field_res in msg.field() {
        let field =
            field_res.map_err(|e| format!("invalid FieldDescriptorProto in '{msg_name}': {e}"))?;
        validate_field(&field)?;
    }

    for ext_res in msg.extension() {
        let ext = ext_res
            .map_err(|e| format!("invalid extension FieldDescriptorProto in '{msg_name}': {e}"))?;
        validate_field(&ext)?;
    }

    for nested_res in msg.nested_type() {
        let nested = nested_res
            .map_err(|e| format!("invalid nested DescriptorProto in '{msg_name}': {e}"))?;
        validate_message(&nested)?;
    }

    for enum_res in msg.enum_type() {
        let en = enum_res
            .map_err(|e| format!("invalid nested EnumDescriptorProto in '{msg_name}': {e}"))?;
        validate_enum(&en)?;
    }

    for er_res in msg.extension_range() {
        let er = er_res.map_err(|e| format!("invalid ExtensionRange in '{msg_name}': {e}"))?;
        er.start()
            .map_err(|e| format!("invalid ExtensionRange.start in '{msg_name}': {e}"))?;
        er.end()
            .map_err(|e| format!("invalid ExtensionRange.end in '{msg_name}': {e}"))?;
        if let Some(opts) = er
            .options()
            .map_err(|e| format!("invalid ExtensionRangeOptions in '{msg_name}': {e}"))?
        {
            validate_extension_range_options(&opts)?;
        }
    }

    for oneof_res in msg.oneof_decl() {
        let oneof =
            oneof_res.map_err(|e| format!("invalid OneofDescriptorProto in '{msg_name}': {e}"))?;
        validate_oneof(&oneof)?;
    }

    if let Some(opts) = msg
        .options()
        .map_err(|e| format!("invalid MessageOptions in '{msg_name}': {e}"))?
    {
        validate_message_options(&opts)?;
    }

    for rr_res in msg.reserved_range() {
        let rr = rr_res.map_err(|e| format!("invalid ReservedRange in '{msg_name}': {e}"))?;
        rr.start()
            .map_err(|e| format!("invalid ReservedRange.start in '{msg_name}': {e}"))?;
        rr.end()
            .map_err(|e| format!("invalid ReservedRange.end in '{msg_name}': {e}"))?;
    }

    for rn_res in msg.reserved_name() {
        rn_res
            .map_err(|e| format!("invalid DescriptorProto.reserved_name in '{msg_name}': {e}"))?;
    }

    Ok(())
}

fn validate_field(field: &FieldDescriptorProtoView<'_>) -> Result<(), String> {
    let field_name = field
        .name()
        .map_err(|e| format!("invalid FieldDescriptorProto.name: {e}"))?;
    field
        .extendee()
        .map_err(|e| format!("invalid FieldDescriptorProto.extendee in '{field_name}': {e}"))?;
    field
        .number()
        .map_err(|e| format!("invalid FieldDescriptorProto.number in '{field_name}': {e}"))?;
    field
        .label()
        .map_err(|e| format!("invalid FieldDescriptorProto.label in '{field_name}': {e}"))?;
    field
        .r#type()
        .map_err(|e| format!("invalid FieldDescriptorProto.type in '{field_name}': {e}"))?;
    field
        .type_name()
        .map_err(|e| format!("invalid FieldDescriptorProto.type_name in '{field_name}': {e}"))?;
    field.default_value().map_err(|e| {
        format!("invalid FieldDescriptorProto.default_value in '{field_name}': {e}")
    })?;
    field
        .oneof_index()
        .map_err(|e| format!("invalid FieldDescriptorProto.oneof_index in '{field_name}': {e}"))?;
    field
        .json_name()
        .map_err(|e| format!("invalid FieldDescriptorProto.json_name in '{field_name}': {e}"))?;
    field.proto3_optional().map_err(|e| {
        format!("invalid FieldDescriptorProto.proto3_optional in '{field_name}': {e}")
    })?;

    if let Some(opts) = field
        .options()
        .map_err(|e| format!("invalid FieldOptions in '{field_name}': {e}"))?
    {
        validate_field_options(&opts)?;
    }

    Ok(())
}

fn validate_oneof(oneof: &OneofDescriptorProtoView<'_>) -> Result<(), String> {
    let name = oneof
        .name()
        .map_err(|e| format!("invalid OneofDescriptorProto.name: {e}"))?;
    if let Some(opts) = oneof
        .options()
        .map_err(|e| format!("invalid OneofOptions in '{name}': {e}"))?
    {
        validate_oneof_options(&opts)?;
    }
    Ok(())
}

fn validate_enum(en: &EnumDescriptorProtoView<'_>) -> Result<(), String> {
    let enum_name = en
        .name()
        .map_err(|e| format!("invalid EnumDescriptorProto.name: {e}"))?;
    en.visibility()
        .map_err(|e| format!("invalid EnumDescriptorProto.visibility in '{enum_name}': {e}"))?;

    for val_res in en.value() {
        let val = val_res
            .map_err(|e| format!("invalid EnumValueDescriptorProto in '{enum_name}': {e}"))?;
        validate_enum_value(&val)?;
    }

    if let Some(opts) = en
        .options()
        .map_err(|e| format!("invalid EnumOptions in '{enum_name}': {e}"))?
    {
        validate_enum_options(&opts)?;
    }

    for rr_res in en.reserved_range() {
        let rr = rr_res.map_err(|e| format!("invalid EnumReservedRange in '{enum_name}': {e}"))?;
        rr.start()
            .map_err(|e| format!("invalid EnumReservedRange.start in '{enum_name}': {e}"))?;
        rr.end()
            .map_err(|e| format!("invalid EnumReservedRange.end in '{enum_name}': {e}"))?;
    }

    for rn_res in en.reserved_name() {
        rn_res.map_err(|e| {
            format!("invalid EnumDescriptorProto.reserved_name in '{enum_name}': {e}")
        })?;
    }

    Ok(())
}

fn validate_enum_value(val: &EnumValueDescriptorProtoView<'_>) -> Result<(), String> {
    let name = val
        .name()
        .map_err(|e| format!("invalid EnumValueDescriptorProto.name: {e}"))?;
    val.number()
        .map_err(|e| format!("invalid EnumValueDescriptorProto.number in '{name}': {e}"))?;
    if let Some(opts) = val
        .options()
        .map_err(|e| format!("invalid EnumValueOptions in '{name}': {e}"))?
    {
        validate_enum_value_options(&opts)?;
    }
    Ok(())
}

fn validate_service(svc: &ServiceDescriptorProtoView<'_>) -> Result<(), String> {
    let svc_name = svc
        .name()
        .map_err(|e| format!("invalid ServiceDescriptorProto.name: {e}"))?;
    for method_res in svc.method() {
        let method = method_res
            .map_err(|e| format!("invalid MethodDescriptorProto in '{svc_name}': {e}"))?;
        validate_method(&method)?;
    }
    if let Some(opts) = svc
        .options()
        .map_err(|e| format!("invalid ServiceOptions in '{svc_name}': {e}"))?
    {
        validate_service_options(&opts)?;
    }
    Ok(())
}

fn validate_method(method: &MethodDescriptorProtoView<'_>) -> Result<(), String> {
    let name = method
        .name()
        .map_err(|e| format!("invalid MethodDescriptorProto.name: {e}"))?;
    method
        .input_type()
        .map_err(|e| format!("invalid MethodDescriptorProto.input_type in '{name}': {e}"))?;
    method
        .output_type()
        .map_err(|e| format!("invalid MethodDescriptorProto.output_type in '{name}': {e}"))?;
    method
        .client_streaming()
        .map_err(|e| format!("invalid MethodDescriptorProto.client_streaming in '{name}': {e}"))?;
    method
        .server_streaming()
        .map_err(|e| format!("invalid MethodDescriptorProto.server_streaming in '{name}': {e}"))?;
    if let Some(opts) = method
        .options()
        .map_err(|e| format!("invalid MethodOptions in '{name}': {e}"))?
    {
        validate_method_options(&opts)?;
    }
    Ok(())
}

fn validate_file_options(opts: &FileOptionsView<'_>) -> Result<(), String> {
    opts.java_package()
        .map_err(|e| format!("invalid FileOptions.java_package: {e}"))?;
    opts.java_outer_classname()
        .map_err(|e| format!("invalid FileOptions.java_outer_classname: {e}"))?;
    opts.optimize_for()
        .map_err(|e| format!("invalid FileOptions.optimize_for: {e}"))?;
    opts.java_multiple_files()
        .map_err(|e| format!("invalid FileOptions.java_multiple_files: {e}"))?;
    opts.go_package()
        .map_err(|e| format!("invalid FileOptions.go_package: {e}"))?;
    opts.cc_generic_services()
        .map_err(|e| format!("invalid FileOptions.cc_generic_services: {e}"))?;
    opts.java_generic_services()
        .map_err(|e| format!("invalid FileOptions.java_generic_services: {e}"))?;
    opts.py_generic_services()
        .map_err(|e| format!("invalid FileOptions.py_generic_services: {e}"))?;
    opts.java_generate_equals_and_hash()
        .map_err(|e| format!("invalid FileOptions.java_generate_equals_and_hash: {e}"))?;
    opts.deprecated()
        .map_err(|e| format!("invalid FileOptions.deprecated: {e}"))?;
    opts.java_string_check_utf8()
        .map_err(|e| format!("invalid FileOptions.java_string_check_utf8: {e}"))?;
    opts.cc_enable_arenas()
        .map_err(|e| format!("invalid FileOptions.cc_enable_arenas: {e}"))?;
    opts.objc_class_prefix()
        .map_err(|e| format!("invalid FileOptions.objc_class_prefix: {e}"))?;
    opts.csharp_namespace()
        .map_err(|e| format!("invalid FileOptions.csharp_namespace: {e}"))?;
    opts.swift_prefix()
        .map_err(|e| format!("invalid FileOptions.swift_prefix: {e}"))?;
    opts.php_class_prefix()
        .map_err(|e| format!("invalid FileOptions.php_class_prefix: {e}"))?;
    opts.php_namespace()
        .map_err(|e| format!("invalid FileOptions.php_namespace: {e}"))?;
    opts.php_metadata_namespace()
        .map_err(|e| format!("invalid FileOptions.php_metadata_namespace: {e}"))?;
    opts.ruby_package()
        .map_err(|e| format!("invalid FileOptions.ruby_package: {e}"))?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid FileOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo = uo_res.map_err(|e| format!("invalid UninterpretedOption in FileOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_message_options(opts: &MessageOptionsView<'_>) -> Result<(), String> {
    opts.message_set_wire_format()
        .map_err(|e| format!("invalid MessageOptions.message_set_wire_format: {e}"))?;
    opts.no_standard_descriptor_accessor()
        .map_err(|e| format!("invalid MessageOptions.no_standard_descriptor_accessor: {e}"))?;
    opts.deprecated()
        .map_err(|e| format!("invalid MessageOptions.deprecated: {e}"))?;
    opts.map_entry()
        .map_err(|e| format!("invalid MessageOptions.map_entry: {e}"))?;
    opts.deprecated_legacy_json_field_conflicts().map_err(|e| {
        format!("invalid MessageOptions.deprecated_legacy_json_field_conflicts: {e}")
    })?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid MessageOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo =
            uo_res.map_err(|e| format!("invalid UninterpretedOption in MessageOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_field_options(opts: &FieldOptionsView<'_>) -> Result<(), String> {
    opts.ctype()
        .map_err(|e| format!("invalid FieldOptions.ctype: {e}"))?;
    opts.packed()
        .map_err(|e| format!("invalid FieldOptions.packed: {e}"))?;
    opts.deprecated()
        .map_err(|e| format!("invalid FieldOptions.deprecated: {e}"))?;
    opts.lazy()
        .map_err(|e| format!("invalid FieldOptions.lazy: {e}"))?;
    opts.jstype()
        .map_err(|e| format!("invalid FieldOptions.jstype: {e}"))?;
    opts.weak()
        .map_err(|e| format!("invalid FieldOptions.weak: {e}"))?;
    opts.unverified_lazy()
        .map_err(|e| format!("invalid FieldOptions.unverified_lazy: {e}"))?;
    opts.debug_redact()
        .map_err(|e| format!("invalid FieldOptions.debug_redact: {e}"))?;
    opts.retention()
        .map_err(|e| format!("invalid FieldOptions.retention: {e}"))?;
    for target_res in opts.targets() {
        target_res.map_err(|e| format!("invalid FieldOptions.targets: {e}"))?;
    }
    for ed_res in opts.edition_defaults() {
        let ed = ed_res.map_err(|e| format!("invalid FieldOptions.edition_defaults: {e}"))?;
        ed.edition()
            .map_err(|e| format!("invalid EditionDefault.edition: {e}"))?;
        ed.value()
            .map_err(|e| format!("invalid EditionDefault.value: {e}"))?;
    }
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid FieldOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    if let Some(fsup) = opts
        .feature_support()
        .map_err(|e| format!("invalid FieldOptions.feature_support: {e}"))?
    {
        fsup.edition_introduced()
            .map_err(|e| format!("invalid FeatureSupport.edition_introduced: {e}"))?;
        fsup.edition_deprecated()
            .map_err(|e| format!("invalid FeatureSupport.edition_deprecated: {e}"))?;
        fsup.deprecation_warning()
            .map_err(|e| format!("invalid FeatureSupport.deprecation_warning: {e}"))?;
        fsup.edition_removed()
            .map_err(|e| format!("invalid FeatureSupport.edition_removed: {e}"))?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo = uo_res.map_err(|e| format!("invalid UninterpretedOption in FieldOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_oneof_options(opts: &OneofOptionsView<'_>) -> Result<(), String> {
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid OneofOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo = uo_res.map_err(|e| format!("invalid UninterpretedOption in OneofOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_enum_options(opts: &EnumOptionsView<'_>) -> Result<(), String> {
    opts.allow_alias()
        .map_err(|e| format!("invalid EnumOptions.allow_alias: {e}"))?;
    opts.deprecated()
        .map_err(|e| format!("invalid EnumOptions.deprecated: {e}"))?;
    opts.deprecated_legacy_json_field_conflicts()
        .map_err(|e| format!("invalid EnumOptions.deprecated_legacy_json_field_conflicts: {e}"))?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid EnumOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo = uo_res.map_err(|e| format!("invalid UninterpretedOption in EnumOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_enum_value_options(opts: &EnumValueOptionsView<'_>) -> Result<(), String> {
    opts.deprecated()
        .map_err(|e| format!("invalid EnumValueOptions.deprecated: {e}"))?;
    opts.debug_redact()
        .map_err(|e| format!("invalid EnumValueOptions.debug_redact: {e}"))?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid EnumValueOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    if let Some(fsup) = opts
        .feature_support()
        .map_err(|e| format!("invalid EnumValueOptions.feature_support: {e}"))?
    {
        fsup.edition_introduced()
            .map_err(|e| format!("invalid FeatureSupport.edition_introduced: {e}"))?;
        fsup.edition_deprecated()
            .map_err(|e| format!("invalid FeatureSupport.edition_deprecated: {e}"))?;
        fsup.deprecation_warning()
            .map_err(|e| format!("invalid FeatureSupport.deprecation_warning: {e}"))?;
        fsup.edition_removed()
            .map_err(|e| format!("invalid FeatureSupport.edition_removed: {e}"))?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo =
            uo_res.map_err(|e| format!("invalid UninterpretedOption in EnumValueOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_service_options(opts: &ServiceOptionsView<'_>) -> Result<(), String> {
    opts.deprecated()
        .map_err(|e| format!("invalid ServiceOptions.deprecated: {e}"))?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid ServiceOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo =
            uo_res.map_err(|e| format!("invalid UninterpretedOption in ServiceOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_method_options(opts: &MethodOptionsView<'_>) -> Result<(), String> {
    opts.deprecated()
        .map_err(|e| format!("invalid MethodOptions.deprecated: {e}"))?;
    opts.idempotency_level()
        .map_err(|e| format!("invalid MethodOptions.idempotency_level: {e}"))?;
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid MethodOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo =
            uo_res.map_err(|e| format!("invalid UninterpretedOption in MethodOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_extension_range_options(opts: &ExtensionRangeOptionsView<'_>) -> Result<(), String> {
    opts.verification()
        .map_err(|e| format!("invalid ExtensionRangeOptions.verification: {e}"))?;
    for decl_res in opts.declaration() {
        let decl =
            decl_res.map_err(|e| format!("invalid ExtensionRangeOptions.declaration: {e}"))?;
        decl.number()
            .map_err(|e| format!("invalid Declaration.number: {e}"))?;
        decl.full_name()
            .map_err(|e| format!("invalid Declaration.full_name: {e}"))?;
        decl.r#type()
            .map_err(|e| format!("invalid Declaration.type: {e}"))?;
        decl.reserved()
            .map_err(|e| format!("invalid Declaration.reserved: {e}"))?;
        decl.repeated()
            .map_err(|e| format!("invalid Declaration.repeated: {e}"))?;
    }
    if let Some(fs) = opts
        .features()
        .map_err(|e| format!("invalid ExtensionRangeOptions.features: {e}"))?
    {
        validate_feature_set(&fs)?;
    }
    for uo_res in opts.uninterpreted_option() {
        let uo = uo_res
            .map_err(|e| format!("invalid UninterpretedOption in ExtensionRangeOptions: {e}"))?;
        validate_uninterpreted_option(&uo)?;
    }
    Ok(())
}

fn validate_feature_set(fs: &FeatureSetView<'_>) -> Result<(), String> {
    fs.field_presence()
        .map_err(|e| format!("invalid FeatureSet.field_presence: {e}"))?;
    fs.enum_type()
        .map_err(|e| format!("invalid FeatureSet.enum_type: {e}"))?;
    fs.repeated_field_encoding()
        .map_err(|e| format!("invalid FeatureSet.repeated_field_encoding: {e}"))?;
    fs.utf8_validation()
        .map_err(|e| format!("invalid FeatureSet.utf8_validation: {e}"))?;
    fs.message_encoding()
        .map_err(|e| format!("invalid FeatureSet.message_encoding: {e}"))?;
    fs.json_format()
        .map_err(|e| format!("invalid FeatureSet.json_format: {e}"))?;
    fs.enforce_naming_style()
        .map_err(|e| format!("invalid FeatureSet.enforce_naming_style: {e}"))?;
    fs.default_symbol_visibility()
        .map_err(|e| format!("invalid FeatureSet.default_symbol_visibility: {e}"))?;
    Ok(())
}

fn validate_uninterpreted_option(uo: &UninterpretedOptionView<'_>) -> Result<(), String> {
    for np_res in uo.name() {
        let np = np_res.map_err(|e| format!("invalid UninterpretedOption.NamePart: {e}"))?;
        np.name_part()
            .map_err(|e| format!("invalid NamePart.name_part: {e}"))?;
        np.is_extension()
            .map_err(|e| format!("invalid NamePart.is_extension: {e}"))?;
    }
    uo.identifier_value()
        .map_err(|e| format!("invalid UninterpretedOption.identifier_value: {e}"))?;
    uo.positive_int_value()
        .map_err(|e| format!("invalid UninterpretedOption.positive_int_value: {e}"))?;
    uo.negative_int_value()
        .map_err(|e| format!("invalid UninterpretedOption.negative_int_value: {e}"))?;
    uo.double_value()
        .map_err(|e| format!("invalid UninterpretedOption.double_value: {e}"))?;
    uo.string_value()
        .map_err(|e| format!("invalid UninterpretedOption.string_value: {e}"))?;
    uo.aggregate_value()
        .map_err(|e| format!("invalid UninterpretedOption.aggregate_value: {e}"))?;
    Ok(())
}

fn validate_source_code_info(sci: &SourceCodeInfoView<'_>) -> Result<(), String> {
    for loc_res in sci.location() {
        let loc = loc_res.map_err(|e| format!("invalid SourceCodeInfo.Location: {e}"))?;
        for p in loc.path() {
            p.map_err(|e| format!("invalid SourceCodeInfo.Location.path: {e}"))?;
        }
        for s in loc.span() {
            s.map_err(|e| format!("invalid SourceCodeInfo.Location.span: {e}"))?;
        }
        loc.leading_comments()
            .map_err(|e| format!("invalid SourceCodeInfo.Location.leading_comments: {e}"))?;
        loc.trailing_comments()
            .map_err(|e| format!("invalid SourceCodeInfo.Location.trailing_comments: {e}"))?;
        for c in loc.leading_detached_comments() {
            c.map_err(|e| {
                format!("invalid SourceCodeInfo.Location.leading_detached_comments: {e}")
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use google_protobuf_roto_bootstrap::google::protobuf::{
        compiler::{CodeGeneratorRequestBuilder, CodeGeneratorResponseView},
        field_descriptor_proto::{Label, Type},
    };
    use std::io::Cursor;

    #[test]
    fn run_stdio_roundtrip_and_feature_flags() {
        let mut backing = [0u8; 512];
        let mut req = CodeGeneratorRequestBuilder::new(&mut backing);
        req.write_proto_file(|file| {
            file.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(Type::TypeInt32)
                        .write_label(Label::LabelRequired)
                        .write_number(1)
                        .write_name("lat_e7");
                })
                .write_name("Location");
            })
            .write_package("telemetry")
            .write_name("sensor.proto");
        })
        .write_file_to_generate("sensor.proto");
        let req_bytes = req.finish().unwrap();

        let mut stdout_buf = Vec::new();
        run_stdio(Cursor::new(req_bytes), &mut stdout_buf).unwrap();

        let resp = CodeGeneratorResponseView::from_bytes(&stdout_buf).unwrap();
        assert!(!resp.has_error());
        assert_eq!(
            resp.supported_features(),
            Ok(Feature::FeatureProto3Optional as u64)
        );
    }

    #[test]
    fn run_stdio_rejects_empty_file_to_generate() {
        let mut backing = [0u8; 128];
        let mut req = CodeGeneratorRequestBuilder::new(&mut backing);
        req.write_file_to_generate("");
        let req_bytes = req.finish().unwrap();

        let resp_bytes = process_bytes(req_bytes).unwrap();
        let resp = CodeGeneratorResponseView::from_bytes(&resp_bytes).unwrap();
        assert!(resp.has_error());
        assert_eq!(resp.error(), Ok("file_to_generate entry cannot be empty"));
    }
}
