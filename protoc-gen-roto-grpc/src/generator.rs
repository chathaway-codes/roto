use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::CodeGeneratorRequestView, field_descriptor_proto::Type as FieldType,
    FileDescriptorProtoView, ServiceDescriptorProtoView,
};
use protoc_gen_roto::{
    generator::{
        config::{GeneratorConfig, OutputMode},
        ident::{
            escape_ident, escape_mod_ident, sanitize_type_ident, to_pascal_case,
            to_screaming_snake_case, to_snake_case,
        },
        modules::format_bytes_literal,
        symbols::{ResolvedType, SymbolTable},
    },
    GeneratedFile,
};

use crate::http::{
    extract_and_validate_method_http_rule, HttpPattern, MessageIndex, PathSegment,
    ResolvedFieldStep, SubSegment, ValidatedBody, ValidatedHttpBinding, ValidatedHttpRule,
    ValidatedResponseBody,
};

const LIB_RS_HEADER: &str = "\
#![no_std]
#![forbid(unsafe_code)]
#![allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]
";

const MOD_RS_HEADER: &str = "\
#![forbid(unsafe_code)]
#![allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]
";

/// Parsed plugin configuration for `protoc-gen-roto-grpc`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrpcGeneratorConfig {
    /// Underlying `protoc-gen-roto` configuration (`mode` and `extern_paths`).
    pub inner: GeneratorConfig,
    /// Optional explicit output file name (`file=grpc.rs`).
    pub file: Option<String>,
    /// Whether `google.api.http` route and Protobuf binding codegen is enabled (`http` / `http=true`).
    pub http: bool,
}

impl GrpcGeneratorConfig {
    /// Parses `CodeGeneratorRequestView::parameter()` into a [`GrpcGeneratorConfig`].
    pub fn from_request(request: &CodeGeneratorRequestView<'_>) -> Result<Self, String> {
        let parameter = request
            .parameter()
            .map_err(|e| format!("invalid CodeGeneratorRequest.parameter: {e}"))?;
        Self::parse(parameter)
    }

    /// Parses a comma-separated `protoc` plugin parameter string (`--roto-grpc_opt=...`).
    ///
    /// Supported options:
    /// - `mode=lib` (default) or `mode=mod`
    /// - `file=<filename>` (e.g. `file=grpc.rs`)
    /// - `descriptors` or `descriptors=true` / `descriptors=false`
    /// - `http` or `http=true` / `http=false`
    /// - `extern_path=<proto_prefix>=<rust_crate_path>`
    /// - `error=<msg>`
    pub fn parse(parameter: &str) -> Result<Self, String> {
        let mut file = None;
        let mut http = false;
        let mut delegated_parts = Vec::new();

        for raw_part in parameter.split(',') {
            let part = raw_part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some(raw_file) = part.strip_prefix("file=") {
                let trimmed = raw_file.trim();
                if trimmed.is_empty()
                    || trimmed.starts_with('/')
                    || trimmed.contains("..")
                    || trimmed.contains('\\')
                {
                    return Err(format!("invalid file parameter '{part}'"));
                }
                file = Some(trimmed.to_string());
            } else if part == "http" {
                http = true;
            } else if let Some(raw_http) = part.strip_prefix("http=") {
                match raw_http.trim() {
                    "true" => http = true,
                    "false" => http = false,
                    _ => {
                        return Err(format!(
                            "invalid http parameter '{part}': expected 'http', 'http=true', or 'http=false'"
                        ));
                    }
                }
            } else {
                delegated_parts.push(part);
            }
        }

        let inner = GeneratorConfig::parse(&delegated_parts.join(","))?;
        Ok(Self { inner, file, http })
    }

    /// Returns the output file name (`"lib.rs"` in `OutputMode::Lib`, `"grpc.rs"` in
    /// `OutputMode::Mod`, or the explicit `file=...` override).
    pub fn output_file_name(&self) -> &str {
        if let Some(ref f) = self.file {
            f.as_str()
        } else {
            match self.inner.mode {
                OutputMode::Lib => "lib.rs",
                OutputMode::Mod => "grpc.rs",
            }
        }
    }
}

#[derive(Default)]
struct ServicePackageNode<'a> {
    proto_scope: String,
    raw_package: String,
    files: Vec<FileDescriptorProtoView<'a>>,
    children: BTreeMap<String, ServicePackageNode<'a>>,
}

#[derive(Clone, Copy)]
struct RenderContext<'a> {
    symbols: &'a SymbolTable,
    config: &'a GrpcGeneratorConfig,
    proto_files_by_name: &'a BTreeMap<&'a str, FileDescriptorProtoView<'a>>,
    message_index: Option<&'a MessageIndex<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamingMode {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BidiStreaming,
}

impl StreamingMode {
    const fn is_client_streaming(self) -> bool {
        matches!(self, Self::ClientStreaming | Self::BidiStreaming)
    }

    const fn is_bidi_streaming(self) -> bool {
        matches!(self, Self::BidiStreaming)
    }

    const fn is_single_request(self) -> bool {
        matches!(self, Self::Unary | Self::ServerStreaming)
    }
}

struct ResolvedMethod {
    raw_name: String,
    method_ident: String,
    async_method_ident: String,
    with_method_ident: String,
    with_async_method_ident: String,
    req_trait_ident: String,
    path_const_ident: String,
    path_value: String,
    mode: StreamingMode,
    input: ResolvedType,
    output: ResolvedType,
    http_rule: Option<ValidatedHttpRule>,
}

/// Generates `#![no_std]` zero-copy gRPC service traits, `<Service>Server<T>` dispatchers,
/// and `<Service>Client<C>` stubs for all target files listed in `request.file_to_generate()`.
pub fn generate(request: &CodeGeneratorRequestView<'_>) -> Result<Vec<GeneratedFile>, String> {
    crate::validate_request(request)?;
    let config = GrpcGeneratorConfig::from_request(request)?;
    let symbols = SymbolTable::from_request(request)?;
    let message_index = if config.http {
        Some(MessageIndex::build(request)?)
    } else {
        None
    };

    let mut proto_files_by_name: BTreeMap<&str, FileDescriptorProtoView<'_>> = BTreeMap::new();
    for file_res in request.proto_file() {
        let file =
            file_res.map_err(|e| format!("invalid FileDescriptorProto in proto_file: {e}"))?;
        let name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
        proto_files_by_name.insert(name, file);
    }

    let ctx = RenderContext {
        symbols: &symbols,
        config: &config,
        proto_files_by_name: &proto_files_by_name,
        message_index: message_index.as_ref(),
    };

    let mut target_files: Vec<FileDescriptorProtoView<'_>> = Vec::new();
    let mut seen_targets: BTreeSet<&str> = BTreeSet::new();
    for target_res in request.file_to_generate() {
        let target_name = target_res
            .map_err(|e| format!("invalid CodeGeneratorRequest.file_to_generate: {e}"))?;
        if !seen_targets.insert(target_name) {
            continue;
        }
        let file = proto_files_by_name
            .get(target_name)
            .copied()
            .ok_or_else(|| {
                format!("file_to_generate '{target_name}' not found in proto_file descriptors")
            })?;
        target_files.push(file);
    }

    if target_files.is_empty() {
        return Ok(Vec::new());
    }

    let root = build_service_package_tree(&target_files)?;
    let mut body = String::new();
    render_package_node(&root, ctx, 0, &mut body)?;

    let out_name = config.output_file_name().to_string();
    let header = if out_name == "lib.rs" && config.inner.mode == OutputMode::Lib {
        LIB_RS_HEADER
    } else {
        MOD_RS_HEADER
    };

    let mut content = String::with_capacity(header.len() + 1 + body.len());
    content.push_str(header);
    if !body.is_empty() {
        content.push('\n');
        content.push_str(&body);
    }

    Ok(vec![GeneratedFile {
        name: out_name,
        content,
    }])
}

fn build_service_package_tree<'a>(
    target_files: &[FileDescriptorProtoView<'a>],
) -> Result<ServicePackageNode<'a>, String> {
    let mut root = ServicePackageNode::default();

    for &file in target_files {
        if file.service().next().is_none() {
            continue;
        }

        let file_name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
        let raw_pkg = file
            .package()
            .map_err(|e| format!("invalid FileDescriptorProto.package in '{file_name}': {e}"))?
            .trim()
            .trim_matches('.');

        if raw_pkg.is_empty() {
            root.files.push(file);
            continue;
        }

        let mut curr = &mut root;
        let mut scope = String::new();
        let mut pkg_accum = String::new();
        for seg in raw_pkg.split('.').filter(|s| !s.is_empty()) {
            write!(scope, ".{seg}").unwrap();
            if !pkg_accum.is_empty() {
                pkg_accum.push('.');
            }
            pkg_accum.push_str(seg);
            let mod_ident = escape_mod_ident(seg);
            curr = curr
                .children
                .entry(mod_ident)
                .or_insert_with(|| ServicePackageNode {
                    proto_scope: scope.clone(),
                    raw_package: pkg_accum.clone(),
                    files: Vec::new(),
                    children: BTreeMap::new(),
                });
        }
        curr.files.push(file);
    }

    Ok(root)
}

fn render_package_node(
    node: &ServicePackageNode<'_>,
    ctx: RenderContext<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let mut first = true;

    let mut direct_out = String::new();
    render_node_services(node, ctx, indent, &mut direct_out)?;
    if !direct_out.is_empty() {
        out.push_str(&direct_out);
        first = false;
    }

    let pad = "    ".repeat(indent);
    for (mod_ident, child) in &node.children {
        if !first {
            writeln!(out).unwrap();
        }
        first = false;
        writeln!(out, "{pad}pub mod {mod_ident} {{").unwrap();
        render_package_node(child, ctx, indent + 1, out)?;
        writeln!(out, "{pad}}}").unwrap();
    }

    Ok(())
}

fn render_node_services(
    node: &ServicePackageNode<'_>,
    ctx: RenderContext<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let mut first = true;
    let mut seen_services: BTreeSet<String> = BTreeSet::new();

    for file in &node.files {
        let file_name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;

        for svc_res in file.service() {
            let svc = svc_res
                .map_err(|e| format!("invalid ServiceDescriptorProto in '{file_name}': {e}"))?;
            let raw_svc_name = svc.name().map_err(|e| {
                format!("invalid ServiceDescriptorProto.name in '{file_name}': {e}")
            })?;
            if raw_svc_name.is_empty() {
                return Err(format!(
                    "ServiceDescriptorProto.name cannot be empty in '{file_name}'"
                ));
            }

            let svc_mod_ident = escape_mod_ident(raw_svc_name);
            if !seen_services.insert(svc_mod_ident.clone()) {
                return Err(format!(
                    "duplicate service name '{raw_svc_name}' in package '{}'",
                    node.raw_package
                ));
            }

            if !first {
                writeln!(out).unwrap();
            }
            first = false;
            generate_service(
                file,
                &svc,
                &node.proto_scope,
                &node.raw_package,
                ctx,
                indent,
                out,
            )?;
        }
    }

    Ok(())
}

fn collect_transitive_descriptors<'a>(
    root_file: &FileDescriptorProtoView<'a>,
    proto_files_by_name: &BTreeMap<&'a str, FileDescriptorProtoView<'a>>,
) -> Result<Vec<FileDescriptorProtoView<'a>>, String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();

    fn visit<'a>(
        file: FileDescriptorProtoView<'a>,
        proto_files_by_name: &BTreeMap<&'a str, FileDescriptorProtoView<'a>>,
        seen: &mut BTreeSet<&'a str>,
        out: &mut Vec<FileDescriptorProtoView<'a>>,
    ) -> Result<(), String> {
        let name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
        if !seen.insert(name) {
            return Ok(());
        }
        out.push(file);
        for dep_res in file.dependency() {
            let dep_name = dep_res
                .map_err(|e| format!("invalid FileDescriptorProto.dependency in '{name}': {e}"))?;
            if let Some(&dep_file) = proto_files_by_name.get(dep_name) {
                visit(dep_file, proto_files_by_name, seen, out)?;
            }
        }
        Ok(())
    }

    visit(*root_file, proto_files_by_name, &mut seen, &mut out)?;
    Ok(out)
}

fn generate_service(
    file: &FileDescriptorProtoView<'_>,
    svc: &ServiceDescriptorProtoView<'_>,
    proto_scope: &str,
    raw_package: &str,
    ctx: RenderContext<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let raw_svc_name = svc
        .name()
        .map_err(|e| format!("invalid ServiceDescriptorProto.name: {e}"))?;
    let svc_mod_ident = escape_mod_ident(raw_svc_name);
    let svc_pascal = to_pascal_case(raw_svc_name);
    let svc_trait_ident = sanitize_type_ident(raw_svc_name);
    let async_trait_ident = format!("Async{svc_pascal}");
    let server_ident = format!("{svc_pascal}Server");
    let client_ident = format!("{svc_pascal}Client");

    let full_svc_name = if raw_package.is_empty() {
        raw_svc_name.to_string()
    } else {
        format!("{raw_package}.{raw_svc_name}")
    };

    let mut methods: Vec<ResolvedMethod> = Vec::new();
    let mut seen_methods: BTreeSet<String> = BTreeSet::new();

    for method_res in svc.method() {
        let method = method_res
            .map_err(|e| format!("invalid MethodDescriptorProto in '{raw_svc_name}': {e}"))?;
        let raw_method_name = method
            .name()
            .map_err(|e| format!("invalid MethodDescriptorProto.name in '{raw_svc_name}': {e}"))?;
        if raw_method_name.is_empty() {
            return Err(format!(
                "MethodDescriptorProto.name cannot be empty in service '{raw_svc_name}'"
            ));
        }

        let method_snake = to_snake_case(raw_method_name);
        if !seen_methods.insert(method_snake.clone()) {
            return Err(format!(
                "duplicate method name '{raw_method_name}' in service '{raw_svc_name}'"
            ));
        }

        let input_type_name = method.input_type().map_err(|e| {
            format!(
                "invalid MethodDescriptorProto.input_type in '{raw_svc_name}.{raw_method_name}': {e}"
            )
        })?;
        if input_type_name.is_empty() {
            return Err(format!(
                "MethodDescriptorProto.input_type cannot be empty in '{raw_svc_name}.{raw_method_name}'"
            ));
        }
        let input = ctx
            .symbols
            .resolve(proto_scope, input_type_name, &ctx.config.inner)?;
        if !input.is_message() {
            return Err(format!(
                "method '{raw_svc_name}.{raw_method_name}' input_type '{input_type_name}' must be a message, not an enum"
            ));
        }

        let output_type_name = method.output_type().map_err(|e| {
            format!(
                "invalid MethodDescriptorProto.output_type in '{raw_svc_name}.{raw_method_name}': {e}"
            )
        })?;
        if output_type_name.is_empty() {
            return Err(format!(
                "MethodDescriptorProto.output_type cannot be empty in '{raw_svc_name}.{raw_method_name}'"
            ));
        }
        let output = ctx
            .symbols
            .resolve(proto_scope, output_type_name, &ctx.config.inner)?;
        if !output.is_message() {
            return Err(format!(
                "method '{raw_svc_name}.{raw_method_name}' output_type '{output_type_name}' must be a message, not an enum"
            ));
        }

        let client_streaming = method.client_streaming().map_err(|e| {
            format!(
                "invalid MethodDescriptorProto.client_streaming in '{raw_svc_name}.{raw_method_name}': {e}"
            )
        })?;
        let server_streaming = method.server_streaming().map_err(|e| {
            format!(
                "invalid MethodDescriptorProto.server_streaming in '{raw_svc_name}.{raw_method_name}': {e}"
            )
        })?;

        let mode = match (client_streaming, server_streaming) {
            (false, false) => StreamingMode::Unary,
            (false, true) => StreamingMode::ServerStreaming,
            (true, false) => StreamingMode::ClientStreaming,
            (true, true) => StreamingMode::BidiStreaming,
        };

        let method_ident = escape_ident(&method_snake);
        let async_method_ident = format!("{method_snake}_async");
        let with_method_ident = format!("{method_snake}_with");
        let with_async_method_ident = format!("{method_snake}_with_async");
        let req_trait_ident = format!("{}Request", to_pascal_case(raw_method_name));
        let path_const_ident = format!("PATH_{}", to_screaming_snake_case(raw_method_name));
        let path_value = format!("/{full_svc_name}/{raw_method_name}");
        let http_rule = if let Some(index) = ctx.message_index {
            extract_and_validate_method_http_rule(&method, proto_scope, index)?
        } else {
            None
        };

        methods.push(ResolvedMethod {
            raw_name: raw_method_name.to_string(),
            method_ident,
            async_method_ident,
            with_method_ident,
            with_async_method_ident,
            req_trait_ident,
            path_const_ident,
            path_value,
            mode,
            input,
            output,
            http_rule,
        });
    }

    let pad = "    ".repeat(indent);
    let inner_pad = "    ".repeat(indent + 1);
    let fn_pad = "    ".repeat(indent + 2);
    let body_pad = "    ".repeat(indent + 3);
    let arm_pad = "    ".repeat(indent + 4);

    writeln!(out, "{pad}pub mod {svc_mod_ident} {{").unwrap();
    writeln!(
        out,
        "{inner_pad}pub const SERVICE_NAME: &str = {full_svc_name:?};"
    )
    .unwrap();

    if ctx.config.inner.descriptors {
        let transitive = collect_transitive_descriptors(file, ctx.proto_files_by_name)?;
        let file_lit = format_bytes_literal(file.as_bytes());
        writeln!(
            out,
            "{inner_pad}pub const FILE_DESCRIPTOR_BYTES: &[u8] = {file_lit};"
        )
        .unwrap();
        let mut set_elems = Vec::with_capacity(transitive.len());
        set_elems.push("FILE_DESCRIPTOR_BYTES".to_string());
        for dep_file in transitive.iter().skip(1) {
            set_elems.push(format!(
                "{} as &[u8]",
                format_bytes_literal(dep_file.as_bytes())
            ));
        }
        writeln!(
            out,
            "{inner_pad}pub const FILE_DESCRIPTOR_SET: &[&[u8]] = &[{}];",
            set_elems.join(", ")
        )
        .unwrap();
    }

    for m in &methods {
        writeln!(
            out,
            "{inner_pad}pub const {}: &str = {:?};",
            m.path_const_ident, m.path_value
        )
        .unwrap();
    }

    // 1. Synchronous Service Trait
    writeln!(out).unwrap();
    if methods.is_empty() {
        writeln!(out, "{inner_pad}pub trait {svc_trait_ident} {{}}").unwrap();
    } else {
        writeln!(out, "{inner_pad}pub trait {svc_trait_ident} {{").unwrap();
        for (idx, m) in methods.iter().enumerate() {
            if idx > 0 {
                writeln!(out).unwrap();
            }
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{fn_pad}fn {}<'a, 'b>(\n\
                         {body_pad}&self,\n\
                         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}req: {}<'a>,\n\
                         {body_pad}resp: &mut {}<'b>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}fn {}<'a, 'b>(\n\
                         {body_pad}&self,\n\
                         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}req: {}<'a>,\n\
                         {body_pad}tx: &mut impl ::roto_grpc::SendStream<{}<'b>>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}fn {}<'a, 'b>(\n\
                         {body_pad}&self,\n\
                         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}rx: &mut impl ::roto_grpc::RecvStream<{}<'a>>,\n\
                         {body_pad}resp: &mut {}<'b>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}fn {}<'a, 'b>(\n\
                         {body_pad}&self,\n\
                         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}rx: &mut impl ::roto_grpc::RecvStream<{}<'a>>,\n\
                         {body_pad}tx: &mut impl ::roto_grpc::SendStream<{}<'b>>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
            }
        }
        writeln!(out, "{inner_pad}}}").unwrap();
    }

    // 2. Asynchronous Service Trait
    writeln!(out).unwrap();
    writeln!(out, "{inner_pad}#[allow(async_fn_in_trait)]").unwrap();
    if methods.is_empty() {
        writeln!(out, "{inner_pad}pub trait {async_trait_ident} {{}}").unwrap();
    } else {
        writeln!(out, "{inner_pad}pub trait {async_trait_ident} {{").unwrap();
        for (idx, m) in methods.iter().enumerate() {
            if idx > 0 {
                writeln!(out).unwrap();
            }
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{fn_pad}async fn {}<'a, 'b>(\n\
                         {body_pad}&'a self,\n\
                         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}req: {}<'a>,\n\
                         {body_pad}resp: &'a mut {}<'b>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}async fn {}<'a, 'b>(\n\
                         {body_pad}&'a self,\n\
                         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}req: {}<'a>,\n\
                         {body_pad}tx: &'a mut impl ::roto_grpc::SendStream<{}<'b>>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}async fn {}<'a, 'r, 'b>(\n\
                         {body_pad}&'a self,\n\
                         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}rx: &'a mut impl ::roto_grpc::RecvStream<{}<'r>>,\n\
                         {body_pad}resp: &'a mut {}<'b>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}async fn {}<'a, 'r, 'b>(\n\
                         {body_pad}&'a self,\n\
                         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
                         {body_pad}rx: &'a mut impl ::roto_grpc::RecvStream<{}<'r>>,\n\
                         {body_pad}tx: &'a mut impl ::roto_grpc::SendStream<{}<'b>>,\n\
                         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;",
                        m.method_ident, m.input.view_ident_path, m.output.builder_ident_path
                    )
                    .unwrap();
                }
            }
        }
        writeln!(out, "{inner_pad}}}").unwrap();
    }

    // 3. Server Dispatcher `<Service>Server<T>`
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}#[derive(Clone, Copy, Debug)]\n\
         {inner_pad}pub struct {server_ident}<T> {{\n\
         {fn_pad}pub inner: T,\n\
         {inner_pad}}}\n\
         \n\
         {inner_pad}impl<T> {server_ident}<T> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub const fn new(inner: T) -> Self {{\n\
         {body_pad}Self {{ inner }}\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn into_inner(self) -> T {{\n\
         {body_pad}self.inner\n\
         {fn_pad}}}\n\
         {inner_pad}}}"
    )
    .unwrap();

    let single_req_methods: Vec<&ResolvedMethod> = methods
        .iter()
        .filter(|m| m.mode.is_single_request())
        .collect();
    let stream_req_methods: Vec<&ResolvedMethod> = methods
        .iter()
        .filter(|m| m.mode.is_client_streaming())
        .collect();
    let bidi_methods: Vec<&ResolvedMethod> = methods
        .iter()
        .filter(|m| m.mode.is_bidi_streaming())
        .collect();

    // Sync GrpcService implementation + inherent convenience methods
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {svc_trait_ident}> {server_ident}<T> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn call_unary(\n\
         {body_pad}&self,\n\
         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::call_unary(self, meta, method_name, req_payload, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn call_stream(\n\
         {body_pad}&self,\n\
         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &str,\n\
         {body_pad}req_frames: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::call_stream(self, meta, method_name, req_frames, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn dispatch_path(\n\
         {body_pad}&self,\n\
         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}path: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::dispatch_path(self, meta, path, req_payload, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn dispatch_stream_path(\n\
         {body_pad}&self,\n\
         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}path: &str,\n\
         {body_pad}req_frames: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::dispatch_stream_path(self, meta, path, req_frames, writer)\n\
         {fn_pad}}}"
    )
    .unwrap();
    if ctx.config.http {
        writeln!(
            out,
            "\n\
             {fn_pad}#[inline]\n\
             {fn_pad}pub fn call_http<'a, 's, 'b>(\n\
             {body_pad}&self,\n\
             {body_pad}req: &::roto_grpc::http::HttpRequest<'a>,\n\
             {body_pad}scratch: &'s mut [u8],\n\
             {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'b>,\n\
             {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>> {{\n\
             {body_pad}<Self as ::roto_grpc::http::HttpService>::call_http(self, req, scratch, writer)\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{inner_pad}}}").unwrap();

    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {svc_trait_ident}> ::roto_grpc::GrpcService for {server_ident}<T> {{\n\
         {fn_pad}const SERVICE_NAME: &'static str = SERVICE_NAME;"
    )
    .unwrap();
    if ctx.config.inner.descriptors {
        writeln!(
            out,
            "{fn_pad}const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = FILE_DESCRIPTOR_SET;"
        )
        .unwrap();
    }
    if !stream_req_methods.is_empty() {
        let pattern = stream_req_methods
            .iter()
            .map(|m| format!("{:?}", m.raw_name))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(
            out,
            "\n\
             {fn_pad}fn is_client_streaming(&self, method_name: &str) -> bool {{\n\
             {body_pad}matches!(method_name, {pattern})\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    if !bidi_methods.is_empty() {
        let pattern = bidi_methods
            .iter()
            .map(|m| format!("{:?}", m.raw_name))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(
            out,
            "\n\
             {fn_pad}fn is_bidi_streaming(&self, method_name: &str) -> bool {{\n\
             {body_pad}matches!(method_name, {pattern})\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(
        out,
        "\n\
         {fn_pad}fn call_unary(\n\
         {body_pad}&self,\n\
         {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{"
    )
    .unwrap();
    if single_req_methods.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (meta, method_name, req_payload, writer);\n\
             {body_pad}Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\"))"
        )
        .unwrap();
    } else {
        writeln!(out, "{body_pad}match method_name {{").unwrap();
        for m in &single_req_methods {
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let req_view = {}::from_bytes(req_payload)\n\
                         {arm_pad}        .map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf request payload\"))?;\n\
                         {arm_pad}    let end_pos = writer.cursor();\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut resp = {}::from_writer(taken);\n\
                         {arm_pad}    let res = self.inner.{}(meta, req_view, &mut resp);\n\
                         {arm_pad}    *writer = resp.writer;\n\
                         {arm_pad}    res?;\n\
                         {arm_pad}    ::roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)\n\
                         {arm_pad}        .map_err(::roto_grpc::GrpcStatus::from)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let req_view = {}::from_bytes(req_payload)\n\
                         {arm_pad}        .map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf request payload\"))?;\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    self.inner.{}(meta, req_view, &mut tx)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
        writeln!(
            out,
            "{arm_pad}_ => Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\")),\n\
             {body_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{fn_pad}}}").unwrap();
    if !stream_req_methods.is_empty() {
        writeln!(
            out,
            "\n\
             {fn_pad}fn call_stream(\n\
             {body_pad}&self,\n\
             {body_pad}meta: &mut ::roto_grpc::RpcMetadata<'_>,\n\
             {body_pad}method_name: &str,\n\
             {body_pad}req_frames: &[u8],\n\
             {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
             {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}match method_name {{"
        )
        .unwrap();
        for m in &stream_req_methods {
            match m.mode {
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let mut rx = ::roto_grpc::LpmRecvStream::new(req_frames, {}::from_bytes);\n\
                         {arm_pad}    let end_pos = writer.cursor();\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut resp = {}::from_writer(taken);\n\
                         {arm_pad}    let res = self.inner.{}(meta, &mut rx, &mut resp);\n\
                         {arm_pad}    *writer = resp.writer;\n\
                         {arm_pad}    res?;\n\
                         {arm_pad}    ::roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)\n\
                         {arm_pad}        .map_err(::roto_grpc::GrpcStatus::from)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let mut rx = ::roto_grpc::LpmRecvStream::new(req_frames, {}::from_bytes);\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    self.inner.{}(meta, &mut rx, &mut tx)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
        writeln!(
            out,
            "{arm_pad}_ => Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\")),\n\
             {body_pad}}}\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    if ctx.config.http {
        writeln!(
            out,
            "\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn call_http<'a, 's, 'b>(\n\
             {body_pad}&self,\n\
             {body_pad}req: &::roto_grpc::http::HttpRequest<'a>,\n\
             {body_pad}scratch: &'s mut [u8],\n\
             {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'b>,\n\
             {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>> {{\n\
             {body_pad}<Self as ::roto_grpc::http::HttpService>::call_http(self, req, scratch, writer)\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{inner_pad}}}").unwrap();

    // AsyncGrpcService implementation + inherent convenience methods
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {async_trait_ident}> {server_ident}<T> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn call_unary_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::call_unary_async(self, meta, method_name, req_payload, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn call_stream_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_frames: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::call_stream_async(self, meta, method_name, req_frames, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn dispatch_path_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}path: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::dispatch_path_async(self, meta, path, req_payload, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn dispatch_stream_path_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}path: &'a str,\n\
         {body_pad}req_frames: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::dispatch_stream_path_async(self, meta, path, req_frames, writer).await\n\
         {fn_pad}}}"
    )
    .unwrap();
    if ctx.config.http {
        writeln!(
            out,
            "\n\
             {fn_pad}#[inline]\n\
             {fn_pad}pub async fn call_http_async<'a, 's, 'b>(\n\
             {body_pad}&'a self,\n\
             {body_pad}req: &'a ::roto_grpc::http::HttpRequest<'a>,\n\
             {body_pad}scratch: &'s mut [u8],\n\
             {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
             {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>> {{\n\
             {body_pad}<Self as ::roto_grpc::http::AsyncHttpService>::call_http(self, req, scratch, writer).await\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{inner_pad}}}").unwrap();

    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {async_trait_ident}> ::roto_grpc::AsyncGrpcService for {server_ident}<T> {{\n\
         {fn_pad}const SERVICE_NAME: &'static str = SERVICE_NAME;"
    )
    .unwrap();
    if ctx.config.inner.descriptors {
        writeln!(
            out,
            "{fn_pad}const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = FILE_DESCRIPTOR_SET;"
        )
        .unwrap();
    }
    if !stream_req_methods.is_empty() {
        let pattern = stream_req_methods
            .iter()
            .map(|m| format!("{:?}", m.raw_name))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(
            out,
            "\n\
             {fn_pad}fn is_client_streaming(&self, method_name: &str) -> bool {{\n\
             {body_pad}matches!(method_name, {pattern})\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    if !bidi_methods.is_empty() {
        let pattern = bidi_methods
            .iter()
            .map(|m| format!("{:?}", m.raw_name))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(
            out,
            "\n\
             {fn_pad}fn is_bidi_streaming(&self, method_name: &str) -> bool {{\n\
             {body_pad}matches!(method_name, {pattern})\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(
        out,
        "\n\
         {fn_pad}async fn call_unary_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{"
    )
    .unwrap();
    if single_req_methods.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (meta, method_name, req_payload, writer);\n\
             {body_pad}Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\"))"
        )
        .unwrap();
    } else {
        writeln!(out, "{body_pad}match method_name {{").unwrap();
        for m in &single_req_methods {
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let req_view = {}::from_bytes(req_payload)\n\
                         {arm_pad}        .map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf request payload\"))?;\n\
                         {arm_pad}    let end_pos = writer.cursor();\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut resp = {}::from_writer(taken);\n\
                         {arm_pad}    let res = self.inner.{}(meta, req_view, &mut resp).await;\n\
                         {arm_pad}    *writer = resp.writer;\n\
                         {arm_pad}    res?;\n\
                         {arm_pad}    ::roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)\n\
                         {arm_pad}        .map_err(::roto_grpc::GrpcStatus::from)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let req_view = {}::from_bytes(req_payload)\n\
                         {arm_pad}        .map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf request payload\"))?;\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    self.inner.{}(meta, req_view, &mut tx).await\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
        writeln!(
            out,
            "{arm_pad}_ => Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\")),\n\
             {body_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{fn_pad}}}").unwrap();
    if !stream_req_methods.is_empty() {
        writeln!(
            out,
            "\n\
             {fn_pad}async fn call_stream_async<'a, 'b>(\n\
             {body_pad}&'a self,\n\
             {body_pad}meta: &'a mut ::roto_grpc::RpcMetadata<'_>,\n\
             {body_pad}method_name: &'a str,\n\
             {body_pad}req_frames: &'a [u8],\n\
             {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
             {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}match method_name {{"
        )
        .unwrap();
        for m in &stream_req_methods {
            match m.mode {
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let mut rx = ::roto_grpc::LpmRecvStream::new(req_frames, {}::from_bytes);\n\
                         {arm_pad}    let end_pos = writer.cursor();\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut resp = {}::from_writer(taken);\n\
                         {arm_pad}    let res = self.inner.{}(meta, &mut rx, &mut resp).await;\n\
                         {arm_pad}    *writer = resp.writer;\n\
                         {arm_pad}    res?;\n\
                         {arm_pad}    ::roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)\n\
                         {arm_pad}        .map_err(::roto_grpc::GrpcStatus::from)\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{arm_pad}{:?} => {{\n\
                         {arm_pad}    let mut rx = ::roto_grpc::LpmRecvStream::new(req_frames, {}::from_bytes);\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    self.inner.{}(meta, &mut rx, &mut tx).await\n\
                         {arm_pad}}}",
                        m.raw_name, m.input.view_ident_path, m.output.builder_ident_path, m.method_ident
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
        writeln!(
            out,
            "{arm_pad}_ => Err(::roto_grpc::GrpcStatus::unimplemented(\"unimplemented gRPC method\")),\n\
             {body_pad}}}\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    if ctx.config.http {
        writeln!(
            out,
            "\n\
             {fn_pad}#[inline]\n\
             {fn_pad}async fn call_http_async<'a, 's, 'b>(\n\
             {body_pad}&'a self,\n\
             {body_pad}req: &'a ::roto_grpc::http::HttpRequest<'a>,\n\
             {body_pad}scratch: &'s mut [u8],\n\
             {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
             {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>>\n\
             {fn_pad}where\n\
             {body_pad}'s: 'a,\n\
             {fn_pad}{{\n\
             {body_pad}<Self as ::roto_grpc::http::AsyncHttpService>::call_http(self, req, scratch, writer).await\n\
             {fn_pad}}}"
        )
        .unwrap();
    }
    writeln!(out, "{inner_pad}}}").unwrap();

    if ctx.config.http {
        render_http_service_impls(
            &methods,
            &svc_trait_ident,
            &async_trait_ident,
            &server_ident,
            ctx,
            indent + 1,
            out,
        )?;
    }

    // 4. Request traits for each single-request method (Unary & ServerStreaming)
    for m in &single_req_methods {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{inner_pad}pub trait {}<'a> {{\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>>;\n\
             {inner_pad}}}\n\
             \n\
             {inner_pad}impl<'a> {}<'a> for {}<'a> {{\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}self.finish().map_err(::roto_grpc::GrpcStatus::from)\n\
             {fn_pad}}}\n\
             {inner_pad}}}\n\
             \n\
             {inner_pad}impl<'a> {}<'a> for &'a {}<'_> {{\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}self.check().map_err(::roto_grpc::GrpcStatus::from)?;\n\
             {body_pad}Ok(self.writer().as_written_slice())\n\
             {fn_pad}}}\n\
             {inner_pad}}}\n\
             \n\
             {inner_pad}impl<'a> {}<'a> for {}<'a> {{\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}Ok(self.as_bytes())\n\
             {fn_pad}}}\n\
             {inner_pad}}}\n\
             \n\
             {inner_pad}impl<'a> {}<'a> for &'a {}<'_> {{\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}Ok(self.as_bytes())\n\
             {fn_pad}}}\n\
             {inner_pad}}}\n\
             \n\
             {inner_pad}impl<'a> {}<'a> for &'a [u8] {{\n\
             {fn_pad}#[inline]\n\
             {fn_pad}fn into_grpc_request_bytes(self) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {{\n\
             {body_pad}Ok(self)\n\
             {fn_pad}}}\n\
             {inner_pad}}}",
            m.req_trait_ident,
            m.req_trait_ident,
            m.input.builder_ident_path,
            m.req_trait_ident,
            m.input.builder_ident_path,
            m.req_trait_ident,
            m.input.view_ident_path,
            m.req_trait_ident,
            m.input.view_ident_path,
            m.req_trait_ident,
        )
        .unwrap();
    }

    // 5. Client Stub `<Service>Client<C>`
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}#[derive(Clone, Copy, Debug)]\n\
         {inner_pad}pub struct {client_ident}<C> {{\n\
         {fn_pad}pub channel: C,\n\
         {inner_pad}}}\n\
         \n\
         {inner_pad}impl<C> {client_ident}<C> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub const fn new(channel: C) -> Self {{\n\
         {body_pad}Self {{ channel }}\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn into_inner(self) -> C {{\n\
         {body_pad}self.channel\n\
         {fn_pad}}}\n\
         {inner_pad}}}\n\
         \n\
         {inner_pad}impl<C: ::roto_grpc::GrpcChannel> {client_ident}<C> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn request_metadata_mut(&mut self) -> Option<&mut dyn ::roto_grpc::MetadataMap> {{\n\
         {body_pad}self.channel.request_metadata_mut()\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn insert_metadata(\n\
         {body_pad}&mut self,\n\
         {body_pad}name: &str,\n\
         {body_pad}value: &str,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}let meta = self.channel.request_metadata_mut().ok_or_else(|| {{\n\
         {arm_pad}::roto_grpc::GrpcStatus::unimplemented(\"channel does not support request metadata\")\n\
         {body_pad}}})?;\n\
         {body_pad}meta.insert(name, value).map_err(::roto_grpc::GrpcStatus::from)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn insert_metadata_bytes(\n\
         {body_pad}&mut self,\n\
         {body_pad}name: &[u8],\n\
         {body_pad}value: &[u8],\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}let meta = self.channel.request_metadata_mut().ok_or_else(|| {{\n\
         {arm_pad}::roto_grpc::GrpcStatus::unimplemented(\"channel does not support request metadata\")\n\
         {body_pad}}})?;\n\
         {body_pad}meta.insert_bytes(name, value).map_err(::roto_grpc::GrpcStatus::from)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn response_headers(&self) -> &dyn ::roto_grpc::MetadataMap {{\n\
         {body_pad}self.channel.response_headers()\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn response_trailers(&self) -> &dyn ::roto_grpc::MetadataMap {{\n\
         {body_pad}self.channel.response_trailers()\n\
         {fn_pad}}}\n\
         {inner_pad}}}"
    )
    .unwrap();

    if !methods.is_empty() {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{inner_pad}impl<C: ::roto_grpc::UnaryChannel> {client_ident}<C> {{"
        )
        .unwrap();
        for (idx, m) in methods.iter().enumerate() {
            if idx > 0 {
                writeln!(out).unwrap();
            }
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{fn_pad}pub fn {}<'a, 'r, R>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}req: R,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}R: {}<'r>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let req_bytes = match req.into_grpc_request_bytes() {{\n\
                         {arm_pad}Ok(bytes) => bytes,\n\
                         {arm_pad}Err(err) => {{\n\
                         {arm_pad}    if let Some(meta) = self.channel.request_metadata_mut() {{\n\
                         {arm_pad}        meta.clear();\n\
                         {arm_pad}    }}\n\
                         {arm_pad}    return Err(err);\n\
                         {arm_pad}}}\n\
                         {body_pad}}};\n\
                         {body_pad}let resp_bytes = self.channel.call_unary_bytes({}, req_bytes)?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}\n\
                         \n\
                         {fn_pad}pub fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}build_req: F,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: FnOnce(&mut {}<'_>),\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self.channel.call_unary({}, |writer| {{\n\
                         {arm_pad}let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}let mut req = {}::from_writer(taken);\n\
                         {arm_pad}build_req(&mut req);\n\
                         {arm_pad}*writer = req.writer;\n\
                         {body_pad}}})?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}",
                        m.method_ident,
                        m.output.view_ident_path,
                        m.req_trait_ident,
                        m.path_const_ident,
                        m.output.view_ident_path,
                        m.with_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub fn {}<'a, 'r, R>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}req: R,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}R: {}<'r>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let req_bytes = match req.into_grpc_request_bytes() {{\n\
                         {arm_pad}Ok(bytes) => bytes,\n\
                         {arm_pad}Err(err) => {{\n\
                         {arm_pad}    if let Some(meta) = self.channel.request_metadata_mut() {{\n\
                         {arm_pad}        meta.clear();\n\
                         {arm_pad}    }}\n\
                         {arm_pad}    return Err(err);\n\
                         {arm_pad}}}\n\
                         {body_pad}}};\n\
                         {body_pad}let resp_bytes = self.channel.call_server_streaming_bytes({}, req_bytes)?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}\n\
                         \n\
                         {fn_pad}pub fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}build_req: F,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: FnOnce(&mut {}<'_>),\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self.channel.call_server_streaming({}, |writer| {{\n\
                         {arm_pad}let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}let mut req = {}::from_writer(taken);\n\
                         {arm_pad}build_req(&mut req);\n\
                         {arm_pad}*writer = req.writer;\n\
                         {body_pad}}})?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}",
                        m.method_ident,
                        m.output.view_ident_path,
                        m.req_trait_ident,
                        m.path_const_ident,
                        m.output.view_ident_path,
                        m.with_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}send_reqs: F,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: for<'w, 'b> FnOnce(\n\
                         {arm_pad}&mut ::roto_grpc::WriterSendStream<'w, 'b, {}<'b>>,\n\
                         {body_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self.channel.call_client_streaming({}, |writer| {{\n\
                         {arm_pad}let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}    writer,\n\
                         {arm_pad}    {}::from_writer,\n\
                         {arm_pad}    |b| b.writer,\n\
                         {arm_pad});\n\
                         {arm_pad}send_reqs(&mut tx)\n\
                         {body_pad}}})?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}",
                        m.method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}send_reqs: F,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: for<'w, 'b> FnOnce(\n\
                         {arm_pad}&mut ::roto_grpc::WriterSendStream<'w, 'b, {}<'b>>,\n\
                         {body_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self.channel.call_bidi_streaming({}, |writer| {{\n\
                         {arm_pad}let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}    writer,\n\
                         {arm_pad}    {}::from_writer,\n\
                         {arm_pad}    |b| b.writer,\n\
                         {arm_pad});\n\
                         {arm_pad}send_reqs(&mut tx)\n\
                         {body_pad}}})?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}",
                        m.method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
            }
        }
        writeln!(out, "{inner_pad}}}").unwrap();

        writeln!(out).unwrap();
        writeln!(
            out,
            "{inner_pad}impl<C: ::roto_grpc::AsyncUnaryChannel> {client_ident}<C> {{"
        )
        .unwrap();
        for (idx, m) in methods.iter().enumerate() {
            if idx > 0 {
                writeln!(out).unwrap();
            }
            match m.mode {
                StreamingMode::Unary => {
                    writeln!(
                        out,
                        "{fn_pad}pub async fn {}<'a, R>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}req: R,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}R: {}<'a>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let req_bytes = match req.into_grpc_request_bytes() {{\n\
                         {arm_pad}Ok(bytes) => bytes,\n\
                         {arm_pad}Err(err) => {{\n\
                         {arm_pad}    if let Some(meta) = self.channel.request_metadata_mut() {{\n\
                         {arm_pad}        meta.clear();\n\
                         {arm_pad}    }}\n\
                         {arm_pad}    return Err(err);\n\
                         {arm_pad}}}\n\
                         {body_pad}}};\n\
                         {body_pad}let resp_bytes = self.channel.call_unary_bytes_async({}, req_bytes).await?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}\n\
                         \n\
                         {fn_pad}pub async fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}build_req: F,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: FnOnce(&mut {}<'_>) + 'a,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self\n\
                         {arm_pad}.channel\n\
                         {arm_pad}.call_unary_async({}, |writer| {{\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut req = {}::from_writer(taken);\n\
                         {arm_pad}    build_req(&mut req);\n\
                         {arm_pad}    *writer = req.writer;\n\
                         {arm_pad}}})\n\
                         {arm_pad}.await?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}",
                        m.async_method_ident,
                        m.output.view_ident_path,
                        m.req_trait_ident,
                        m.path_const_ident,
                        m.output.view_ident_path,
                        m.with_async_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ServerStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub async fn {}<'a, R>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}req: R,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}R: {}<'a>,\n\
                         {fn_pad}{{\n\
                         {body_pad}let req_bytes = match req.into_grpc_request_bytes() {{\n\
                         {arm_pad}Ok(bytes) => bytes,\n\
                         {arm_pad}Err(err) => {{\n\
                         {arm_pad}    if let Some(meta) = self.channel.request_metadata_mut() {{\n\
                         {arm_pad}        meta.clear();\n\
                         {arm_pad}    }}\n\
                         {arm_pad}    return Err(err);\n\
                         {arm_pad}}}\n\
                         {body_pad}}};\n\
                         {body_pad}let resp_bytes = self\n\
                         {arm_pad}.channel\n\
                         {arm_pad}.call_server_streaming_bytes_async({}, req_bytes)\n\
                         {arm_pad}.await?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}\n\
                         \n\
                         {fn_pad}pub async fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}build_req: F,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: FnOnce(&mut {}<'_>) + 'a,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self\n\
                         {arm_pad}.channel\n\
                         {arm_pad}.call_server_streaming_async({}, |writer| {{\n\
                         {arm_pad}    let taken = ::roto_grpc::framing::take_writer(writer);\n\
                         {arm_pad}    let mut req = {}::from_writer(taken);\n\
                         {arm_pad}    build_req(&mut req);\n\
                         {arm_pad}    *writer = req.writer;\n\
                         {arm_pad}}})\n\
                         {arm_pad}.await?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}",
                        m.async_method_ident,
                        m.output.view_ident_path,
                        m.req_trait_ident,
                        m.path_const_ident,
                        m.output.view_ident_path,
                        m.with_async_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::ClientStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub async fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}send_reqs: F,\n\
                         {fn_pad}) -> Result<{}<'a>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: for<'w, 'b> FnOnce(\n\
                         {arm_pad}&mut ::roto_grpc::WriterSendStream<'w, 'b, {}<'b>>,\n\
                         {body_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>\n\
                         {arm_pad}+ 'a,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self\n\
                         {arm_pad}.channel\n\
                         {arm_pad}.call_client_streaming_async({}, |writer| {{\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    send_reqs(&mut tx)\n\
                         {arm_pad}}})\n\
                         {arm_pad}.await?;\n\
                         {body_pad}{}::from_bytes(resp_bytes)\n\
                         {arm_pad}.map_err(|_| ::roto_grpc::GrpcStatus::internal(\"invalid protobuf response payload\"))\n\
                         {fn_pad}}}",
                        m.async_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
                StreamingMode::BidiStreaming => {
                    writeln!(
                        out,
                        "{fn_pad}pub async fn {}<'a, F>(\n\
                         {body_pad}&'a mut self,\n\
                         {body_pad}send_reqs: F,\n\
                         {fn_pad}) -> Result<::roto_grpc::LpmRecvStream<'a, {}<'a>>, ::roto_grpc::GrpcStatus<'a>>\n\
                         {fn_pad}where\n\
                         {body_pad}F: for<'w, 'b> FnOnce(\n\
                         {arm_pad}&mut ::roto_grpc::WriterSendStream<'w, 'b, {}<'b>>,\n\
                         {body_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>>\n\
                         {arm_pad}+ 'a,\n\
                         {fn_pad}{{\n\
                         {body_pad}let resp_bytes = self\n\
                         {arm_pad}.channel\n\
                         {arm_pad}.call_bidi_streaming_async({}, |writer| {{\n\
                         {arm_pad}    let mut tx = ::roto_grpc::WriterSendStream::new(\n\
                         {arm_pad}        writer,\n\
                         {arm_pad}        {}::from_writer,\n\
                         {arm_pad}        |b| b.writer,\n\
                         {arm_pad}    );\n\
                         {arm_pad}    send_reqs(&mut tx)\n\
                         {arm_pad}}})\n\
                         {arm_pad}.await?;\n\
                         {body_pad}Ok(::roto_grpc::LpmRecvStream::new(resp_bytes, {}::from_bytes))\n\
                         {fn_pad}}}",
                        m.async_method_ident,
                        m.output.view_ident_path,
                        m.input.builder_ident_path,
                        m.path_const_ident,
                        m.input.builder_ident_path,
                        m.output.view_ident_path
                    )
                    .unwrap();
                }
            }
        }
        writeln!(out, "{inner_pad}}}").unwrap();
    }

    writeln!(out, "{pad}}}").unwrap();
    writeln!(
        out,
        "{pad}pub use {svc_mod_ident}::{{{async_trait_ident}, {client_ident}, {server_ident}, {svc_trait_ident}}};"
    )
    .unwrap();

    Ok(())
}

#[derive(Default)]
struct BindingNode {
    path_vars: Vec<(usize, ResolvedFieldStep)>,
    query_params: Vec<(String, ResolvedFieldStep)>,
    submessages: BTreeMap<u32, SubmessageBindingNode>,
}

struct SubmessageBindingNode {
    step: ResolvedFieldStep,
    is_body_target: bool,
    has_path_var_descendant: bool,
    node: BindingNode,
}

impl BindingNode {
    fn insert_path_var(&mut self, pv_idx: usize, steps: &[ResolvedFieldStep]) {
        let Some((first, rest)) = steps.split_first() else {
            return;
        };
        if rest.is_empty() {
            self.path_vars.push((pv_idx, first.clone()));
        } else {
            let entry =
                self.submessages
                    .entry(first.tag)
                    .or_insert_with(|| SubmessageBindingNode {
                        step: first.clone(),
                        is_body_target: false,
                        has_path_var_descendant: false,
                        node: BindingNode::default(),
                    });
            entry.has_path_var_descendant = true;
            entry.node.insert_path_var(pv_idx, rest);
        }
    }

    fn insert_query_param(&mut self, dotted_key: String, steps: &[ResolvedFieldStep]) {
        let Some((first, rest)) = steps.split_first() else {
            return;
        };
        if rest.is_empty() {
            self.query_params.push((dotted_key, first.clone()));
        } else {
            let entry =
                self.submessages
                    .entry(first.tag)
                    .or_insert_with(|| SubmessageBindingNode {
                        step: first.clone(),
                        is_body_target: false,
                        has_path_var_descendant: false,
                        node: BindingNode::default(),
                    });
            entry.node.insert_query_param(dotted_key, rest);
        }
    }
}

fn build_binding_tree(
    binding: &ValidatedHttpBinding,
    index: &MessageIndex<'_>,
) -> Result<(BindingNode, Option<ResolvedFieldStep>), String> {
    let mut root = BindingNode::default();
    let mut bound_path_keys = BTreeSet::new();

    for (pv_idx, pv) in binding.path_variables.iter().enumerate() {
        bound_path_keys.insert(pv.variable.dotted_path());
        root.insert_path_var(pv_idx, &pv.steps);
    }

    let mut root_non_msg_body = None;
    if let ValidatedBody::Field(ref body_step) = binding.body {
        if body_step.proto_type == FieldType::TypeMessage {
            let entry =
                root.submessages
                    .entry(body_step.tag)
                    .or_insert_with(|| SubmessageBindingNode {
                        step: body_step.clone(),
                        is_body_target: true,
                        has_path_var_descendant: false,
                        node: BindingNode::default(),
                    });
            entry.is_body_target = true;
        } else {
            root_non_msg_body = Some(body_step.clone());
        }
    }

    if binding.body != ValidatedBody::Wildcard {
        let query_chains = index.query_bindable_fields(&binding.input_fq)?;
        for chain in query_chains {
            if chain.is_empty() {
                continue;
            }
            if let ValidatedBody::Field(ref body_step) = binding.body {
                if chain[0].name == body_step.name {
                    continue;
                }
            }
            let dotted = chain
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join(".");
            if bound_path_keys.contains(&dotted) {
                continue;
            }
            root.insert_query_param(dotted, &chain);
        }
    }

    Ok((root, root_non_msg_body))
}

fn render_http_service_impls(
    methods: &[ResolvedMethod],
    svc_trait_ident: &str,
    async_trait_ident: &str,
    server_ident: &str,
    ctx: RenderContext<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let index = ctx
        .message_index
        .ok_or_else(|| "internal error: missing MessageIndex when http=true".to_string())?;

    let mut routes: Vec<(&ResolvedMethod, &ValidatedHttpBinding)> = Vec::new();
    for m in methods {
        if let Some(ref rule) = m.http_rule {
            for binding in rule.all_bindings() {
                routes.push((m, binding));
            }
        }
    }

    let inner_pad = "    ".repeat(indent);
    let fn_pad = "    ".repeat(indent + 1);
    let body_pad = "    ".repeat(indent + 2);

    // 1. Sync HttpService implementation
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {svc_trait_ident}> ::roto_grpc::http::HttpService for {server_ident}<T> {{\n\
         {fn_pad}fn call_http<'a, 's, 'b>(\n\
         {body_pad}&self,\n\
         {body_pad}req: &::roto_grpc::http::HttpRequest<'a>,\n\
         {body_pad}scratch: &'s mut [u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>> {{"
    )
    .unwrap();
    if routes.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (req, scratch, writer);\n\
             {body_pad}None"
        )
        .unwrap();
    } else {
        for (m, binding) in &routes {
            render_http_route_dispatch(m, binding, index, false, indent + 2, out)?;
        }
        writeln!(out, "{body_pad}let _ = (scratch, writer);").unwrap();
        writeln!(out, "{body_pad}None").unwrap();
    }
    writeln!(out, "{fn_pad}}}\n{inner_pad}}}").unwrap();

    // 2. AsyncHttpService implementation
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {async_trait_ident}> ::roto_grpc::http::AsyncHttpService for {server_ident}<T> {{\n\
         {fn_pad}async fn call_http<'a, 's, 'b>(\n\
         {body_pad}&self,\n\
         {body_pad}req: &::roto_grpc::http::HttpRequest<'a>,\n\
         {body_pad}scratch: &'s mut [u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> ::core::option::Option<Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>>> {{"
    )
    .unwrap();
    if routes.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (req, scratch, writer);\n\
             {body_pad}None"
        )
        .unwrap();
    } else {
        for (m, binding) in &routes {
            render_http_route_dispatch(m, binding, index, true, indent + 2, out)?;
        }
        writeln!(out, "{body_pad}let _ = (scratch, writer);").unwrap();
        writeln!(out, "{body_pad}None").unwrap();
    }
    writeln!(out, "{fn_pad}}}\n{inner_pad}}}").unwrap();

    Ok(())
}

fn render_http_route_dispatch(
    m: &ResolvedMethod,
    binding: &ValidatedHttpBinding,
    index: &MessageIndex<'_>,
    is_async: bool,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let pad = "    ".repeat(indent);
    let inner_pad = "    ".repeat(indent + 1);
    let match_pad = "    ".repeat(indent + 2);
    let exec_pad = "    ".repeat(indent + 3);

    let method_cond = match &binding.pattern {
        HttpPattern::Get(_) => "req.method == ::roto_grpc::http::HttpMethod::Get".to_string(),
        HttpPattern::Put(_) => "req.method == ::roto_grpc::http::HttpMethod::Put".to_string(),
        HttpPattern::Post(_) => "req.method == ::roto_grpc::http::HttpMethod::Post".to_string(),
        HttpPattern::Delete(_) => "req.method == ::roto_grpc::http::HttpMethod::Delete".to_string(),
        HttpPattern::Patch(_) => "req.method == ::roto_grpc::http::HttpMethod::Patch".to_string(),
        HttpPattern::Custom { kind, .. } => format!("req.method.matches({kind:?})"),
    };

    writeln!(out, "{pad}if {method_cond} {{").unwrap();

    let num_vars = binding.path_variables.len();
    if num_vars == 0 {
        writeln!(out, "{inner_pad}if (|| -> ::core::option::Option<()> {{").unwrap();
    } else {
        writeln!(
            out,
            "{inner_pad}if let Some(path_vars) = (|| -> ::core::option::Option<[&'a str; {num_vars}]> {{"
        )
        .unwrap();
    }

    let cursor_mut = if binding.path_template.segments.is_empty() {
        ""
    } else {
        "mut "
    };
    if let Some(ref verb) = binding.path_template.verb {
        writeln!(
            out,
            "{match_pad}let {cursor_mut}cursor = ::roto_grpc::http::PathCursor::with_custom_verb(req.path, {verb:?})?;"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "{match_pad}let {cursor_mut}cursor = ::roto_grpc::http::PathCursor::without_custom_verb(req.path)?;"
        )
        .unwrap();
    }

    let mut var_counter = 0usize;
    for seg in &binding.path_template.segments {
        match seg {
            PathSegment::Literal(lit) => {
                writeln!(
                    out,
                    "{match_pad}if !cursor.literal({lit:?}) {{\n\
                     {match_pad}    return None;\n\
                     {match_pad}}}"
                )
                .unwrap();
            }
            PathSegment::SingleWildcard => {
                writeln!(out, "{match_pad}cursor.capture_single()?;").unwrap();
            }
            PathSegment::MultiWildcard => {
                writeln!(out, "{match_pad}cursor.capture_remaining()?;").unwrap();
            }
            PathSegment::Variable(var) => {
                let v_idx = var_counter;
                var_counter += 1;
                if var.sub_template.as_slice() == [SubSegment::SingleWildcard] {
                    writeln!(
                        out,
                        "{match_pad}let var_{v_idx} = cursor.capture_single()?;"
                    )
                    .unwrap();
                } else if var.sub_template.as_slice() == [SubSegment::MultiWildcard] {
                    writeln!(
                        out,
                        "{match_pad}let var_{v_idx} = cursor.capture_remaining()?;"
                    )
                    .unwrap();
                } else {
                    writeln!(out, "{match_pad}let start_{v_idx} = cursor.remaining();").unwrap();
                    for sub in &var.sub_template {
                        match sub {
                            SubSegment::Literal(lit) => {
                                writeln!(
                                    out,
                                    "{match_pad}if !cursor.literal({lit:?}) {{\n\
                                     {match_pad}    return None;\n\
                                     {match_pad}}}"
                                )
                                .unwrap();
                            }
                            SubSegment::SingleWildcard => {
                                writeln!(out, "{match_pad}cursor.capture_single()?;").unwrap();
                            }
                            SubSegment::MultiWildcard => {
                                writeln!(out, "{match_pad}cursor.capture_remaining()?;").unwrap();
                            }
                        }
                    }
                    writeln!(
                        out,
                        "{match_pad}let var_{v_idx} = cursor.slice_since(start_{v_idx});"
                    )
                    .unwrap();
                }
            }
        }
    }

    writeln!(
        out,
        "{match_pad}if !cursor.is_end() {{\n\
         {match_pad}    return None;\n\
         {match_pad}}}"
    )
    .unwrap();

    if num_vars == 0 {
        writeln!(
            out,
            "{match_pad}Some(())\n\
             {inner_pad}}})().is_some() {{"
        )
        .unwrap();
    } else {
        let vars_list = (0..num_vars)
            .map(|i| format!("var_{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            out,
            "{match_pad}Some([{vars_list}])\n\
             {inner_pad}}})() {{"
        )
        .unwrap();
    }

    if is_async {
        writeln!(
            out,
            "{match_pad}let res: Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>> = async {{"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "{match_pad}let res: Result<::roto_grpc::http::HttpResponse<'b>, ::roto_grpc::GrpcStatus<'static>> = (|| {{"
        )
        .unwrap();
    }

    // Request decoding / construction
    let is_zero_copy_fast_path =
        binding.body == ValidatedBody::Wildcard && binding.path_variables.is_empty();
    if is_zero_copy_fast_path {
        writeln!(
            out,
            "{exec_pad}let _ = scratch;\n\
             {exec_pad}let req_view = {}::from_bytes(req.body)\n\
             {exec_pad}    .map_err(|_| ::roto_grpc::GrpcStatus::invalid_argument(\"invalid protobuf request payload\"))?;",
            m.input.view_ident_path
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "{exec_pad}let (decode_scratch, req_scratch) = scratch.split_at_mut(scratch.len() / 3);\n\
             {exec_pad}let _ = &decode_scratch;\n\
             {exec_pad}let mut req_writer = ::roto::runtime::BackwardWriter::new(req_scratch);\n\
             {exec_pad}let _ = &mut req_writer;"
        )
        .unwrap();
        if binding.body == ValidatedBody::Wildcard {
            writeln!(
                out,
                "{exec_pad}let _ = {}::from_bytes(req.body)\n\
                 {exec_pad}    .map_err(|_| ::roto_grpc::GrpcStatus::invalid_argument(\"invalid protobuf request payload\"))?;",
                m.input.view_ident_path
            )
            .unwrap();
        }
        let (tree, root_non_msg_body) = build_binding_tree(binding, index)?;
        let root_body_slice = if binding.body == ValidatedBody::Wildcard {
            Some("req.body")
        } else {
            None
        };
        let mut sub_counter = 0usize;
        render_binding_node(
            &tree,
            root_body_slice,
            root_non_msg_body.as_ref(),
            index,
            &mut sub_counter,
            indent + 3,
            out,
        );
        writeln!(
            out,
            "{exec_pad}let req_bytes = req_writer.finish().map_err(::roto_grpc::GrpcStatus::from)?;\n\
             {exec_pad}let req_view = {}::from_bytes(req_bytes)\n\
             {exec_pad}    .map_err(|_| ::roto_grpc::GrpcStatus::invalid_argument(\"invalid protobuf request payload\"))?;",
            m.input.view_ident_path
        )
        .unwrap();
    }

    let resp_field_arg = match &binding.response_body {
        ValidatedResponseBody::Empty => "None".to_string(),
        ValidatedResponseBody::Field(step) => format!("Some({})", step.tag),
    };
    let await_suffix = if is_async { ".await" } else { "" };
    writeln!(
        out,
        "{exec_pad}let taken = ::roto_grpc::framing::take_writer(writer);\n\
         {exec_pad}let mut resp = {}::from_writer(taken);\n\
         {exec_pad}let mut http_ctx = ::roto_grpc::RpcContext::<::roto_grpc::FixedMetadata>::new();\n\
         {exec_pad}let mut http_meta = http_ctx.metadata();\n\
         {exec_pad}let handler_res = self.inner.{}(&mut http_meta, req_view, &mut resp){await_suffix};\n\
         {exec_pad}*writer = resp.writer;\n\
         {exec_pad}handler_res?;\n\
         {exec_pad}::roto_grpc::http::finish_writer_response(writer, {resp_field_arg})",
        m.output.builder_ident_path, m.method_ident
    )
    .unwrap();

    if is_async {
        writeln!(
            out,
            "{match_pad}}}.await;\n\
             {match_pad}return Some(res);\n\
             {inner_pad}}}\n\
             {pad}}}"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "{match_pad}}})();\n\
             {match_pad}return Some(res);\n\
             {inner_pad}}}\n\
             {pad}}}"
        )
        .unwrap();
    }

    Ok(())
}

fn render_binding_node(
    node: &BindingNode,
    body_slice_expr: Option<&str>,
    root_non_msg_body: Option<&ResolvedFieldStep>,
    index: &MessageIndex<'_>,
    sub_counter: &mut usize,
    indent: usize,
    out: &mut String,
) {
    let pad = "    ".repeat(indent);

    // 1. Direct leaf path variables (written first to BackwardWriter -> last on wire -> highest precedence)
    for (pv_idx, leaf_step) in &node.path_vars {
        let raw_expr = format!("path_vars[{pv_idx}]");
        render_parse_and_write_leaf(leaf_step, &raw_expr, true, index, indent, out);
    }

    // 2. Child submessages
    for sub in node.submessages.values() {
        *sub_counter += 1;
        let sub_id = *sub_counter;
        let child_body_expr = if sub.is_body_target {
            Some("req.body".to_string())
        } else if let Some(parent_body) = body_slice_expr {
            if sub.has_path_var_descendant {
                writeln!(
                    out,
                    "{pad}let body_sub_{sub_id} = ::roto_grpc::http::extract_request_submessage_field({parent_body}, {})?;",
                    sub.step.tag
                )
                .unwrap();
                Some(format!("body_sub_{sub_id}"))
            } else {
                None
            }
        } else {
            None
        };

        writeln!(
            out,
            "{pad}let sub_prev_{sub_id} = req_writer.written_len();"
        )
        .unwrap();
        render_binding_node(
            &sub.node,
            child_body_expr.as_deref(),
            None,
            index,
            sub_counter,
            indent,
            out,
        );
        if sub.has_path_var_descendant || sub.is_body_target {
            writeln!(
                out,
                "{pad}::roto_grpc::http::wrap_nested_path(&mut req_writer, sub_prev_{sub_id}, &[{}])?;",
                sub.step.tag
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{pad}if req_writer.written_len() > sub_prev_{sub_id} {{\n\
                 {pad}    ::roto_grpc::http::wrap_nested_path(&mut req_writer, sub_prev_{sub_id}, &[{}])?;\n\
                 {pad}}}",
                sub.step.tag
            )
            .unwrap();
        }
    }

    // 3. Direct leaf query parameters (iterated in reverse so BackwardWriter preserves forward order)
    if !node.query_params.is_empty() {
        writeln!(
            out,
            "{pad}if !req.query.is_empty() {{\n\
             {pad}    for pair in req.query.strip_prefix('?').unwrap_or(req.query).rsplit('&') {{\n\
             {pad}        if pair.is_empty() {{\n\
             {pad}            continue;\n\
             {pad}        }}\n\
             {pad}        let (raw_k, raw_v) = match pair.split_once('=') {{\n\
             {pad}            Some((k, v)) => (k, v),\n\
             {pad}            None => (pair, \"\"),\n\
             {pad}        }};\n\
             {pad}        if raw_k.is_empty() {{\n\
             {pad}            continue;\n\
             {pad}        }}\n\
             {pad}        let q_idx = match ::roto_grpc::http::percent_decode_str(raw_k, decode_scratch)? {{"
        )
        .unwrap();
        for (q_idx, (dotted_key, _)) in node.query_params.iter().enumerate() {
            writeln!(
                out,
                "{pad}            {dotted_key:?} => Some({q_idx}usize),"
            )
            .unwrap();
        }
        writeln!(
            out,
            "{pad}            _ => None,\n\
             {pad}        }};\n\
             {pad}        match q_idx {{"
        )
        .unwrap();
        for (q_idx, (_, q_step)) in node.query_params.iter().enumerate() {
            writeln!(out, "{pad}            Some({q_idx}usize) => {{").unwrap();
            render_parse_and_write_leaf(q_step, "raw_v", true, index, indent + 4, out);
            writeln!(out, "{pad}            }}").unwrap();
        }
        writeln!(
            out,
            "{pad}            _ => {{}}\n\
             {pad}        }}\n\
             {pad}    }}\n\
             {pad}}}"
        )
        .unwrap();
    }

    // 4. Direct non-message `body: "<field>"` at root
    if let Some(body_step) = root_non_msg_body {
        if body_step.proto_type == FieldType::TypeBytes {
            writeln!(
                out,
                "{pad}::roto_grpc::http::write_bytes_field(&mut req_writer, {}, req.body)?;",
                body_step.tag
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{pad}let body_str = ::core::str::from_utf8(req.body)\n\
                 {pad}    .map_err(|_| ::roto_grpc::GrpcStatus::invalid_argument(\"invalid UTF-8 in request body\"))?;"
            )
            .unwrap();
            render_parse_and_write_leaf(body_step, "body_str", false, index, indent, out);
        }
    }

    // 5. Underlying message body bytes (written last to BackwardWriter -> first on wire -> overridden by path vars)
    if let Some(body_slice) = body_slice_expr {
        writeln!(
            out,
            "{pad}if !{body_slice}.is_empty() {{\n\
             {pad}    ::roto_grpc::http::write_validated_message_bytes(&mut req_writer, {body_slice})?;\n\
             {pad}}}"
        )
        .unwrap();
    }
}

fn render_parse_and_write_leaf(
    step: &ResolvedFieldStep,
    raw_str_expr: &str,
    needs_percent_decode: bool,
    index: &MessageIndex<'_>,
    indent: usize,
    out: &mut String,
) {
    let pad = "    ".repeat(indent);
    let tag = step.tag;

    if step.proto_type == FieldType::TypeBytes {
        if needs_percent_decode {
            writeln!(
                out,
                "{pad}{{\n\
                 {pad}    let decoded = ::roto_grpc::http::percent_decode_bytes({raw_str_expr}.as_bytes(), decode_scratch)?;\n\
                 {pad}    ::roto_grpc::http::write_bytes_field(&mut req_writer, {tag}, decoded)?;\n\
                 {pad}}}"
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{pad}::roto_grpc::http::write_bytes_field(&mut req_writer, {tag}, {raw_str_expr}.as_bytes())?;"
            )
            .unwrap();
        }
        return;
    }

    writeln!(out, "{pad}{{").unwrap();
    if needs_percent_decode {
        writeln!(
            out,
            "{pad}    let decoded = ::roto_grpc::http::percent_decode_str({raw_str_expr}, decode_scratch)?;"
        )
        .unwrap();
    } else {
        writeln!(out, "{pad}    let decoded = {raw_str_expr};").unwrap();
    }

    match step.proto_type {
        FieldType::TypeString => {
            writeln!(
                out,
                "{pad}    ::roto_grpc::http::write_str_field(&mut req_writer, {tag}, decoded)?;"
            )
            .unwrap();
        }
        FieldType::TypeBool => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_bool(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_bool_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeInt32 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_int32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeSint32 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_sint32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeSfixed32 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_sfixed32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeUint32 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_u32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_uint32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeFixed32 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_u32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_fixed32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeInt64 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_int64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeSint64 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_sint64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeSfixed64 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_i64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_sfixed64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeUint64 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_u64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_uint64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeFixed64 => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_u64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_fixed64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeFloat => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_f32(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_f32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeDouble => {
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_f64(decoded)?;\n\
                 {pad}    ::roto_grpc::http::write_f64_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeEnum => {
            let variants = index.enum_variants(&step.type_name).unwrap_or(&[]);
            writeln!(
                out,
                "{pad}    let val = ::roto_grpc::http::parse_enum(decoded, |s| match s {{"
            )
            .unwrap();
            for (v_name, v_num) in variants {
                writeln!(out, "{pad}        {v_name:?} => Some({v_num}i32),").unwrap();
            }
            writeln!(
                out,
                "{pad}        _ => None,\n\
                 {pad}    }})?;\n\
                 {pad}    ::roto_grpc::http::write_int32_field(&mut req_writer, {tag}, val)?;"
            )
            .unwrap();
        }
        FieldType::TypeBytes | FieldType::TypeMessage | FieldType::TypeGroup => unreachable!(),
    }
    writeln!(out, "{pad}}}").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grpc_generator_config_modes_files_and_extern_paths() {
        let cfg = GrpcGeneratorConfig::parse("").unwrap();
        assert_eq!(cfg.inner.mode, OutputMode::Lib);
        assert_eq!(cfg.output_file_name(), "lib.rs");
        assert!(!cfg.http);

        let cfg_mod = GrpcGeneratorConfig::parse("mode=mod").unwrap();
        assert_eq!(cfg_mod.inner.mode, OutputMode::Mod);
        assert_eq!(cfg_mod.output_file_name(), "grpc.rs");
        assert!(!cfg_mod.http);

        let cfg_http = GrpcGeneratorConfig::parse("http").unwrap();
        assert!(cfg_http.http);

        let cfg_http_true = GrpcGeneratorConfig::parse("mode=mod,http=true").unwrap();
        assert!(cfg_http_true.http);

        let cfg_http_false = GrpcGeneratorConfig::parse("http=true,http=false").unwrap();
        assert!(!cfg_http_false.http);

        let cfg_custom = GrpcGeneratorConfig::parse(
            "mode=mod,file=custom_grpc.rs,descriptors=true,http=true,extern_path=google/protobuf=::google_protobuf_roto",
        )
        .unwrap();
        assert_eq!(cfg_custom.output_file_name(), "custom_grpc.rs");
        assert!(cfg_custom.inner.descriptors);
        assert!(cfg_custom.http);
        assert_eq!(cfg_custom.inner.extern_paths.len(), 1);

        assert!(GrpcGeneratorConfig::parse("file=").is_err());
        assert!(GrpcGeneratorConfig::parse("file=../evil.rs").is_err());
        assert!(GrpcGeneratorConfig::parse("file=/abs/path.rs").is_err());
        assert!(GrpcGeneratorConfig::parse("http=maybe").is_err());
        assert!(GrpcGeneratorConfig::parse("unknown_opt=1").is_err());
    }

    #[test]
    fn emits_service_file_descriptor_bytes_and_transitive_set_when_opted_in() {
        use google_protobuf_roto_bootstrap::google::protobuf::compiler::CodeGeneratorRequestBuilder;

        let mut buf = vec![0u8; 8192];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);
        req.write_parameter("mode=mod,file=grpc.rs,descriptors=true")
            .write_proto_file(|fb| {
                fb.write_service(|svc| {
                    svc.write_method(|m| {
                        m.write_output_type(".telemetry.v1.Pong")
                            .write_input_type(".common.v1.Ping")
                            .write_name("Call");
                    })
                    .write_name("Telemetry");
                })
                .write_message_type(|msg| {
                    msg.write_name("Pong");
                })
                .write_dependency("common/v1/common.proto")
                .write_package("telemetry.v1")
                .write_name("telemetry/v1/telemetry.proto");
            })
            .write_proto_file(|fb| {
                fb.write_message_type(|msg| {
                    msg.write_name("Ping");
                })
                .write_package("common.v1")
                .write_name("common/v1/common.proto");
            })
            .write_file_to_generate("telemetry/v1/telemetry.proto");

        let req_view = CodeGeneratorRequestView::from_bytes(req.finish().unwrap()).unwrap();
        let mut fds = req_view.proto_file();
        let common_fd = fds.next().unwrap().unwrap();
        let telemetry_fd = fds.next().unwrap().unwrap();

        let files = generate(&req_view).unwrap();
        assert_eq!(files.len(), 1);
        let content = &files[0].content;

        let telemetry_lit = format_bytes_literal(telemetry_fd.as_bytes());
        let common_lit = format_bytes_literal(common_fd.as_bytes());

        assert!(content.contains(&format!(
            "pub const FILE_DESCRIPTOR_BYTES: &[u8] = {telemetry_lit};"
        )));
        assert!(content.contains(&format!(
            "pub const FILE_DESCRIPTOR_SET: &[&[u8]] = &[FILE_DESCRIPTOR_BYTES, {common_lit} as &[u8]];"
        )));
        assert!(content.contains(
            "const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = FILE_DESCRIPTOR_SET;"
        ));
    }
}
