use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of `.proto` source file discovery for a target crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredProtos {
    /// Target crate's proto include directory (`<crate_root>/proto` by default), passed as `-I` to `protoc`.
    pub include_dir: PathBuf,
    /// Deterministically sorted `.proto` file paths relative to `include_dir` (using `/` separators).
    pub proto_files: Vec<String>,
}

/// Discovers `.proto` files in `<crate_root>/proto` matching `provides` prefixes (or all
/// `<crate_root>/proto/**/*.proto` files when `provides` is empty).
pub fn discover_protos(crate_root: &Path, provides: &[String]) -> Result<DiscoveredProtos, String> {
    let proto_dir = crate_root.join("proto");
    discover_protos_in_dir(&proto_dir, provides)
}

/// Discovers `.proto` files in `proto_dir` matching `provides` prefixes (or all
/// `proto_dir/**/*.proto` files when `provides` is empty).
pub fn discover_protos_in_dir(
    proto_dir: &Path,
    provides: &[String],
) -> Result<DiscoveredProtos, String> {
    if !proto_dir.exists() {
        return Err(format!(
            "proto directory '{}' does not exist",
            proto_dir.display()
        ));
    }
    if !proto_dir.is_dir() {
        return Err(format!(
            "proto path '{}' is not a directory",
            proto_dir.display()
        ));
    }

    let mut collected = BTreeSet::new();

    if provides.is_empty() {
        collect_proto_files_recursive(proto_dir, proto_dir, &mut collected)?;
    } else {
        for raw_prefix in provides {
            let trimmed = raw_prefix.trim().trim_matches('/');
            if trimmed.is_empty() {
                return Err("provides prefix cannot be empty".to_string());
            }

            let rel_prefix = if !trimmed.ends_with(".proto")
                && !trimmed.contains('/')
                && trimmed.contains('.')
                && !proto_dir.join(trimmed).exists()
            {
                trimmed.replace('.', "/")
            } else {
                trimmed.to_string()
            };

            let candidate_dir = proto_dir.join(&rel_prefix);
            let candidate_file = if rel_prefix.ends_with(".proto") {
                proto_dir.join(&rel_prefix)
            } else {
                proto_dir.join(format!("{rel_prefix}.proto"))
            };

            let mut matched_any = false;

            if candidate_dir.is_dir() {
                collect_proto_files_recursive(proto_dir, &candidate_dir, &mut collected)?;
                matched_any = true;
            }

            if candidate_file.is_file() {
                let rel = relative_forward_slash_path(proto_dir, &candidate_file)?;
                collected.insert(rel);
                matched_any = true;
            }

            if !matched_any {
                return Err(format!(
                    "provides prefix '{raw_prefix}' did not match any directory or .proto file under '{}'",
                    proto_dir.display()
                ));
            }
        }
    }

    if collected.is_empty() {
        return Err(format!(
            "no .proto files found under '{}'",
            proto_dir.display()
        ));
    }

    Ok(DiscoveredProtos {
        include_dir: proto_dir.to_path_buf(),
        proto_files: collected.into_iter().collect(),
    })
}

fn collect_proto_files_recursive(
    root_proto_dir: &Path,
    current_dir: &Path,
    out: &mut BTreeSet<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir).map_err(|e| {
        format!(
            "failed to read directory '{}': {e}",
            current_dir.display()
        )
    })?;

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry_res in entries {
        let entry = entry_res.map_err(|e| {
            format!(
                "failed to read directory entry in '{}': {e}",
                current_dir.display()
            )
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_proto_files_recursive(root_proto_dir, &path, out)?;
        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("proto") {
            let rel = relative_forward_slash_path(root_proto_dir, &path)?;
            out.insert(rel);
        }
    }

    Ok(())
}

fn relative_forward_slash_path(root: &Path, file_path: &Path) -> Result<String, String> {
    let rel = file_path.strip_prefix(root).map_err(|_| {
        format!(
            "file '{}' is not under root '{}'",
            file_path.display(),
            root.display()
        )
    })?;

    let mut parts = Vec::new();
    for comp in rel.components() {
        let s = comp.as_os_str().to_str().ok_or_else(|| {
            format!(
                "proto path '{}' contains non-UTF-8 characters",
                file_path.display()
            )
        })?;
        parts.push(s);
    }

    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(suffix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "cargo_roto_discovery_{}_{}_{}",
                std::process::id(),
                nanos,
                suffix
            ));
            fs::create_dir_all(&dir).expect("create temp test dir");
            Self { path: dir }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn discovers_real_google_protobuf_roto_protos() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-roto is in workspace root");

        let bootstrap_root = workspace_root.join("google-protobuf-roto-bootstrap");
        let discovered_bootstrap =
            discover_protos(&bootstrap_root, &["google/protobuf".to_string()])
                .expect("should discover google-protobuf-roto-bootstrap protos");
        assert_eq!(
            discovered_bootstrap.include_dir,
            bootstrap_root.join("proto")
        );
        assert_eq!(
            discovered_bootstrap.proto_files,
            vec![
                "google/protobuf/compiler/plugin.proto".to_string(),
                "google/protobuf/descriptor.proto".to_string(),
            ]
        );

        let crate_root = workspace_root.join("google-protobuf-roto");
        let discovered = discover_protos(&crate_root, &["google/protobuf".to_string()])
            .expect("should discover google-protobuf-roto protos");

        assert_eq!(discovered.include_dir, crate_root.join("proto"));
        assert_eq!(
            discovered.proto_files,
            vec![
                "google/protobuf/any.proto".to_string(),
                "google/protobuf/api.proto".to_string(),
                "google/protobuf/c_sharp_features.proto".to_string(),
                "google/protobuf/compiler/plugin.proto".to_string(),
                "google/protobuf/cpp_features.proto".to_string(),
                "google/protobuf/descriptor.proto".to_string(),
                "google/protobuf/duration.proto".to_string(),
                "google/protobuf/empty.proto".to_string(),
                "google/protobuf/field_mask.proto".to_string(),
                "google/protobuf/go_features.proto".to_string(),
                "google/protobuf/java_features.proto".to_string(),
                "google/protobuf/source_context.proto".to_string(),
                "google/protobuf/struct.proto".to_string(),
                "google/protobuf/timestamp.proto".to_string(),
                "google/protobuf/type.proto".to_string(),
                "google/protobuf/wrappers.proto".to_string(),
            ]
        );
    }

    #[test]
    fn discovers_filtered_provides_prefixes_and_all_protos_when_provides_empty() {
        let tmp = TempDirGuard::new("multi_ns");
        let proto_root = tmp.path.join("proto");

        let dirs = [
            proto_root.join("google/api"),
            proto_root.join("google/rpc"),
            proto_root.join("google/type"),
            proto_root.join("internal/ignored"),
        ];
        for d in &dirs {
            fs::create_dir_all(d).unwrap();
        }

        fs::write(proto_root.join("google/api/annotations.proto"), "syntax = \"proto3\";\n").unwrap();
        fs::write(proto_root.join("google/api/http.proto"), "syntax = \"proto3\";\n").unwrap();
        fs::write(proto_root.join("google/rpc/status.proto"), "syntax = \"proto3\";\n").unwrap();
        fs::write(proto_root.join("google/type/money.proto"), "syntax = \"proto3\";\n").unwrap();
        fs::write(proto_root.join("google/type/README.md"), "not a proto\n").unwrap();
        fs::write(proto_root.join("internal/ignored/secret.proto"), "syntax = \"proto3\";\n").unwrap();

        // With provides = ["google/type", "google/api"], only those namespaces are returned in sorted order
        let filtered = discover_protos(
            &tmp.path,
            &["google/type".to_string(), "google/api".to_string()],
        )
        .expect("filtered discovery should succeed");

        assert_eq!(
            filtered.proto_files,
            vec![
                "google/api/annotations.proto".to_string(),
                "google/api/http.proto".to_string(),
                "google/type/money.proto".to_string(),
            ]
        );

        // With empty provides, all .proto files under proto/ are discovered in sorted order
        let all = discover_protos(&tmp.path, &[]).expect("unfiltered discovery should succeed");
        assert_eq!(
            all.proto_files,
            vec![
                "google/api/annotations.proto".to_string(),
                "google/api/http.proto".to_string(),
                "google/rpc/status.proto".to_string(),
                "google/type/money.proto".to_string(),
                "internal/ignored/secret.proto".to_string(),
            ]
        );
    }

    #[test]
    fn reports_error_when_proto_dir_or_provides_prefix_missing() {
        let tmp = TempDirGuard::new("missing");
        assert!(discover_protos(&tmp.path, &[]).is_err());

        let proto_root = tmp.path.join("proto/acme/v1");
        fs::create_dir_all(&proto_root).unwrap();
        fs::write(proto_root.join("service.proto"), "syntax = \"proto3\";\n").unwrap();

        assert!(discover_protos(&tmp.path, &["nonexistent/prefix".to_string()]).is_err());
    }
}
