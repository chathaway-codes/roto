use google_protobuf_roto_bootstrap::google::protobuf::compiler::CodeGeneratorRequestView;

/// Output mode controlling how `protoc-gen-roto` organizes generated Rust files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputMode {
    /// Emit a single cohesive `lib.rs` containing the 1:1 nested package module hierarchy.
    #[default]
    Lib,
    /// Emit per-package module files alongside `lib.rs` / `mod.rs`.
    Mod,
}

/// External crate mapping parsed from `extern_path=<proto_prefix>=<rust_crate_path>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternPath {
    /// Canonical dot-separated proto package or type prefix starting with `.` (e.g. `.google.protobuf`).
    pub proto_prefix: String,
    /// Target Rust crate or module path (e.g. `::google_protobuf_roto` or `::google_protobuf_roto::google::protobuf`).
    pub rust_path: String,
}

/// Parsed plugin configuration from `CodeGeneratorRequest.parameter`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeneratorConfig {
    pub mode: OutputMode,
    pub extern_paths: Vec<ExternPath>,
}

impl GeneratorConfig {
    /// Parses `CodeGeneratorRequestView::parameter()` into a `GeneratorConfig`.
    pub fn from_request(request: &CodeGeneratorRequestView<'_>) -> Result<Self, String> {
        let parameter = request
            .parameter()
            .map_err(|e| format!("invalid CodeGeneratorRequest.parameter: {e}"))?;
        Self::parse(parameter)
    }

    /// Parses a comma-separated `protoc` plugin parameter string (`--roto_opt=...`).
    ///
    /// Supported options:
    /// - `mode=lib` (default) or `mode=mod`
    /// - `extern_path=<proto_prefix>=<rust_crate_path>` (supports `.google.protobuf`, `google.protobuf`, and `google/protobuf` prefixes)
    /// - `error=<msg>` (surfaces `<msg>` as a generator error)
    pub fn parse(parameter: &str) -> Result<Self, String> {
        let mut config = Self::default();

        for raw_part in parameter.split(',') {
            let part = raw_part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some(msg) = part.strip_prefix("error=") {
                return Err(msg.to_string());
            } else if let Some(mode_val) = part.strip_prefix("mode=") {
                config.mode = match mode_val.trim() {
                    "lib" => OutputMode::Lib,
                    "mod" => OutputMode::Mod,
                    other => {
                        return Err(format!(
                            "invalid mode '{other}' in parameter '{part}': expected 'lib' or 'mod'"
                        ));
                    }
                };
            } else if let Some(ext_val) = part.strip_prefix("extern_path=") {
                let (raw_proto, raw_rust) = ext_val.split_once('=').ok_or_else(|| {
                    format!(
                        "invalid extern_path '{part}': expected 'extern_path=<proto_prefix>=<rust_path>'"
                    )
                })?;

                let proto_prefix = normalize_proto_prefix(raw_proto).ok_or_else(|| {
                    format!(
                        "invalid proto prefix '{raw_proto}' in parameter '{part}': must not be empty"
                    )
                })?;

                let rust_path = normalize_rust_path(raw_rust).ok_or_else(|| {
                    format!(
                        "invalid rust path '{raw_rust}' in parameter '{part}': must not be empty"
                    )
                })?;

                if let Some(existing) = config
                    .extern_paths
                    .iter_mut()
                    .find(|e| e.proto_prefix == proto_prefix)
                {
                    existing.rust_path = rust_path;
                } else {
                    config.extern_paths.push(ExternPath {
                        proto_prefix,
                        rust_path,
                    });
                }
            } else {
                return Err(format!("unknown protoc-gen-roto parameter '{part}'"));
            }
        }

        Ok(config)
    }

    /// Finds the `ExternPath` with the longest `proto_prefix` matching `fq_proto_name`
    /// on a `.` segment boundary.
    pub fn match_extern_path(&self, fq_proto_name: &str) -> Option<&ExternPath> {
        let normalized_fq = if fq_proto_name.starts_with('.') {
            fq_proto_name.to_string()
        } else {
            format!(".{fq_proto_name}")
        };

        self.extern_paths
            .iter()
            .filter(|ext| prefix_matches(&normalized_fq, &ext.proto_prefix))
            .max_by_key(|ext| ext.proto_prefix.len())
    }
}

/// Normalizes a dot- or slash-separated proto prefix (e.g. `google/protobuf`,
/// `.google.protobuf`, `google.type`) into canonical `.google.protobuf` form.
pub fn normalize_proto_prefix(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches(|c| c == '.' || c == '/');
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::with_capacity(trimmed.len() + 1);
    for seg in trimmed.split(['.', '/']) {
        let seg = seg.trim();
        if seg.is_empty() {
            return None;
        }
        normalized.push('.');
        normalized.push_str(seg);
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_rust_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches("::");
    if trimmed.is_empty() || trimmed.contains('=') {
        return None;
    }

    let without_leading = trimmed.strip_prefix("::").unwrap_or(trimmed);
    if without_leading.is_empty() {
        return None;
    }

    for seg in without_leading.split("::") {
        if seg.trim().is_empty() {
            return None;
        }
    }

    Some(trimmed.to_string())
}

fn prefix_matches(fq_proto_name: &str, proto_prefix: &str) -> bool {
    if fq_proto_name == proto_prefix {
        return true;
    }
    if let Some(rest) = fq_proto_name.strip_prefix(proto_prefix) {
        rest.starts_with('.')
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_and_explicit_output_modes() {
        let cfg = GeneratorConfig::parse("").unwrap();
        assert_eq!(cfg.mode, OutputMode::Lib);
        assert!(cfg.extern_paths.is_empty());

        let cfg_lib = GeneratorConfig::parse("mode=lib").unwrap();
        assert_eq!(cfg_lib.mode, OutputMode::Lib);

        let cfg_mod = GeneratorConfig::parse("mode=mod").unwrap();
        assert_eq!(cfg_mod.mode, OutputMode::Mod);
    }

    #[test]
    fn parse_multiple_extern_paths_dot_and_slash_forms() {
        let param = "mode=mod,extern_path=google/protobuf=::google_protobuf_roto,extern_path=.google.protobuf.compiler=::protoc_plugin_roto::google::protobuf::compiler,extern_path=google.type=::googleapis_roto";
        let cfg = GeneratorConfig::parse(param).unwrap();
        assert_eq!(cfg.mode, OutputMode::Mod);
        assert_eq!(
            cfg.extern_paths,
            vec![
                ExternPath {
                    proto_prefix: ".google.protobuf".to_string(),
                    rust_path: "::google_protobuf_roto".to_string(),
                },
                ExternPath {
                    proto_prefix: ".google.protobuf.compiler".to_string(),
                    rust_path: "::protoc_plugin_roto::google::protobuf::compiler".to_string(),
                },
                ExternPath {
                    proto_prefix: ".google.type".to_string(),
                    rust_path: "::googleapis_roto".to_string(),
                },
            ]
        );

        // Longest prefix match wins
        let matched_compiler = cfg
            .match_extern_path(".google.protobuf.compiler.CodeGeneratorRequest")
            .unwrap();
        assert_eq!(matched_compiler.proto_prefix, ".google.protobuf.compiler");

        let matched_pb = cfg
            .match_extern_path(".google.protobuf.Timestamp")
            .unwrap();
        assert_eq!(matched_pb.proto_prefix, ".google.protobuf");

        // Segment boundary enforcement (.google.protobuf must not match .google.protobuf_ext.Foo)
        assert!(cfg
            .match_extern_path(".google.protobuf_ext.Foo")
            .is_none());
    }

    #[test]
    fn parse_rejects_invalid_parameters_and_preserves_error_option() {
        assert_eq!(
            GeneratorConfig::parse("error=custom_failure").unwrap_err(),
            "custom_failure"
        );
        assert!(GeneratorConfig::parse("mode=flat").is_err());
        assert!(GeneratorConfig::parse("extern_path=.google.protobuf").is_err());
        assert!(GeneratorConfig::parse("extern_path==::crate").is_err());
        assert!(GeneratorConfig::parse("extern_path=.google.protobuf=").is_err());
        assert!(GeneratorConfig::parse("extern_path=google..protobuf=::crate").is_err());
        assert!(GeneratorConfig::parse("unknown_flag=1").is_err());
    }
}
