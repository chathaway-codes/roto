use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::CodeGeneratorRequestView, FileDescriptorProtoView, ServiceDescriptorProtoView,
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
    /// - `extern_path=<proto_prefix>=<rust_crate_path>`
    /// - `error=<msg>`
    pub fn parse(parameter: &str) -> Result<Self, String> {
        let mut file = None;
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
            } else {
                delegated_parts.push(part);
            }
        }

        let inner = GeneratorConfig::parse(&delegated_parts.join(","))?;
        Ok(Self { inner, file })
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
}

/// Generates `#![no_std]` zero-copy gRPC service traits, `<Service>Server<T>` dispatchers,
/// and `<Service>Client<C>` stubs for all target files listed in `request.file_to_generate()`.
pub fn generate(request: &CodeGeneratorRequestView<'_>) -> Result<Vec<GeneratedFile>, String> {
    crate::validate_request(request)?;
    let config = GrpcGeneratorConfig::from_request(request)?;
    let symbols = SymbolTable::from_request(request)?;

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
         {body_pad}method_name: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::call_unary(self, method_name, req_payload, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn call_stream(\n\
         {body_pad}&self,\n\
         {body_pad}method_name: &str,\n\
         {body_pad}req_frames: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::call_stream(self, method_name, req_frames, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn dispatch_path(\n\
         {body_pad}&self,\n\
         {body_pad}path: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::dispatch_path(self, path, req_payload, writer)\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub fn dispatch_stream_path(\n\
         {body_pad}&self,\n\
         {body_pad}path: &str,\n\
         {body_pad}req_frames: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::GrpcService>::dispatch_stream_path(self, path, req_frames, writer)\n\
         {fn_pad}}}\n\
         {inner_pad}}}"
    )
    .unwrap();

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
         {body_pad}method_name: &str,\n\
         {body_pad}req_payload: &[u8],\n\
         {body_pad}writer: &mut ::roto::runtime::BackwardWriter<'_>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{"
    )
    .unwrap();
    if single_req_methods.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (method_name, req_payload, writer);\n\
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
                         {arm_pad}    let res = self.inner.{}(req_view, &mut resp);\n\
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
                         {arm_pad}    self.inner.{}(req_view, &mut tx)\n\
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
                         {arm_pad}    let res = self.inner.{}(&mut rx, &mut resp);\n\
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
                         {arm_pad}    self.inner.{}(&mut rx, &mut tx)\n\
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
    writeln!(out, "{inner_pad}}}").unwrap();

    // AsyncGrpcService implementation + inherent convenience methods
    writeln!(out).unwrap();
    writeln!(
        out,
        "{inner_pad}impl<T: {async_trait_ident}> {server_ident}<T> {{\n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn call_unary_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::call_unary_async(self, method_name, req_payload, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn call_stream_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_frames: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::call_stream_async(self, method_name, req_frames, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn dispatch_path_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}path: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::dispatch_path_async(self, path, req_payload, writer).await\n\
         {fn_pad}}}\n\
         \n\
         {fn_pad}#[inline]\n\
         {fn_pad}pub async fn dispatch_stream_path_async<'a, 'b>(\n\
         {body_pad}&'a self,\n\
         {body_pad}path: &'a str,\n\
         {body_pad}req_frames: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{\n\
         {body_pad}<Self as ::roto_grpc::AsyncGrpcService>::dispatch_stream_path_async(self, path, req_frames, writer).await\n\
         {fn_pad}}}\n\
         {inner_pad}}}"
    )
    .unwrap();

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
         {body_pad}method_name: &'a str,\n\
         {body_pad}req_payload: &'a [u8],\n\
         {body_pad}writer: &'a mut ::roto::runtime::BackwardWriter<'b>,\n\
         {fn_pad}) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {{"
    )
    .unwrap();
    if single_req_methods.is_empty() {
        writeln!(
            out,
            "{body_pad}let _ = (method_name, req_payload, writer);\n\
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
                         {arm_pad}    let res = self.inner.{}(req_view, &mut resp).await;\n\
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
                         {arm_pad}    self.inner.{}(req_view, &mut tx).await\n\
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
                         {arm_pad}    let res = self.inner.{}(&mut rx, &mut resp).await;\n\
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
                         {arm_pad}    self.inner.{}(&mut rx, &mut tx).await\n\
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
    writeln!(out, "{inner_pad}}}").unwrap();

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
                         {body_pad}let req_bytes = req.into_grpc_request_bytes()?;\n\
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
                         {body_pad}let req_bytes = req.into_grpc_request_bytes()?;\n\
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
                         {body_pad}let req_bytes = req.into_grpc_request_bytes()?;\n\
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
                         {body_pad}let req_bytes = req.into_grpc_request_bytes()?;\n\
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grpc_generator_config_modes_files_and_extern_paths() {
        let cfg = GrpcGeneratorConfig::parse("").unwrap();
        assert_eq!(cfg.inner.mode, OutputMode::Lib);
        assert_eq!(cfg.output_file_name(), "lib.rs");

        let cfg_mod = GrpcGeneratorConfig::parse("mode=mod").unwrap();
        assert_eq!(cfg_mod.inner.mode, OutputMode::Mod);
        assert_eq!(cfg_mod.output_file_name(), "grpc.rs");

        let cfg_custom = GrpcGeneratorConfig::parse(
            "mode=mod,file=custom_grpc.rs,descriptors=true,extern_path=google/protobuf=::google_protobuf_roto",
        )
        .unwrap();
        assert_eq!(cfg_custom.output_file_name(), "custom_grpc.rs");
        assert!(cfg_custom.inner.descriptors);
        assert_eq!(cfg_custom.inner.extern_paths.len(), 1);

        assert!(GrpcGeneratorConfig::parse("file=").is_err());
        assert!(GrpcGeneratorConfig::parse("file=../evil.rs").is_err());
        assert!(GrpcGeneratorConfig::parse("file=/abs/path.rs").is_err());
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
