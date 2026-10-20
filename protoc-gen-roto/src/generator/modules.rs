use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use google_protobuf_roto::google::protobuf::{
    compiler::CodeGeneratorRequestView, DescriptorProtoView, FileDescriptorProtoView,
};

use crate::GeneratedFile;

use super::builders::generate_builder;
use super::config::{GeneratorConfig, OutputMode};
use super::enums::{collect_message_oneofs, generate_enum, generate_oneof};
use super::ident::escape_mod_ident;
use super::symbols::SymbolTable;
use super::views::generate_view_only;

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

#[derive(Default)]
struct PackageNode<'a> {
    proto_scope: String,
    raw_segment: String,
    files: Vec<FileDescriptorProtoView<'a>>,
    children: BTreeMap<String, PackageNode<'a>>,
}

#[derive(Clone, Copy)]
struct RenderContext<'a> {
    symbols: &'a SymbolTable,
    config: &'a GeneratorConfig,
}

/// Generates `#![no_std]` Rust source files (`lib.rs` in `OutputMode::Lib` or `mod.rs` +
/// per-package module files in `OutputMode::Mod`) for all target files listed in
/// `request.file_to_generate()`.
pub fn generate(request: &CodeGeneratorRequestView<'_>) -> Result<Vec<GeneratedFile>, String> {
    crate::validate_request(request)?;
    let config = GeneratorConfig::from_request(request)?;
    let symbols = SymbolTable::from_request(request)?;
    let ctx = RenderContext {
        symbols: &symbols,
        config: &config,
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

    let mut target_files: Vec<FileDescriptorProtoView<'_>> = Vec::new();
    let mut seen_targets: BTreeSet<&str> = BTreeSet::new();
    for target_res in request.file_to_generate() {
        let target_name = target_res
            .map_err(|e| format!("invalid CodeGeneratorRequest.file_to_generate: {e}"))?;
        if !seen_targets.insert(target_name) {
            continue;
        }
        let file = proto_files_by_name.get(target_name).copied().ok_or_else(|| {
            format!("file_to_generate '{target_name}' not found in proto_file descriptors")
        })?;
        target_files.push(file);
    }

    if target_files.is_empty() {
        return Ok(Vec::new());
    }

    let root = build_package_tree(&target_files)?;

    match config.mode {
        OutputMode::Lib => {
            let mut body = String::new();
            render_package_node_inline(&root, ctx, 0, &mut body)?;

            let mut content = String::with_capacity(LIB_RS_HEADER.len() + 1 + body.len());
            content.push_str(LIB_RS_HEADER);
            if !body.is_empty() {
                content.push('\n');
                content.push_str(&body);
            }

            Ok(vec![GeneratedFile {
                name: "lib.rs".to_string(),
                content,
            }])
        }
        OutputMode::Mod => {
            let mut mod_rs_content = String::new();
            let mut module_files = Vec::new();

            render_node_direct_items(&root, ctx.symbols, ctx.config, 0, &mut mod_rs_content)?;

            for (mod_ident, child) in &root.children {
                if !mod_rs_content.is_empty() && !mod_rs_content.ends_with("\n\n") {
                    writeln!(mod_rs_content).unwrap();
                }
                render_mod_tree_child(
                    mod_ident,
                    child,
                    &[],
                    ctx,
                    0,
                    &mut mod_rs_content,
                    &mut module_files,
                )?;
            }

            let mut out_files = Vec::with_capacity(1 + module_files.len());
            out_files.push(GeneratedFile {
                name: "mod.rs".to_string(),
                content: mod_rs_content,
            });
            out_files.extend(module_files);
            Ok(out_files)
        }
    }
}

fn build_package_tree<'a>(
    target_files: &[FileDescriptorProtoView<'a>],
) -> Result<PackageNode<'a>, String> {
    let mut root = PackageNode::default();

    for &file in target_files {
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
        for seg in raw_pkg.split('.').filter(|s| !s.is_empty()) {
            write!(scope, ".{seg}").unwrap();
            let mod_ident = escape_mod_ident(seg);
            let raw_seg = mod_ident.strip_prefix("r#").unwrap_or(&mod_ident).to_string();
            curr = curr.children.entry(mod_ident).or_insert_with(|| PackageNode {
                proto_scope: scope.clone(),
                raw_segment: raw_seg,
                files: Vec::new(),
                children: BTreeMap::new(),
            });
        }
        curr.files.push(file);
    }

    Ok(root)
}

fn render_package_node_inline(
    node: &PackageNode<'_>,
    ctx: RenderContext<'_>,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let mut first = true;

    let mut direct_out = String::new();
    render_node_direct_items(node, ctx.symbols, ctx.config, indent, &mut direct_out)?;
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
        render_package_node_inline(child, ctx, indent + 1, out)?;
        writeln!(out, "{pad}}}").unwrap();
    }

    Ok(())
}

fn render_mod_tree_child(
    mod_ident: &str,
    node: &PackageNode<'_>,
    parent_path: &[String],
    ctx: RenderContext<'_>,
    indent: usize,
    parent_out: &mut String,
    module_files: &mut Vec<GeneratedFile>,
) -> Result<(), String> {
    let pad = "    ".repeat(indent);
    let mut current_path = parent_path.to_vec();
    current_path.push(node.raw_segment.clone());

    if !node.files.is_empty() {
        writeln!(parent_out, "{pad}pub mod {mod_ident};").unwrap();

        let mut file_content = String::new();
        render_node_direct_items(node, ctx.symbols, ctx.config, 0, &mut file_content)?;

        let mut child_files = Vec::new();
        for (child_ident, child_node) in &node.children {
            if !file_content.is_empty() && !file_content.ends_with("\n\n") {
                writeln!(file_content).unwrap();
            }
            render_mod_tree_child(
                child_ident,
                child_node,
                &current_path,
                ctx,
                0,
                &mut file_content,
                &mut child_files,
            )?;
        }

        module_files.push(GeneratedFile {
            name: format!("{}.rs", current_path.join("/")),
            content: file_content,
        });
        module_files.extend(child_files);
    } else {
        writeln!(parent_out, "{pad}pub mod {mod_ident} {{").unwrap();
        for (child_ident, child_node) in &node.children {
            render_mod_tree_child(
                child_ident,
                child_node,
                &current_path,
                ctx,
                indent + 1,
                parent_out,
                module_files,
            )?;
        }
        writeln!(parent_out, "{pad}}}").unwrap();
    }

    Ok(())
}

fn render_node_direct_items(
    node: &PackageNode<'_>,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    let mut first = true;

    for file in &node.files {
        let file_name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;

        for enum_res in file.enum_type() {
            let en = enum_res
                .map_err(|e| format!("invalid EnumDescriptorProto in '{file_name}': {e}"))?;
            if !first {
                writeln!(out).unwrap();
            }
            first = false;
            generate_enum(&en, indent, out)?;
        }

        for msg_res in file.message_type() {
            let msg =
                msg_res.map_err(|e| format!("invalid DescriptorProto in '{file_name}': {e}"))?;
            if !first {
                writeln!(out).unwrap();
            }
            first = false;
            generate_full_message(&msg, &node.proto_scope, symbols, config, indent, out)?;
        }
    }

    Ok(())
}

/// Generates `<Msg>View<'a>`, `<Msg>Builder<'b>`, and (when the message declares
/// nested enums, real `oneof`s, or nested messages) `pub mod <msg_snake> { ... }`
/// containing all nested enums, real oneofs, and recursive nested messages (both Views and Builders).
pub fn generate_full_message(
    msg_desc: &DescriptorProtoView<'_>,
    current_scope: &str,
    symbols: &SymbolTable,
    config: &GeneratorConfig,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    generate_view_only(msg_desc, current_scope, symbols, config, indent, out)?;
    writeln!(out).unwrap();
    generate_builder(msg_desc, current_scope, symbols, config, indent, out)?;

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

    writeln!(out).unwrap();
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
        generate_full_message(nested, &msg_scope, symbols, config, indent + 1, out)?;
    }

    writeln!(out, "{pad}}}").unwrap();
    Ok(())
}

fn compute_message_scope(current_scope: &str, raw_msg_name: &str, symbols: &SymbolTable) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use google_protobuf_roto::google::protobuf::{
        compiler::CodeGeneratorRequestBuilder,
        field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    };

    #[test]
    fn generates_cohesive_module_hierarchy_and_escapes_keyword_packages() {
        let mut buf = vec![0u8; 8192];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);

        // File 3: keyword package `google.type.match`
        req.write_proto_file(|fb| {
            fb.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeInt32)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(1)
                        .write_name("type");
                })
                .write_name("Expr");
            })
            .write_package("google.type.match")
            .write_name("google/type/match/expr.proto");
        })
        // File 2: second file in `telemetry` package (merges with File 1)
        .write_proto_file(|fb| {
            fb.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type_name(".telemetry.Location")
                        .write_type(FieldType::TypeMessage)
                        .write_label(FieldLabel::LabelOptional)
                        .write_number(1)
                        .write_name("loc");
                })
                .write_name("Ping");
            })
            .write_package("telemetry")
            .write_name("ping.proto");
        })
        // File 1: first file in `telemetry` package
        .write_proto_file(|fb| {
            fb.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(FieldType::TypeInt32)
                        .write_label(FieldLabel::LabelRequired)
                        .write_number(1)
                        .write_name("lat_e7");
                })
                .write_name("Location");
            })
            .write_package("telemetry")
            .write_name("sensor.proto");
        })
        .write_file_to_generate(" google/type/match/expr.proto".trim())
        .write_file_to_generate("ping.proto")
        .write_file_to_generate("sensor.proto");

        let req_view = CodeGeneratorRequestView::from_bytes(req.finish().unwrap()).unwrap();
        let files = generate(&req_view).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "lib.rs");

        let content = &files[0].content;
        assert!(content.starts_with("#![no_std]\n#![forbid(unsafe_code)]"));
        assert!(content.contains("pub mod google {\n    pub mod r#type {\n        pub mod r#match {"));
        assert!(content.contains("pub struct ExprView<'a>"));
        assert!(content.contains("pub struct ExprBuilder<'b>"));
        assert!(content.contains("pub mod telemetry {"));
        assert!(content.contains("pub struct LocationView<'a>"));
        assert!(content.contains("pub struct LocationBuilder<'b>"));
        assert!(content.contains("pub struct PingView<'a>"));
        assert!(content.contains("pub struct PingBuilder<'b>"));
        // Both sensor.proto and ping.proto must be merged into a single `pub mod telemetry`
        assert_eq!(content.matches("pub mod telemetry {").count(), 1);
    }

    #[test]
    fn generates_mod_rs_and_per_package_files_in_mod_mode() {
        let mut buf = vec![0u8; 8192];
        let mut req = CodeGeneratorRequestBuilder::new(&mut buf);
        req.write_parameter("mode=mod")
            .write_proto_file(|fb| {
                fb.write_message_type(|msg| {
                    msg.write_name("PluginMsg");
                })
                .write_package("google.protobuf.compiler")
                .write_name("google/protobuf/compiler/plugin.proto");
            })
            .write_proto_file(|fb| {
                fb.write_message_type(|msg| {
                    msg.write_name("DescMsg");
                })
                .write_package("google.protobuf")
                .write_name("google/protobuf/descriptor.proto");
            })
            .write_file_to_generate("google/protobuf/compiler/plugin.proto")
            .write_file_to_generate("google/protobuf/descriptor.proto");

        let req_view = CodeGeneratorRequestView::from_bytes(req.finish().unwrap()).unwrap();
        let files = generate(&req_view).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "mod.rs",
                "google/protobuf.rs",
                "google/protobuf/compiler.rs"
            ]
        );
        assert!(files[0]
            .content
            .contains("pub mod google {\n    pub mod protobuf;\n}"));
        assert!(files[1].content.contains("pub struct DescMsgView<'a>"));
        assert!(files[1].content.contains("pub mod compiler;"));
        assert!(files[2].content.contains("pub struct PluginMsgView<'a>"));

        // Verify path-keyword package segments (`self`, `super`, `crate`, `type`) in `mode=mod`
        let mut kw_buf = vec![0u8; 4096];
        let mut kw_req = CodeGeneratorRequestBuilder::new(&mut kw_buf);
        kw_req
            .write_parameter("mode=mod")
            .write_proto_file(|fb| {
                fb.write_message_type(|msg| {
                    msg.write_name("KwMsg");
                })
                .write_package("self.super.type.crate")
                .write_name("kw.proto");
            })
            .write_file_to_generate("kw.proto");
        let kw_view = CodeGeneratorRequestView::from_bytes(kw_req.finish().unwrap()).unwrap();
        let kw_files = generate(&kw_view).unwrap();
        let kw_names: Vec<&str> = kw_files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(kw_names, vec!["mod.rs", "self_/super_/type/crate_.rs"]);
        assert!(kw_files[0].content.contains(
            "pub mod self_ {\n    pub mod super_ {\n        pub mod r#type {\n            pub mod crate_;\n        }\n    }\n}"
        ));
    }
}

