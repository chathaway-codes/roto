use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::json::{parse_json, JsonValue};

/// Options forwarded to `cargo metadata --format-version 1`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataOptions {
    /// Optional `--manifest-path <PATH>` argument.
    pub manifest_path: Option<PathBuf>,
    /// Forward `--offline` to `cargo metadata`.
    pub offline: bool,
    /// Forward `--locked` to `cargo metadata`.
    pub locked: bool,
    /// Forward `--frozen` to `cargo metadata`.
    pub frozen: bool,
}

/// An `extern_path` mapping connecting a `.proto` namespace prefix to a Rust crate path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternPathMapping {
    /// Raw or normalized `.proto` prefix declared in `[package.metadata.roto] provides` (e.g. `"google/protobuf"`).
    pub proto_prefix: String,
    /// Target Rust crate root path (e.g. `"::google_protobuf_roto"`).
    pub rust_path: String,
}

impl ExternPathMapping {
    /// Formats this mapping as a `protoc-gen-roto` option (`extern_path=<proto_prefix>=<rust_path>`).
    pub fn to_option_string(&self) -> String {
        format!("extern_path={}={}", self.proto_prefix, self.rust_path)
    }
}

/// Metadata extracted for a single resolved proto dependency crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtoDependency {
    /// Cargo package ID (`packages[].id`).
    pub package_id: String,
    /// Cargo package name (`packages[].name`).
    pub package_name: String,
    /// Resolved Rust `extern crate` identifier from `resolve.nodes[].deps[].name` (e.g. `google_protobuf_roto`).
    pub extern_crate_name: String,
    /// Qualified Rust crate path (`::<extern_crate_name>`).
    pub rust_crate_path: String,
    /// Path to the dependency's `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Directory containing the dependency's `Cargo.toml` (`manifest_path.parent()`).
    pub crate_root: PathBuf,
    /// Include directory `<dep_root>/proto` passed as `-I` to `protoc`.
    pub proto_include_dir: PathBuf,
    /// Namespace prefixes declared in `[package.metadata.roto] provides = [...]`.
    pub provides: Vec<String>,
    /// Constructed `extern_path` mappings for each prefix in `provides`.
    pub extern_paths: Vec<ExternPathMapping>,
}

/// Resolved target crate and its transitive `[package.metadata.roto]` dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedWorkspace {
    /// Target package ID (`packages[].id`).
    pub target_package_id: String,
    /// Target package name (`packages[].name`).
    pub target_package_name: String,
    /// Path to the target package's `Cargo.toml`.
    pub target_manifest_path: PathBuf,
    /// Root directory of the target package (`target_manifest_path.parent()`).
    pub target_crate_root: PathBuf,
    /// Target crate's own `[package.metadata.roto] provides = [...]` prefixes (may be empty for leaf crates).
    pub target_provides: Vec<String>,
    /// Ordered list of resolved proto dependencies with `[package.metadata.roto] provides`.
    pub dependencies: Vec<ProtoDependency>,
    /// Deduplicated `-I <dep_root>/proto` include directories in deterministic order.
    pub include_dirs: Vec<PathBuf>,
    /// Flattened `extern_path` mappings across all resolved proto dependencies in deterministic order.
    pub extern_paths: Vec<ExternPathMapping>,
}

impl ResolvedWorkspace {
    /// Returns all `extern_path=<prefix>=::<crate_ident>` option strings for `protoc-gen-roto`.
    pub fn extern_path_options(&self) -> Vec<String> {
        self.extern_paths
            .iter()
            .map(ExternPathMapping::to_option_string)
            .collect()
    }
}

/// Runs `cargo metadata --format-version 1` with `options` and resolves the target crate
/// and its `[package.metadata.roto]` dependency graph.
pub fn resolve_workspace(options: &MetadataOptions) -> Result<ResolvedWorkspace, String> {
    let json_out = run_cargo_metadata(options)?;
    resolve_from_metadata_json(&json_out, options.manifest_path.as_deref())
}

/// Executes `cargo metadata --format-version 1` and returns the raw stdout JSON string.
pub fn run_cargo_metadata(options: &MetadataOptions) -> Result<String, String> {
    let cargo_bin = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo_bin);
    cmd.arg("metadata").arg("--format-version").arg("1");

    if let Some(manifest_path) = &options.manifest_path {
        cmd.arg("--manifest-path").arg(manifest_path);
    }
    if options.offline {
        cmd.arg("--offline");
    }
    if options.locked {
        cmd.arg("--locked");
    }
    if options.frozen {
        cmd.arg("--frozen");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to execute `cargo metadata`: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("`cargo metadata` failed: {}", stderr.trim()));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("`cargo metadata` output was not valid UTF-8: {e}"))
}

#[derive(Clone, Debug)]
struct RawPackage {
    id: String,
    name: String,
    manifest_path: PathBuf,
    crate_root: PathBuf,
    provides: Vec<String>,
}

#[derive(Clone, Debug)]
struct RawDepEdge {
    pkg_id: String,
    extern_name: String,
}

/// Parses a `cargo metadata --format-version 1` JSON document and extracts the target package,
/// its `[package.metadata.roto]` configuration, and all transitively reachable proto dependencies.
pub fn resolve_from_metadata_json(
    json_str: &str,
    manifest_path: Option<&Path>,
) -> Result<ResolvedWorkspace, String> {
    let root_val = parse_json(json_str)?;

    let packages_arr = root_val
        .get("packages")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "missing or invalid 'packages' array in cargo metadata".to_string())?;

    let mut packages_by_id: BTreeMap<String, RawPackage> = BTreeMap::new();
    let mut package_order: Vec<String> = Vec::with_capacity(packages_arr.len());

    for pkg_val in packages_arr {
        let id = pkg_val
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "package entry missing string 'id'".to_string())?
            .to_string();
        let name = pkg_val
            .get("name")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| format!("package '{id}' missing string 'name'"))?
            .to_string();
        let manifest_str = pkg_val
            .get("manifest_path")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| format!("package '{id}' missing string 'manifest_path'"))?;
        let pkg_manifest_path = PathBuf::from(manifest_str);
        let crate_root = pkg_manifest_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "package '{id}' manifest_path '{}' has no parent directory",
                    pkg_manifest_path.display()
                )
            })?
            .to_path_buf();

        let provides = extract_roto_provides(pkg_val, &name)?;

        package_order.push(id.clone());
        packages_by_id.insert(
            id.clone(),
            RawPackage {
                id,
                name,
                manifest_path: pkg_manifest_path,
                crate_root,
                provides,
            },
        );
    }

    let resolve_obj = root_val
        .get("resolve")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "missing or null 'resolve' object in cargo metadata".to_string())?;

    let resolve_root_id = resolve_obj.get("root").and_then(JsonValue::as_str);

    let target_pkg_id = select_target_package_id(
        manifest_path,
        resolve_root_id,
        &package_order,
        &packages_by_id,
    )?;

    let target_pkg = packages_by_id
        .get(&target_pkg_id)
        .ok_or_else(|| format!("target package '{target_pkg_id}' not found in 'packages'"))?
        .clone();

    let nodes_arr = resolve_obj
        .get("nodes")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "missing 'resolve.nodes' array in cargo metadata".to_string())?;

    let mut node_edges: BTreeMap<String, Vec<RawDepEdge>> = BTreeMap::new();
    for node_val in nodes_arr {
        let node_id = node_val
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "resolve.nodes entry missing string 'id'".to_string())?
            .to_string();

        let mut edges = Vec::new();
        if let Some(deps_arr) = node_val.get("deps").and_then(JsonValue::as_array) {
            for dep_val in deps_arr {
                let pkg_id = dep_val
                    .get("pkg")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| {
                        format!("resolve.nodes['{node_id}'].deps entry missing string 'pkg'")
                    })?
                    .to_string();
                let extern_name = dep_val
                    .get("name")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| {
                        format!("resolve.nodes['{node_id}'].deps entry missing string 'name'")
                    })?
                    .trim()
                    .to_string();
                if extern_name.is_empty() {
                    return Err(format!(
                        "resolve.nodes['{node_id}'].deps entry for '{pkg_id}' has empty 'name'"
                    ));
                }
                edges.push(RawDepEdge {
                    pkg_id,
                    extern_name,
                });
            }
        } else if let Some(dep_ids) = node_val.get("dependencies").and_then(JsonValue::as_array) {
            for dep_id_val in dep_ids {
                let pkg_id = dep_id_val
                    .as_str()
                    .ok_or_else(|| {
                        format!(
                            "resolve.nodes['{node_id}'].dependencies entry must be a string"
                        )
                    })?
                    .to_string();
                let default_ident = packages_by_id
                    .get(&pkg_id)
                    .map(|p| p.name.replace('-', "_"))
                    .unwrap_or_else(|| pkg_id.replace('-', "_"));
                edges.push(RawDepEdge {
                    pkg_id,
                    extern_name: default_ident,
                });
            }
        }

        node_edges.insert(node_id, edges);
    }

    // Traverse dependencies in BFS order starting from the target package's resolve node so that:
    // 1. Direct dependencies of the target crate are always visited before transitive ones
    //    (preserving any direct rename in the target crate's Cargo.toml).
    // 2. Transitive proto dependencies are also discovered in deterministic order.
    let mut visited_pkgs: BTreeSet<String> = BTreeSet::new();
    visited_pkgs.insert(target_pkg.id.clone());

    let mut queue: VecDeque<RawDepEdge> = VecDeque::new();
    if let Some(root_deps) = node_edges.get(&target_pkg.id) {
        for edge in root_deps {
            queue.push_back(edge.clone());
        }
    }

    let mut dependencies = Vec::new();
    let mut include_dirs = Vec::new();
    let mut extern_paths = Vec::new();

    while let Some(edge) = queue.pop_front() {
        if !visited_pkgs.insert(edge.pkg_id.clone()) {
            continue;
        }

        let dep_pkg = packages_by_id.get(&edge.pkg_id).ok_or_else(|| {
            format!(
                "dependency package '{}' referenced in resolve.nodes not found in packages",
                edge.pkg_id
            )
        })?;

        if !dep_pkg.provides.is_empty() {
            let rust_crate_path = format!("::{}", edge.extern_name);
            let proto_include_dir = dep_pkg.crate_root.join("proto");

            let mut dep_extern_paths = Vec::with_capacity(dep_pkg.provides.len());
            for prefix in &dep_pkg.provides {
                let mapping = ExternPathMapping {
                    proto_prefix: prefix.clone(),
                    rust_path: rust_crate_path.clone(),
                };
                dep_extern_paths.push(mapping.clone());
                extern_paths.push(mapping);
            }

            if !include_dirs.contains(&proto_include_dir) {
                include_dirs.push(proto_include_dir.clone());
            }

            dependencies.push(ProtoDependency {
                package_id: dep_pkg.id.clone(),
                package_name: dep_pkg.name.clone(),
                extern_crate_name: edge.extern_name.clone(),
                rust_crate_path,
                manifest_path: dep_pkg.manifest_path.clone(),
                crate_root: dep_pkg.crate_root.clone(),
                proto_include_dir,
                provides: dep_pkg.provides.clone(),
                extern_paths: dep_extern_paths,
            });
        }

        if let Some(child_edges) = node_edges.get(&edge.pkg_id) {
            for child_edge in child_edges {
                if !visited_pkgs.contains(&child_edge.pkg_id) {
                    queue.push_back(child_edge.clone());
                }
            }
        }
    }

    Ok(ResolvedWorkspace {
        target_package_id: target_pkg.id,
        target_package_name: target_pkg.name,
        target_manifest_path: target_pkg.manifest_path,
        target_crate_root: target_pkg.crate_root,
        target_provides: target_pkg.provides,
        dependencies,
        include_dirs,
        extern_paths,
    })
}

fn select_target_package_id(
    manifest_path: Option<&Path>,
    resolve_root_id: Option<&str>,
    package_order: &[String],
    packages_by_id: &BTreeMap<String, RawPackage>,
) -> Result<String, String> {
    if let Some(req_path) = manifest_path {
        // 1. Direct path match
        for id in package_order {
            let pkg = &packages_by_id[id];
            if pkg.manifest_path == req_path {
                return Ok(id.clone());
            }
        }

        // 2. Canonicalized path match (when paths exist on disk)
        if let Ok(canon_req) = std::fs::canonicalize(req_path) {
            for id in package_order {
                let pkg = &packages_by_id[id];
                if let Ok(canon_pkg) = std::fs::canonicalize(&pkg.manifest_path) {
                    if canon_pkg == canon_req {
                        return Ok(id.clone());
                    }
                }
            }
            if let Some(root_id) = resolve_root_id {
                return Ok(root_id.to_string());
            }
            return Err(format!(
                "manifest path '{}' is a virtual workspace without a root package; pass --manifest-path <member/Cargo.toml>",
                req_path.display()
            ));
        }

        // 3. Suffix match for relative manifest paths (e.g. "google-protobuf-roto/Cargo.toml")
        for id in package_order {
            let pkg = &packages_by_id[id];
            if pkg.manifest_path.ends_with(req_path) {
                return Ok(id.clone());
            }
        }

        // 4. Fallback to resolve.root if present
        if let Some(root_id) = resolve_root_id {
            return Ok(root_id.to_string());
        }

        return Err(format!(
            "no package in cargo metadata matches manifest path '{}'",
            req_path.display()
        ));
    }

    if let Some(root_id) = resolve_root_id {
        return Ok(root_id.to_string());
    }

    if package_order.len() == 1 {
        return Ok(package_order[0].clone());
    }

    Err(
        "cargo metadata did not specify a root package (virtual workspace); pass --manifest-path <Cargo.toml>"
            .to_string(),
    )
}

fn extract_roto_provides(pkg_val: &JsonValue, pkg_name: &str) -> Result<Vec<String>, String> {
    let Some(metadata_val) = pkg_val.get("metadata") else {
        return Ok(Vec::new());
    };
    if metadata_val.is_null() {
        return Ok(Vec::new());
    }
    let Some(metadata_obj) = metadata_val.as_object() else {
        return Ok(Vec::new());
    };
    let Some(roto_val) = metadata_obj.get("roto") else {
        return Ok(Vec::new());
    };
    if roto_val.is_null() {
        return Ok(Vec::new());
    }
    let roto_obj = roto_val.as_object().ok_or_else(|| {
        format!("package '{pkg_name}' has non-table [package.metadata.roto]")
    })?;

    let Some(provides_val) = roto_obj.get("provides") else {
        return Ok(Vec::new());
    };
    if provides_val.is_null() {
        return Ok(Vec::new());
    }
    let provides_arr = provides_val.as_array().ok_or_else(|| {
        format!(
            "package '{pkg_name}' [package.metadata.roto].provides must be an array of strings"
        )
    })?;

    let mut provides = Vec::with_capacity(provides_arr.len());
    for item in provides_arr {
        let raw_prefix = item.as_str().ok_or_else(|| {
            format!(
                "package '{pkg_name}' [package.metadata.roto].provides entries must be strings"
            )
        })?;
        let trimmed = raw_prefix.trim().trim_matches('/');
        if trimmed.is_empty() {
            return Err(format!(
                "package '{pkg_name}' [package.metadata.roto].provides contains an empty prefix"
            ));
        }
        provides.push(trimmed.to_string());
    }

    Ok(provides)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_real_google_protobuf_roto_workspace_metadata() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-roto is in workspace root");
        let manifest_path = workspace_root.join("google-protobuf-roto/Cargo.toml");

        let resolved = resolve_workspace(&MetadataOptions {
            manifest_path: Some(manifest_path.clone()),
            offline: true,
            locked: false,
            frozen: false,
        })
        .expect("should resolve google-protobuf-roto metadata");

        assert_eq!(resolved.target_package_name, "google-protobuf-roto");
        assert_eq!(resolved.target_provides, vec!["google/protobuf".to_string()]);
        assert_eq!(
            resolved.target_crate_root,
            std::fs::canonicalize(workspace_root.join("google-protobuf-roto"))
                .unwrap_or_else(|_| workspace_root.join("google-protobuf-roto"))
        );
        // google-protobuf-roto only depends on roto and roto-runtime, neither of which has [package.metadata.roto]
        assert!(resolved.dependencies.is_empty());
        assert!(resolved.include_dirs.is_empty());
        assert!(resolved.extern_paths.is_empty());
    }

    #[test]
    fn resolves_real_protoc_gen_roto_dependency_on_google_protobuf_roto() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-roto is in workspace root");
        let manifest_path = workspace_root.join("protoc-gen-roto/Cargo.toml");

        let resolved = resolve_workspace(&MetadataOptions {
            manifest_path: Some(manifest_path),
            offline: true,
            locked: false,
            frozen: false,
        })
        .expect("should resolve protoc-gen-roto metadata");

        assert_eq!(resolved.target_package_name, "protoc-gen-roto");
        assert!(resolved.target_provides.is_empty());
        assert_eq!(resolved.dependencies.len(), 1);

        let dep = &resolved.dependencies[0];
        assert_eq!(dep.package_name, "google-protobuf-roto");
        assert_eq!(dep.extern_crate_name, "google_protobuf_roto");
        assert_eq!(dep.rust_crate_path, "::google_protobuf_roto");
        assert_eq!(dep.provides, vec!["google/protobuf".to_string()]);
        assert_eq!(
            resolved.extern_path_options(),
            vec!["extern_path=google/protobuf=::google_protobuf_roto".to_string()]
        );
        assert_eq!(resolved.include_dirs.len(), 1);
        assert!(resolved.include_dirs[0].ends_with("google-protobuf-roto/proto"));
    }

    #[test]
    fn resolves_multi_namespace_provides_transitive_deps_and_renamed_crates() {
        let synthetic_json = r#"
        {
            "packages": [
                {
                    "name": "my-service-proto",
                    "version": "0.1.0",
                    "id": "path+file:///workspace/my-service-proto#0.1.0",
                    "manifest_path": "/workspace/my-service-proto/Cargo.toml",
                    "metadata": {
                        "roto": {
                            "provides": ["acme/service/v1"]
                        }
                    }
                },
                {
                    "name": "googleapis-roto",
                    "version": "0.1.0",
                    "id": "git+https://github.com/example/googleapis-roto#0.1.0",
                    "manifest_path": "/cargo/git/checkouts/googleapis-roto-1234/abcdef/Cargo.toml",
                    "metadata": {
                        "roto": {
                            "provides": ["google/api", "google/rpc", "google/type"]
                        }
                    }
                },
                {
                    "name": "google-protobuf-roto",
                    "version": "0.1.0",
                    "id": "registry+https://github.com/rust-lang/crates.io-index#google-protobuf-roto@0.1.0",
                    "manifest_path": "/cargo/registry/src/index/google-protobuf-roto-0.1.0/Cargo.toml",
                    "metadata": {
                        "roto": {
                            "provides": ["google/protobuf"]
                        }
                    }
                },
                {
                    "name": "roto",
                    "version": "0.1.0",
                    "id": "path+file:///workspace/roto#0.1.0",
                    "manifest_path": "/workspace/roto/Cargo.toml",
                    "metadata": null
                }
            ],
            "resolve": {
                "root": "path+file:///workspace/my-service-proto#0.1.0",
                "nodes": [
                    {
                        "id": "path+file:///workspace/my-service-proto#0.1.0",
                        "deps": [
                            {
                                "name": "gapis_renamed",
                                "pkg": "git+https://github.com/example/googleapis-roto#0.1.0"
                            },
                            {
                                "name": "roto",
                                "pkg": "path+file:///workspace/roto#0.1.0"
                            }
                        ]
                    },
                    {
                        "id": "git+https://github.com/example/googleapis-roto#0.1.0",
                        "deps": [
                            {
                                "name": "google_protobuf_roto",
                                "pkg": "registry+https://github.com/rust-lang/crates.io-index#google-protobuf-roto@0.1.0"
                            }
                        ]
                    },
                    {
                        "id": "registry+https://github.com/rust-lang/crates.io-index#google-protobuf-roto@0.1.0",
                        "deps": []
                    },
                    {
                        "id": "path+file:///workspace/roto#0.1.0",
                        "deps": []
                    }
                ]
            },
            "version": 1
        }
        "#;

        let resolved = resolve_from_metadata_json(
            synthetic_json,
            Some(Path::new("/workspace/my-service-proto/Cargo.toml")),
        )
        .expect("synthetic multi-crate graph should resolve");

        assert_eq!(resolved.target_package_name, "my-service-proto");
        assert_eq!(
            resolved.target_crate_root,
            PathBuf::from("/workspace/my-service-proto")
        );
        assert_eq!(resolved.target_provides, vec!["acme/service/v1"]);

        assert_eq!(resolved.dependencies.len(), 2);

        // 1. Direct dependency with renamed crate in Cargo.toml (`gapis_renamed`) and 3 provides namespaces
        let gapis = &resolved.dependencies[0];
        assert_eq!(gapis.package_name, "googleapis-roto");
        assert_eq!(gapis.extern_crate_name, "gapis_renamed");
        assert_eq!(gapis.rust_crate_path, "::gapis_renamed");
        assert_eq!(
            gapis.proto_include_dir,
            PathBuf::from("/cargo/git/checkouts/googleapis-roto-1234/abcdef/proto")
        );
        assert_eq!(
            gapis.provides,
            vec!["google/api", "google/rpc", "google/type"]
        );

        // 2. Transitive dependency (`google-protobuf-roto` -> `google_protobuf_roto`)
        let wkt = &resolved.dependencies[1];
        assert_eq!(wkt.package_name, "google-protobuf-roto");
        assert_eq!(wkt.extern_crate_name, "google_protobuf_roto");
        assert_eq!(wkt.rust_crate_path, "::google_protobuf_roto");
        assert_eq!(
            wkt.proto_include_dir,
            PathBuf::from("/cargo/registry/src/index/google-protobuf-roto-0.1.0/proto")
        );
        assert_eq!(wkt.provides, vec!["google/protobuf"]);

        assert_eq!(
            resolved.include_dirs,
            vec![
                PathBuf::from("/cargo/git/checkouts/googleapis-roto-1234/abcdef/proto"),
                PathBuf::from("/cargo/registry/src/index/google-protobuf-roto-0.1.0/proto"),
            ]
        );

        assert_eq!(
            resolved.extern_path_options(),
            vec![
                "extern_path=google/api=::gapis_renamed",
                "extern_path=google/rpc=::gapis_renamed",
                "extern_path=google/type=::gapis_renamed",
                "extern_path=google/protobuf=::google_protobuf_roto",
            ]
        );
    }
}
