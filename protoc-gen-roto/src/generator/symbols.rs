use std::collections::BTreeMap;

use google_protobuf_roto::google::protobuf::{
    compiler::CodeGeneratorRequestView, field_descriptor_proto::Type as FieldType,
    DescriptorProtoView, EnumDescriptorProtoView, FieldDescriptorProtoView,
    FileDescriptorProtoView,
};

use super::config::{ExternPath, GeneratorConfig};
use super::ident::{escape_ident, escape_mod_ident, sanitize_type_ident, to_pascal_case};

/// Kind of a Protobuf type registered in the [`SymbolTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    /// A Protobuf `message` (or `group`).
    Message,
    /// A Protobuf `enum` with its default first variant and full list of
    /// `(proto_variant_name, rust_variant_ident, tag_number)` entries.
    Enum {
        default_variant: String,
        variants: Vec<(String, String, i32)>,
    },
}

/// Metadata for an indexed message or enum descriptor in the [`SymbolTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    /// Canonical fully-qualified Protobuf name starting with `.` (e.g. `.google.protobuf.FieldDescriptorProto.Type`).
    pub fq_proto_name: String,
    /// Message vs Enum kind and enum variant metadata.
    pub kind: SymbolKind,
    /// Raw proto package name (e.g. `"google.protobuf"` or `""`).
    pub package: String,
    /// Unescaped proto package segments (e.g. `["google", "protobuf"]`).
    pub package_segments: Vec<String>,
    /// Enclosing message names from outermost to immediate parent (e.g. `["GeneratedCodeInfo", "Annotation"]`).
    pub nesting_path: Vec<String>,
    /// Sanitized Rust type identifier (e.g. `"FileDescriptorProto"`, `"Type"`, `"JSType"`).
    pub rust_name: String,
    /// Source `.proto` file name where this symbol was defined.
    pub source_file: String,
}

impl Symbol {
    /// Resolves a Protobuf enum default value name (e.g. `"SPEED"` or `""` for the first variant)
    /// to its Rust variant identifier (e.g. `"Speed"`).
    pub fn resolve_enum_variant(&self, proto_variant_name: &str) -> Option<&str> {
        match &self.kind {
            SymbolKind::Enum {
                default_variant,
                variants,
            } => {
                if proto_variant_name.is_empty() {
                    if default_variant.is_empty() {
                        None
                    } else {
                        Some(default_variant.as_str())
                    }
                } else {
                    variants
                        .iter()
                        .find(|(proto_name, _, _)| proto_name == proto_variant_name)
                        .map(|(_, rust_ident, _)| rust_ident.as_str())
                }
            }
            SymbolKind::Message => None,
        }
    }
}

/// Resolved Rust paths for a Protobuf message or enum reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedType {
    /// Canonical fully-qualified Protobuf name starting with `.`.
    pub fq_proto_name: String,
    /// Symbol kind (Message or Enum).
    pub kind: SymbolKind,
    /// True if resolved through an `extern_path` mapping to an external crate.
    pub is_extern: bool,
    /// Rust module path containing the type (e.g. `crate::google::protobuf::field_descriptor_proto`
    /// or `::google_protobuf_roto::google::protobuf`).
    pub module_path: String,
    /// Sanitized Rust base type name (e.g. `FileDescriptorProto`, `Type`, `JSType`).
    pub rust_name: String,
    /// Full Rust View type with lifetime for messages (`<module_path>::<Name>View<'a>`)
    /// or enum path for enums (`<module_path>::<Name>`).
    pub view_type: String,
    /// Full Rust View struct path without lifetime parameters (`<module_path>::<Name>View`)
    /// or enum path for enums (`<module_path>::<Name>`).
    pub view_ident_path: String,
    /// Full Rust Builder type with lifetimes for messages (`<module_path>::<Name>Builder<'a, 'b>`)
    /// or enum path for enums (`<module_path>::<Name>`).
    pub builder_type: String,
    /// Full Rust Builder struct path without lifetime parameters (`<module_path>::<Name>Builder`)
    /// or enum path for enums (`<module_path>::<Name>`).
    pub builder_ident_path: String,
    /// Full Rust enum path (`<module_path>::<Name>`).
    pub enum_type: String,
}

impl ResolvedType {
    /// Returns the full Rust View type (`...::<Name>View<'a>`).
    #[inline]
    pub fn view_path(&self) -> &str {
        &self.view_type
    }

    /// Returns the full Rust Builder type (`...::<Name>Builder<'a, 'b>`).
    #[inline]
    pub fn builder_path(&self) -> &str {
        &self.builder_type
    }

    /// Returns the full Rust Enum type (`...::<Name>`).
    #[inline]
    pub fn enum_path(&self) -> &str {
        &self.enum_type
    }

    /// Returns true if the resolved symbol is a message.
    #[inline]
    pub fn is_message(&self) -> bool {
        matches!(self.kind, SymbolKind::Message)
    }

    /// Returns true if the resolved symbol is an enum.
    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self.kind, SymbolKind::Enum { .. })
    }

    /// Resolves a default enum variant path (`<enum_path>::<VariantIdent>`) from an optional
    /// Protobuf default value string (or the enum's first variant when `proto_default` is empty).
    pub fn resolve_enum_default(&self, proto_default: &str) -> Result<String, String> {
        match &self.kind {
            SymbolKind::Enum {
                default_variant,
                variants,
            } => {
                let variant_ident = if proto_default.is_empty() {
                    if default_variant.is_empty() {
                        return Err(format!(
                            "enum '{}' has no variants to resolve default",
                            self.fq_proto_name
                        ));
                    }
                    default_variant.as_str()
                } else {
                    variants
                        .iter()
                        .find(|(proto_name, _, _)| proto_name == proto_default)
                        .map(|(_, rust_ident, _)| rust_ident.as_str())
                        .ok_or_else(|| {
                            format!(
                                "unknown enum default variant '{proto_default}' for '{}'",
                                self.fq_proto_name
                            )
                        })?
                };
                Ok(format!("{}::{variant_ident}", self.enum_type))
            }
            SymbolKind::Message => Err(format!(
                "cannot resolve enum default on message type '{}'",
                self.fq_proto_name
            )),
        }
    }
}

/// Multi-file symbol table indexing all messages and enums across a `CodeGeneratorRequestView`.
#[derive(Clone, Debug, Default)]
pub struct SymbolTable {
    symbols: BTreeMap<String, Symbol>,
}

impl SymbolTable {
    /// Creates an empty [`SymbolTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes all `FileDescriptorProtoView` entries in `CodeGeneratorRequestView::proto_file()`.
    pub fn from_request(request: &CodeGeneratorRequestView<'_>) -> Result<Self, String> {
        let mut table = Self::new();
        for file_res in request.proto_file() {
            let file =
                file_res.map_err(|e| format!("invalid FileDescriptorProto in proto_file: {e}"))?;
            table.insert_file(&file)?;
        }
        Ok(table)
    }

    /// Indexes an iterator of `FileDescriptorProtoView` descriptors.
    pub fn from_files<'a, I>(files: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = FileDescriptorProtoView<'a>>,
    {
        let mut table = Self::new();
        for file in files {
            table.insert_file(&file)?;
        }
        Ok(table)
    }

    /// Indexes all top-level and recursively nested messages and enums in `file`.
    pub fn insert_file(&mut self, file: &FileDescriptorProtoView<'_>) -> Result<(), String> {
        let source_file = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?
            .to_string();
        let package = file
            .package()
            .map_err(|e| format!("invalid FileDescriptorProto.package in '{source_file}': {e}"))?
            .trim_matches('.')
            .to_string();

        let package_segments: Vec<String> = if package.is_empty() {
            Vec::new()
        } else {
            package.split('.').map(str::to_string).collect()
        };

        let base_prefix = if package.is_empty() {
            String::new()
        } else {
            format!(".{package}")
        };

        for enum_res in file.enum_type() {
            let en = enum_res
                .map_err(|e| format!("invalid EnumDescriptorProto in '{source_file}': {e}"))?;
            self.insert_enum(&en, &base_prefix, &package, &package_segments, &[], &source_file)?;
        }

        for msg_res in file.message_type() {
            let msg = msg_res
                .map_err(|e| format!("invalid DescriptorProto in '{source_file}': {e}"))?;
            self.insert_message(
                &msg,
                &base_prefix,
                &package,
                &package_segments,
                &[],
                &source_file,
            )?;
        }

        Ok(())
    }

    fn insert_message(
        &mut self,
        msg: &DescriptorProtoView<'_>,
        parent_prefix: &str,
        package: &str,
        package_segments: &[String],
        nesting_path: &[String],
        source_file: &str,
    ) -> Result<(), String> {
        let name = msg
            .name()
            .map_err(|e| format!("invalid DescriptorProto.name in '{source_file}': {e}"))?;
        if name.is_empty() {
            return Err(format!(
                "DescriptorProto.name cannot be empty in '{source_file}'"
            ));
        }

        let fq_proto_name = format!("{parent_prefix}.{name}");
        let rust_name = sanitize_type_ident(name);

        self.symbols.insert(
            fq_proto_name.clone(),
            Symbol {
                fq_proto_name: fq_proto_name.clone(),
                kind: SymbolKind::Message,
                package: package.to_string(),
                package_segments: package_segments.to_vec(),
                nesting_path: nesting_path.to_vec(),
                rust_name,
                source_file: source_file.to_string(),
            },
        );

        let mut child_nesting = nesting_path.to_vec();
        child_nesting.push(name.to_string());

        for enum_res in msg.enum_type() {
            let en = enum_res.map_err(|e| {
                format!("invalid nested EnumDescriptorProto in '{fq_proto_name}': {e}")
            })?;
            self.insert_enum(
                &en,
                &fq_proto_name,
                package,
                package_segments,
                &child_nesting,
                source_file,
            )?;
        }

        for nested_res in msg.nested_type() {
            let nested = nested_res.map_err(|e| {
                format!("invalid nested DescriptorProto in '{fq_proto_name}': {e}")
            })?;
            self.insert_message(
                &nested,
                &fq_proto_name,
                package,
                package_segments,
                &child_nesting,
                source_file,
            )?;
        }

        Ok(())
    }

    fn insert_enum(
        &mut self,
        en: &EnumDescriptorProtoView<'_>,
        parent_prefix: &str,
        package: &str,
        package_segments: &[String],
        nesting_path: &[String],
        source_file: &str,
    ) -> Result<(), String> {
        let name = en
            .name()
            .map_err(|e| format!("invalid EnumDescriptorProto.name in '{source_file}': {e}"))?;
        if name.is_empty() {
            return Err(format!(
                "EnumDescriptorProto.name cannot be empty in '{source_file}'"
            ));
        }

        let fq_proto_name = format!("{parent_prefix}.{name}");
        let rust_name = sanitize_type_ident(name);

        let mut variants = Vec::new();
        for val_res in en.value() {
            let val = val_res.map_err(|e| {
                format!("invalid EnumValueDescriptorProto in '{fq_proto_name}': {e}")
            })?;
            let val_name = val.name().map_err(|e| {
                format!("invalid EnumValueDescriptorProto.name in '{fq_proto_name}': {e}")
            })?;
            let val_num = val.number().map_err(|e| {
                format!("invalid EnumValueDescriptorProto.number in '{fq_proto_name}': {e}")
            })?;
            let rust_variant = escape_ident(&to_pascal_case(val_name));
            variants.push((val_name.to_string(), rust_variant, val_num));
        }

        let default_variant = variants
            .first()
            .map(|(_, rust_ident, _)| rust_ident.clone())
            .unwrap_or_default();

        self.symbols.insert(
            fq_proto_name.clone(),
            Symbol {
                fq_proto_name,
                kind: SymbolKind::Enum {
                    default_variant,
                    variants,
                },
                package: package.to_string(),
                package_segments: package_segments.to_vec(),
                nesting_path: nesting_path.to_vec(),
                rust_name,
                source_file: source_file.to_string(),
            },
        );

        Ok(())
    }

    /// Returns the number of indexed symbols.
    #[inline]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns true if no symbols have been indexed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Looks up a symbol by its exact canonical fully-qualified name (e.g. `.google.protobuf.Timestamp`).
    pub fn get(&self, fq_proto_name: &str) -> Option<&Symbol> {
        if fq_proto_name.starts_with('.') {
            self.symbols.get(fq_proto_name)
        } else {
            let dotted = format!(".{fq_proto_name}");
            self.symbols.get(&dotted)
        }
    }

    /// Iterates over all `(fq_proto_name, Symbol)` entries in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Symbol)> {
        self.symbols.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Looks up a Protobuf `type_name` (either fully-qualified starting with `.` or relative
    /// C++-scoped from `current_scope`) and returns the matching [`Symbol`].
    pub fn lookup<'a>(&'a self, current_scope: &str, type_name: &str) -> Option<&'a Symbol> {
        let trimmed = type_name.trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed.starts_with('.') {
            return self.symbols.get(trimmed);
        }

        let mut scope = current_scope.trim().trim_matches('.').to_string();
        loop {
            let candidate = if scope.is_empty() {
                format!(".{trimmed}")
            } else {
                format!(".{scope}.{trimmed}")
            };
            if let Some(sym) = self.symbols.get(&candidate) {
                return Some(sym);
            }
            if scope.is_empty() {
                break;
            }
            match scope.rfind('.') {
                Some(idx) => scope.truncate(idx),
                None => scope.clear(),
            }
        }

        None
    }

    /// Resolves `type_name` from `current_scope` into a [`ResolvedType`] using `config`
    /// (`crate::...` for local symbols or `::<extern_crate>::...` when matched by longest-prefix `extern_path`).
    pub fn resolve(
        &self,
        current_scope: &str,
        type_name: &str,
        config: &GeneratorConfig,
    ) -> Result<ResolvedType, String> {
        let symbol = self.lookup(current_scope, type_name).ok_or_else(|| {
            format!(
                "unresolved protobuf type '{type_name}' (from scope '{current_scope}')"
            )
        })?;

        Ok(resolve_symbol(symbol, config))
    }

    /// Resolves the message or enum type of `field` (if it is a message, group, or enum field
    /// or has a non-empty `type_name`) from `current_scope`.
    pub fn resolve_field(
        &self,
        current_scope: &str,
        field: &FieldDescriptorProtoView<'_>,
        config: &GeneratorConfig,
    ) -> Result<Option<ResolvedType>, String> {
        let field_name = field
            .name()
            .map_err(|e| format!("invalid FieldDescriptorProto.name: {e}"))?;
        let type_name = field
            .type_name()
            .map_err(|e| format!("invalid FieldDescriptorProto.type_name in '{field_name}': {e}"))?;
        let field_type = field
            .r#type()
            .map_err(|e| format!("invalid FieldDescriptorProto.type in '{field_name}': {e}"))?;

        let needs_resolution = !type_name.is_empty()
            || matches!(
                field_type,
                FieldType::TypeMessage | FieldType::TypeEnum | FieldType::TypeGroup
            );

        if !needs_resolution {
            return Ok(None);
        }

        if type_name.is_empty() {
            return Err(format!(
                "field '{field_name}' in '{current_scope}' has composite type {field_type:?} without type_name"
            ));
        }

        self.resolve(current_scope, type_name, config).map(Some)
    }
}

fn resolve_symbol(symbol: &Symbol, config: &GeneratorConfig) -> ResolvedType {
    let pkg_mods: Vec<String> = symbol
        .package_segments
        .iter()
        .map(|s| escape_mod_ident(s))
        .collect();
    let nesting_mods: Vec<String> = symbol
        .nesting_path
        .iter()
        .map(|s| escape_mod_ident(s))
        .collect();

    let (is_extern, module_path, rust_name) =
        if let Some(ext) = config.match_extern_path(&symbol.fq_proto_name) {
            let (mod_path, type_ident) = compute_extern_module_and_type(
                symbol,
                ext,
                &pkg_mods,
                &nesting_mods,
            );
            (true, mod_path, type_ident)
        } else {
            let mut segments = Vec::with_capacity(1 + pkg_mods.len() + nesting_mods.len());
            segments.push("crate".to_string());
            segments.extend(pkg_mods);
            segments.extend(nesting_mods);
            (false, segments.join("::"), symbol.rust_name.clone())
        };

    let enum_type = format!("{module_path}::{rust_name}");
    let (view_type, view_ident_path, builder_type, builder_ident_path) = match symbol.kind {
        SymbolKind::Message => (
            format!("{module_path}::{rust_name}View<'a>"),
            format!("{module_path}::{rust_name}View"),
            format!("{module_path}::{rust_name}Builder<'a, 'b>"),
            format!("{module_path}::{rust_name}Builder"),
        ),
        SymbolKind::Enum { .. } => (
            enum_type.clone(),
            enum_type.clone(),
            enum_type.clone(),
            enum_type.clone(),
        ),
    };

    ResolvedType {
        fq_proto_name: symbol.fq_proto_name.clone(),
        kind: symbol.kind.clone(),
        is_extern,
        module_path,
        rust_name,
        view_type,
        view_ident_path,
        builder_type,
        builder_ident_path,
        enum_type,
    }
}

fn compute_extern_module_and_type(
    symbol: &Symbol,
    ext: &ExternPath,
    pkg_mods: &[String],
    nesting_mods: &[String],
) -> (String, String) {
    let prefix_segments: Vec<&str> = ext
        .proto_prefix
        .trim_start_matches('.')
        .split('.')
        .filter(|s| !s.is_empty())
        .collect();

    let rust_trimmed = ext.rust_path.trim_end_matches("::");
    let has_leading_colons = rust_trimmed.starts_with("::");
    let rust_segs: Vec<&str> = rust_trimmed
        .trim_start_matches("::")
        .split("::")
        .filter(|s| !s.is_empty())
        .collect();

    // Case 1: `rust_path` is a single crate identifier (e.g. `::google_protobuf_roto`).
    // In roto's 1:1 package-to-module hierarchy, the crate root exports `pub mod <pkg> { ... }`.
    if rust_segs.len() == 1 {
        let mut mod_segs = Vec::with_capacity(1 + pkg_mods.len() + nesting_mods.len());
        mod_segs.push(rust_segs[0].to_string());
        mod_segs.extend_from_slice(pkg_mods);
        mod_segs.extend_from_slice(nesting_mods);
        let joined = mod_segs.join("::");
        let mod_path = if has_leading_colons {
            format!("::{joined}")
        } else {
            joined
        };
        return (mod_path, symbol.rust_name.clone());
    }

    // Case 2: `proto_prefix` is a package or parent-package prefix, and `rust_path` already
    // includes a module path (e.g. `::google_protobuf_roto::google::protobuf` or `::googleapis_roto::google::r#type`).
    if prefix_segments.len() <= pkg_mods.len() {
        let mut mod_segs: Vec<String> = rust_segs.iter().map(|s| (*s).to_string()).collect();
        mod_segs.extend_from_slice(&pkg_mods[prefix_segments.len()..]);
        mod_segs.extend_from_slice(nesting_mods);
        let joined = mod_segs.join("::");
        let mod_path = if has_leading_colons {
            format!("::{joined}")
        } else {
            joined
        };
        return (mod_path, symbol.rust_name.clone());
    }

    // Case 3: `proto_prefix` targets an exact message/enum type or an enclosing message.
    if ext.proto_prefix == symbol.fq_proto_name {
        let type_ident = rust_segs.last().copied().unwrap_or(&symbol.rust_name);
        let parent_segs = &rust_segs[..rust_segs.len() - 1];
        let joined = parent_segs.join("::");
        let mod_path = if has_leading_colons {
            format!("::{joined}")
        } else {
            joined
        };
        return (mod_path, type_ident.to_string());
    }

    let covered_nesting = (prefix_segments.len() - pkg_mods.len()).min(nesting_mods.len());
    let mut mod_segs: Vec<String> = rust_segs.iter().map(|s| (*s).to_string()).collect();
    mod_segs.extend_from_slice(&nesting_mods[covered_nesting..]);
    let joined = mod_segs.join("::");
    let mod_path = if has_leading_colons {
        format!("::{joined}")
    } else {
        joined
    };
    (mod_path, symbol.rust_name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    use google_protobuf_roto::google::protobuf::{
        field_descriptor_proto::{Label, Type},
        FileDescriptorProtoBuilder, FileDescriptorSetView,
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
    fn indexes_real_descriptor_and_plugin_protos_and_resolves_local_paths() {
        let fds_bytes = load_descriptor_and_plugin_fds();
        let fds = FileDescriptorSetView::from_bytes(&fds_bytes).unwrap();
        let table = SymbolTable::from_files(fds.file().map(|f| f.unwrap())).unwrap();
        let cfg = GeneratorConfig::default();

        // Top-level message in google.protobuf
        let fdp = table
            .resolve("", ".google.protobuf.FileDescriptorProto", &cfg)
            .unwrap();
        assert!(!fdp.is_extern);
        assert!(fdp.is_message());
        assert_eq!(fdp.module_path, "crate::google::protobuf");
        assert_eq!(
            fdp.view_path(),
            "crate::google::protobuf::FileDescriptorProtoView<'a>"
        );
        assert_eq!(
            fdp.builder_path(),
            "crate::google::protobuf::FileDescriptorProtoBuilder<'a, 'b>"
        );

        // Nested enum in google.protobuf (FieldDescriptorProto.Type)
        let fdp_type = table
            .resolve(
                ".google.protobuf.FieldDescriptorProto",
                "Type",
                &cfg,
            )
            .unwrap();
        assert!(fdp_type.is_enum());
        assert_eq!(
            fdp_type.enum_path(),
            "crate::google::protobuf::field_descriptor_proto::Type"
        );
        assert_eq!(
            fdp_type.resolve_enum_default("").unwrap(),
            "crate::google::protobuf::field_descriptor_proto::Type::TypeDouble"
        );
        assert_eq!(
            fdp_type.resolve_enum_default("TYPE_SFIXED32").unwrap(),
            "crate::google::protobuf::field_descriptor_proto::Type::TypeSfixed32"
        );

        // Nested message in google.protobuf (SourceCodeInfo.Location)
        let sci_loc = table
            .resolve(".google.protobuf.SourceCodeInfo", "Location", &cfg)
            .unwrap();
        assert!(sci_loc.is_message());
        assert_eq!(
            sci_loc.view_path(),
            "crate::google::protobuf::source_code_info::LocationView<'a>"
        );
        assert_eq!(
            sci_loc.builder_path(),
            "crate::google::protobuf::source_code_info::LocationBuilder<'a, 'b>"
        );

        // Deeply nested enum in google.protobuf (GeneratedCodeInfo.Annotation.Semantic)
        let semantic = table
            .resolve(
                ".google.protobuf.GeneratedCodeInfo.Annotation",
                "Semantic",
                &cfg,
            )
            .unwrap();
        assert!(semantic.is_enum());
        assert_eq!(
            semantic.enum_path(),
            "crate::google::protobuf::generated_code_info::annotation::Semantic"
        );

        // Acronym enum names preserved (FieldOptions.JSType and FieldOptions.CType)
        let js_type = table
            .resolve(".google.protobuf.FieldOptions", "JSType", &cfg)
            .unwrap();
        assert_eq!(
            js_type.enum_path(),
            "crate::google::protobuf::field_options::JSType"
        );
        assert_eq!(
            js_type.resolve_enum_default("JS_NORMAL").unwrap(),
            "crate::google::protobuf::field_options::JSType::JsNormal"
        );

        let c_type = table
            .resolve(".google.protobuf.FieldOptions", "CType", &cfg)
            .unwrap();
        assert_eq!(
            c_type.enum_path(),
            "crate::google::protobuf::field_options::CType"
        );
        assert_eq!(
            c_type.resolve_enum_default("STRING_PIECE").unwrap(),
            "crate::google::protobuf::field_options::CType::StringPiece"
        );

        // FileOptions.OptimizeMode [default = SPEED]
        let opt_mode = table
            .resolve(".google.protobuf.FileOptions", "OptimizeMode", &cfg)
            .unwrap();
        assert_eq!(
            opt_mode.enum_path(),
            "crate::google::protobuf::file_options::OptimizeMode"
        );
        assert_eq!(
            opt_mode.resolve_enum_default("SPEED").unwrap(),
            "crate::google::protobuf::file_options::OptimizeMode::Speed"
        );

        // Nested message in google.protobuf.compiler (CodeGeneratorResponse.File)
        let cgr_file = table
            .resolve(
                ".google.protobuf.compiler.CodeGeneratorResponse",
                "File",
                &cfg,
            )
            .unwrap();
        assert!(cgr_file.is_message());
        assert_eq!(
            cgr_file.view_path(),
            "crate::google::protobuf::compiler::code_generator_response::FileView<'a>"
        );
        assert_eq!(
            cgr_file.builder_path(),
            "crate::google::protobuf::compiler::code_generator_response::FileBuilder<'a, 'b>"
        );

        // Cross-package field resolution on real CodeGeneratorRequest descriptor:
        // `proto_file` field in `google.protobuf.compiler.CodeGeneratorRequest` references `.google.protobuf.FileDescriptorProto`
        let plugin_file = fds
            .file()
            .map(|f| f.unwrap())
            .find(|f| f.name() == Ok("google/protobuf/compiler/plugin.proto"))
            .unwrap();
        let cgr_msg = plugin_file
            .message_type()
            .map(|m| m.unwrap())
            .find(|m| m.name() == Ok("CodeGeneratorRequest"))
            .unwrap();
        let proto_file_field = cgr_msg
            .field()
            .map(|f| f.unwrap())
            .find(|f| f.name() == Ok("proto_file"))
            .unwrap();

        let resolved_field = table
            .resolve_field(
                ".google.protobuf.compiler.CodeGeneratorRequest",
                &proto_file_field,
                &cfg,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            resolved_field.view_path(),
            "crate::google::protobuf::FileDescriptorProtoView<'a>"
        );
    }

    #[test]
    fn resolves_extern_paths_for_crate_roots_module_paths_and_escaped_keywords() {
        let fds_bytes = load_descriptor_and_plugin_fds();
        let fds = FileDescriptorSetView::from_bytes(&fds_bytes).unwrap();
        let mut table = SymbolTable::from_files(fds.file().map(|f| f.unwrap())).unwrap();

        // Add a synthetic `google/type/latlng.proto` (package `google.type`, keyword `type`)
        // and a root-package `nofolder.proto` (package `""`)
        let mut buf1 = [0u8; 512];
        let mut w1 = BackwardWriter::new(&mut buf1);
        {
            let mut f = FileDescriptorProtoBuilder::new(&mut w1);
            f.write_message_type(|m| {
                m.write_field(|fld| {
                    fld.write_type(Type::TypeDouble)?;
                    fld.write_label(Label::LabelOptional)?;
                    fld.write_number(1)?;
                    fld.write_name("latitude")
                })?;
                m.write_name("LatLng")
            })
            .unwrap();
            f.write_package("google.type").unwrap();
            f.write_name("google/type/latlng.proto").unwrap();
        }
        let latlng_file = FileDescriptorProtoView::from_bytes(w1.finish()).unwrap();
        table.insert_file(&latlng_file).unwrap();

        let mut buf2 = [0u8; 512];
        let mut w2 = BackwardWriter::new(&mut buf2);
        {
            let mut f = FileDescriptorProtoBuilder::new(&mut w2);
            f.write_message_type(|m| m.write_name("RootEnvelope")).unwrap();
            f.write_name("root.proto").unwrap();
        }
        let root_file = FileDescriptorProtoView::from_bytes(w2.finish()).unwrap();
        table.insert_file(&root_file).unwrap();

        // 1. Crate-root extern_path mappings (`google/protobuf=::google_protobuf_roto` and `google/type=::googleapis_roto`)
        let cfg_crate_roots = GeneratorConfig::parse(
            "extern_path=google/protobuf=::google_protobuf_roto,extern_path=google/type=::googleapis_roto",
        )
        .unwrap();

        let ext_fdp = table
            .resolve("", ".google.protobuf.FileDescriptorProto", &cfg_crate_roots)
            .unwrap();
        assert!(ext_fdp.is_extern);
        assert_eq!(
            ext_fdp.view_path(),
            "::google_protobuf_roto::google::protobuf::FileDescriptorProtoView<'a>"
        );
        assert_eq!(
            ext_fdp.builder_path(),
            "::google_protobuf_roto::google::protobuf::FileDescriptorProtoBuilder<'a, 'b>"
        );

        let ext_nested_enum = table
            .resolve(
                "",
                ".google.protobuf.FieldDescriptorProto.Type",
                &cfg_crate_roots,
            )
            .unwrap();
        assert!(ext_nested_enum.is_extern);
        assert_eq!(
            ext_nested_enum.enum_path(),
            "::google_protobuf_roto::google::protobuf::field_descriptor_proto::Type"
        );

        let ext_compiler_file = table
            .resolve(
                "",
                ".google.protobuf.compiler.CodeGeneratorResponse.File",
                &cfg_crate_roots,
            )
            .unwrap();
        assert!(ext_compiler_file.is_extern);
        assert_eq!(
            ext_compiler_file.view_path(),
            "::google_protobuf_roto::google::protobuf::compiler::code_generator_response::FileView<'a>"
        );

        let ext_latlng = table
            .resolve("", ".google.type.LatLng", &cfg_crate_roots)
            .unwrap();
        assert!(ext_latlng.is_extern);
        assert_eq!(
            ext_latlng.view_path(),
            "::googleapis_roto::google::r#type::LatLngView<'a>"
        );
        assert_eq!(
            ext_latlng.builder_path(),
            "::googleapis_roto::google::r#type::LatLngBuilder<'a, 'b>"
        );

        // 2. Full module-path extern_path mappings (must not duplicate `::google::protobuf` or `::google::r#type`)
        let cfg_full_mods = GeneratorConfig::parse(
            "extern_path=.google.protobuf=::google_protobuf_roto::google::protobuf,extern_path=.google.type=::googleapis_roto::google::r#type",
        )
        .unwrap();

        let ext_fdp2 = table
            .resolve("", ".google.protobuf.FileDescriptorProto", &cfg_full_mods)
            .unwrap();
        assert_eq!(
            ext_fdp2.view_path(),
            "::google_protobuf_roto::google::protobuf::FileDescriptorProtoView<'a>"
        );

        let ext_latlng2 = table
            .resolve("", ".google.type.LatLng", &cfg_full_mods)
            .unwrap();
        assert_eq!(
            ext_latlng2.view_path(),
            "::googleapis_roto::google::r#type::LatLngView<'a>"
        );

        // 3. Empty-package message resolves to `crate::RootEnvelopeView<'a>`
        let root_msg = table
            .resolve("", ".RootEnvelope", &cfg_crate_roots)
            .unwrap();
        assert!(!root_msg.is_extern);
        assert_eq!(root_msg.view_path(), "crate::RootEnvelopeView<'a>");
        assert_eq!(
            root_msg.builder_path(),
            "crate::RootEnvelopeBuilder<'a, 'b>"
        );
    }
}
