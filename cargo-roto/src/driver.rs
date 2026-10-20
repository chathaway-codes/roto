use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::CliConfig;
use crate::discovery::{discover_protos, discover_protos_in_dir, DiscoveredProtos};
use crate::metadata::{resolve_workspace, ResolvedWorkspace};

/// Fully resolved `protoc` invocation parameters for a target crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocInvocation {
    /// Path to the `protoc` binary.
    pub protoc_bin: PathBuf,
    /// Path to the `protoc-gen-roto` plugin binary.
    pub plugin_bin: PathBuf,
    /// Target directory where generated `.rs` files will be written (`--roto_out`).
    pub out_dir: PathBuf,
    /// Comma-separated options passed via `--roto_opt` (`mode=...,extern_path=...`).
    pub roto_opt: String,
    /// Ordered `-I` include directories (`<target_proto_dir>` followed by `<dep_root>/proto`).
    pub include_dirs: Vec<PathBuf>,
    /// Deterministically sorted `.proto` files relative to `<target_proto_dir>`.
    pub proto_files: Vec<String>,
}

impl ProtocInvocation {
    /// Constructs a [`ProtocInvocation`] from resolved workspace metadata, discovered `.proto` files,
    /// CLI configuration, and resolved tool binaries.
    pub fn new(
        config: &CliConfig,
        workspace: &ResolvedWorkspace,
        protos: &DiscoveredProtos,
        protoc_bin: PathBuf,
        plugin_bin: PathBuf,
    ) -> Self {
        let out_dir = config
            .out_dir
            .clone()
            .unwrap_or_else(|| workspace.target_crate_root.join("src"));

        let mut opt_parts = Vec::with_capacity(1 + workspace.extern_paths.len());
        opt_parts.push(config.mode.to_option_string());
        opt_parts.extend(workspace.extern_path_options());
        let roto_opt = opt_parts.join(",");

        let mut include_dirs = Vec::with_capacity(1 + workspace.include_dirs.len());
        include_dirs.push(protos.include_dir.clone());
        for dep_inc in &workspace.include_dirs {
            if !include_dirs.contains(dep_inc) {
                include_dirs.push(dep_inc.clone());
            }
        }

        Self {
            protoc_bin,
            plugin_bin,
            out_dir,
            roto_opt,
            include_dirs,
            proto_files: protos.proto_files.clone(),
        }
    }

    /// Returns the CLI arguments passed to `protoc`.
    pub fn args(&self) -> Vec<OsString> {
        let mut args = Vec::new();

        let mut plugin_arg = OsString::from("--plugin=protoc-gen-roto=");
        plugin_arg.push(self.plugin_bin.as_os_str());
        args.push(plugin_arg);

        args.push(OsString::from(format!("--roto_opt={}", self.roto_opt)));

        let mut out_arg = OsString::from("--roto_out=");
        out_arg.push(self.out_dir.as_os_str());
        args.push(out_arg);

        for inc_dir in &self.include_dirs {
            args.push(OsString::from("-I"));
            args.push(inc_dir.as_os_str().to_os_string());
        }

        for proto in &self.proto_files {
            args.push(OsString::from(proto));
        }

        args
    }

    /// Ensures `self.out_dir` exists and executes `protoc`, enforcing the Unix Rule of Silence
    /// on success and returning a descriptive error on failure.
    pub fn execute(&self) -> Result<(), String> {
        fs::create_dir_all(&self.out_dir).map_err(|e| {
            format!(
                "failed to create output directory '{}': {e}",
                self.out_dir.display()
            )
        })?;

        let output = Command::new(&self.protoc_bin)
            .args(self.args())
            .output()
            .map_err(|e| {
                format!(
                    "failed to execute `protoc` ('{}'): {e}",
                    self.protoc_bin.display()
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.trim()
            } else if !stdout.trim().is_empty() {
                stdout.trim()
            } else {
                "protoc exited with non-zero status and no output"
            };
            return Err(format!("`protoc` failed: {detail}"));
        }

        Ok(())
    }
}

/// Executes the full `cargo-roto` / `roto-build` pipeline for `config`.
pub fn run(config: &CliConfig) -> Result<(), String> {
    let protoc_bin = resolve_protoc()?;
    let plugin_bin = resolve_plugin(config.plugin.as_deref())?;

    let workspace = resolve_workspace(&config.metadata_options())?;
    let protos = match &config.proto_dir {
        Some(custom_proto_dir) => {
            discover_protos_in_dir(custom_proto_dir, &workspace.target_provides)?
        }
        None => discover_protos(&workspace.target_crate_root, &workspace.target_provides)?,
    };

    let invocation = ProtocInvocation::new(config, &workspace, &protos, protoc_bin, plugin_bin);
    invocation.execute()
}

/// Resolves the `protoc` binary using `$PROTOC` or `$PATH`.
pub fn resolve_protoc() -> Result<PathBuf, String> {
    resolve_protoc_with_env(
        std::env::var_os("PROTOC").as_deref(),
        std::env::var_os("PATH").as_deref(),
    )
}

/// Resolves the `protoc` binary from explicit environment variable values (testable without mutating process env).
pub fn resolve_protoc_with_env(
    protoc_env: Option<&OsStr>,
    path_env: Option<&OsStr>,
) -> Result<PathBuf, String> {
    if let Some(raw_protoc) = protoc_env {
        if !raw_protoc.is_empty() {
            let candidate = PathBuf::from(raw_protoc);
            if candidate.is_file() {
                return Ok(absolutize_path(&candidate));
            }
            if !has_path_separator(&candidate) {
                if let Some(found) = find_in_path(&candidate, path_env) {
                    return Ok(found);
                }
            }
            return Err(format!(
                "PROTOC environment variable points to '{}', which does not exist or is not a file",
                candidate.display()
            ));
        }
    }

    let exe_name = format!("protoc{}", std::env::consts::EXE_SUFFIX);
    if let Some(found) = find_in_path(Path::new(&exe_name), path_env) {
        return Ok(found);
    }

    Err(
        "failed to locate `protoc`: set `$PROTOC` or ensure `protoc` is installed on `$PATH`"
            .to_string(),
    )
}

/// Resolves the `protoc-gen-roto` plugin binary using `--plugin`, `$PROTOC_GEN_ROTO`,
/// sibling directory of `current_exe()`, or `$PATH`.
pub fn resolve_plugin(explicit_plugin: Option<&Path>) -> Result<PathBuf, String> {
    let plugin_env = std::env::var_os("PROTOC_GEN_ROTO");
    let current_exe = std::env::current_exe().ok();
    let path_env = std::env::var_os("PATH");

    resolve_plugin_with(
        explicit_plugin,
        plugin_env.as_deref(),
        current_exe.as_deref(),
        path_env.as_deref(),
    )
}

/// Resolves the `protoc-gen-roto` binary from explicit inputs (testable without mutating process env).
pub fn resolve_plugin_with(
    explicit_plugin: Option<&Path>,
    plugin_env: Option<&OsStr>,
    current_exe: Option<&Path>,
    path_env: Option<&OsStr>,
) -> Result<PathBuf, String> {
    // 1. `--plugin <PATH>` CLI argument
    if let Some(plugin_path) = explicit_plugin {
        if plugin_path.is_file() {
            return Ok(absolutize_path(plugin_path));
        }
        return Err(format!(
            "--plugin path '{}' does not exist or is not a file",
            plugin_path.display()
        ));
    }

    // 2. `$PROTOC_GEN_ROTO` environment variable
    if let Some(raw_env) = plugin_env {
        if !raw_env.is_empty() {
            let candidate = PathBuf::from(raw_env);
            if candidate.is_file() {
                return Ok(absolutize_path(&candidate));
            }
            if !has_path_separator(&candidate) {
                if let Some(found) = find_in_path(&candidate, path_env) {
                    return Ok(found);
                }
            }
            return Err(format!(
                "PROTOC_GEN_ROTO environment variable points to '{}', which does not exist or is not a file",
                candidate.display()
            ));
        }
    }

    // 3. Sibling binary in the same directory as `current_exe()` (or parent of `deps/`)
    if let Some(exe) = current_exe {
        if let Some(found) = find_sibling_plugin(exe) {
            return Ok(found);
        }
    }

    // 4. `$PATH` lookup
    let exe_name = plugin_exe_name();
    if let Some(found) = find_in_path(Path::new(&exe_name), path_env) {
        return Ok(found);
    }

    // 5. If running inside the `roto` workspace (where `<workspace_root>/protoc-gen-roto/Cargo.toml`
    //    exists above `current_exe`), build `protoc-gen-roto` so `cargo test` / `cargo run` works
    //    even before `protoc-gen-roto` has been manually built.
    if let Some(exe) = current_exe {
        if let Some(workspace_root) = find_workspace_root_from_exe(exe) {
            let plugin_manifest = workspace_root.join("protoc-gen-roto/Cargo.toml");
            let cargo_bin = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
            let status = Command::new(cargo_bin)
                .arg("build")
                .arg("--manifest-path")
                .arg(&plugin_manifest)
                .arg("--quiet")
                .status();
            if let Ok(st) = status {
                if st.success() {
                    if let Some(found) = find_sibling_plugin(exe) {
                        return Ok(found);
                    }
                }
            }
        }
    }

    Err(
        "failed to locate `protoc-gen-roto`: pass `--plugin <PATH>`, set `$PROTOC_GEN_ROTO`, build `protoc-gen-roto` in the target directory, or install `protoc-gen-roto` on `$PATH`"
            .to_string(),
    )
}

fn plugin_exe_name() -> String {
    format!("protoc-gen-roto{}", std::env::consts::EXE_SUFFIX)
}

fn find_sibling_plugin(current_exe: &Path) -> Option<PathBuf> {
    let exe_dir = current_exe.parent()?;
    let exe_name = plugin_exe_name();

    let direct_sibling = exe_dir.join(&exe_name);
    if direct_sibling.is_file() {
        return Some(absolutize_path(&direct_sibling));
    }

    // When invoked from a Cargo test binary in `target/<profile>/deps/<test_bin>`,
    // check `target/<profile>/protoc-gen-roto`.
    if exe_dir.file_name().and_then(|s| s.to_str()) == Some("deps") {
        if let Some(parent_dir) = exe_dir.parent() {
            let parent_sibling = parent_dir.join(&exe_name);
            if parent_sibling.is_file() {
                return Some(absolutize_path(&parent_sibling));
            }
        }
    }

    None
}

fn find_workspace_root_from_exe(current_exe: &Path) -> Option<PathBuf> {
    let mut ancestor = current_exe.parent()?;
    for _ in 0..4 {
        let candidate_manifest = ancestor.join("protoc-gen-roto/Cargo.toml");
        if candidate_manifest.is_file() {
            return Some(ancestor.to_path_buf());
        }
        ancestor = ancestor.parent()?;
    }
    None
}

fn find_in_path(binary_name: &Path, path_env: Option<&OsStr>) -> Option<PathBuf> {
    let path_val = path_env?;
    for dir in std::env::split_paths(path_val) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(absolutize_path(&candidate));
        }
    }
    None
}

fn has_path_separator(path: &Path) -> bool {
    path.components().count() > 1
}

fn absolutize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Ok(canon) = fs::canonicalize(path) {
        canon
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(path)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputMode;
    use crate::metadata::{ExternPathMapping, ProtoDependency};
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
                "cargo_roto_driver_{}_{}_{}",
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
    fn resolves_protoc_via_env_and_path_and_reports_errors() {
        let tmp = TempDirGuard::new("protoc_discovery");
        let fake_protoc = tmp.path.join(format!("protoc{}", std::env::consts::EXE_SUFFIX));
        fs::write(&fake_protoc, b"#!/bin/sh\n").unwrap();

        // 1. Explicit $PROTOC path
        let resolved = resolve_protoc_with_env(Some(fake_protoc.as_os_str()), None).unwrap();
        assert_eq!(resolved, fake_protoc);

        // 2. Fallback to $PATH when $PROTOC is unset
        let resolved_from_path =
            resolve_protoc_with_env(None, Some(tmp.path.as_os_str())).unwrap();
        assert_eq!(resolved_from_path, fake_protoc);

        // 3. Invalid $PROTOC returns clear error
        let missing_protoc = tmp.path.join("nonexistent-protoc");
        let err = resolve_protoc_with_env(Some(missing_protoc.as_os_str()), Some(tmp.path.as_os_str()))
            .unwrap_err();
        assert!(err.contains("PROTOC environment variable points to"));

        // 4. Empty $PROTOC and empty $PATH returns clear error
        let empty_dir = tmp.path.join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        let err_no_path = resolve_protoc_with_env(None, Some(empty_dir.as_os_str())).unwrap_err();
        assert!(err_no_path.contains("failed to locate `protoc`"));
    }

    #[test]
    fn resolves_plugin_via_flag_env_sibling_deps_and_path() {
        let tmp = TempDirGuard::new("plugin_discovery");
        let exe_name = plugin_exe_name();

        let explicit_dir = tmp.path.join("explicit");
        let env_dir = tmp.path.join("env");
        let target_debug_dir = tmp.path.join("target/debug");
        let target_deps_dir = target_debug_dir.join("deps");
        let path_dir = tmp.path.join("path_bin");
        let empty_dir = tmp.path.join("empty");

        for d in [
            &explicit_dir,
            &env_dir,
            &target_debug_dir,
            &target_deps_dir,
            &path_dir,
            &empty_dir,
        ] {
            fs::create_dir_all(d).unwrap();
        }

        let explicit_bin = explicit_dir.join(&exe_name);
        let env_bin = env_dir.join(&exe_name);
        let sibling_bin = target_debug_dir.join(&exe_name);
        let path_bin = path_dir.join(&exe_name);
        let fake_cargo_roto = target_debug_dir.join("cargo-roto");
        let fake_test_exe = target_deps_dir.join("cli_driver-12345");

        for f in [&explicit_bin, &env_bin, &sibling_bin, &path_bin, &fake_cargo_roto, &fake_test_exe] {
            fs::write(f, b"#!/bin/sh\n").unwrap();
        }

        // 1. `--plugin` takes highest precedence
        let r1 = resolve_plugin_with(
            Some(&explicit_bin),
            Some(env_bin.as_os_str()),
            Some(&fake_cargo_roto),
            Some(path_dir.as_os_str()),
        )
        .unwrap();
        assert_eq!(r1, explicit_bin);

        // 2. `$PROTOC_GEN_ROTO` takes second precedence
        let r2 = resolve_plugin_with(
            None,
            Some(env_bin.as_os_str()),
            Some(&fake_cargo_roto),
            Some(path_dir.as_os_str()),
        )
        .unwrap();
        assert_eq!(r2, env_bin);

        // 3a. Direct sibling of `current_exe()` in `target/debug/`
        let r3a = resolve_plugin_with(
            None,
            None,
            Some(&fake_cargo_roto),
            Some(path_dir.as_os_str()),
        )
        .unwrap();
        assert_eq!(r3a, sibling_bin);

        // 3b. Parent of `target/debug/deps/` when invoked from a test executable
        let r3b = resolve_plugin_with(
            None,
            None,
            Some(&fake_test_exe),
            Some(path_dir.as_os_str()),
        )
        .unwrap();
        assert_eq!(r3b, sibling_bin);

        // 4. Fallback to `$PATH` when no sibling exists
        let isolated_exe = empty_dir.join("cargo-roto");
        fs::write(&isolated_exe, b"#!/bin/sh\n").unwrap();
        let r4 = resolve_plugin_with(
            None,
            None,
            Some(&isolated_exe),
            Some(path_dir.as_os_str()),
        )
        .unwrap();
        assert_eq!(r4, path_bin);

        // 5. Error diagnostics when missing
        let err_flag = resolve_plugin_with(
            Some(&empty_dir.join("missing-plugin")),
            None,
            None,
            None,
        )
        .unwrap_err();
        assert!(err_flag.contains("--plugin path"));

        let err_env = resolve_plugin_with(
            None,
            Some(empty_dir.join("missing-env-plugin").as_os_str()),
            None,
            None,
        )
        .unwrap_err();
        assert!(err_env.contains("PROTOC_GEN_ROTO environment variable"));

        let err_none = resolve_plugin_with(
            None,
            None,
            Some(&isolated_exe),
            Some(empty_dir.as_os_str()),
        )
        .unwrap_err();
        assert!(err_none.contains("failed to locate `protoc-gen-roto`"));
    }

    #[test]
    fn constructs_protoc_invocation_with_extern_paths_and_include_dirs() {
        let config = CliConfig {
            manifest_path: Some(PathBuf::from("/workspace/telemetry/Cargo.toml")),
            proto_dir: None,
            out_dir: None,
            plugin: None,
            mode: OutputMode::Lib,
            offline: true,
            locked: false,
            frozen: false,
        };

        let workspace = ResolvedWorkspace {
            target_package_id: "telemetry#0.1.0".to_string(),
            target_package_name: "telemetry".to_string(),
            target_manifest_path: PathBuf::from("/workspace/telemetry/Cargo.toml"),
            target_crate_root: PathBuf::from("/workspace/telemetry"),
            target_provides: vec!["telemetry".to_string()],
            dependencies: vec![ProtoDependency {
                package_id: "google-protobuf-roto#0.1.0".to_string(),
                package_name: "google-protobuf-roto".to_string(),
                extern_crate_name: "google_protobuf_roto".to_string(),
                rust_crate_path: "::google_protobuf_roto".to_string(),
                manifest_path: PathBuf::from("/workspace/google-protobuf-roto/Cargo.toml"),
                crate_root: PathBuf::from("/workspace/google-protobuf-roto"),
                proto_include_dir: PathBuf::from("/workspace/google-protobuf-roto/proto"),
                provides: vec!["google/protobuf".to_string()],
                extern_paths: vec![ExternPathMapping {
                    proto_prefix: "google/protobuf".to_string(),
                    rust_path: "::google_protobuf_roto".to_string(),
                }],
            }],
            include_dirs: vec![PathBuf::from("/workspace/google-protobuf-roto/proto")],
            extern_paths: vec![ExternPathMapping {
                proto_prefix: "google/protobuf".to_string(),
                rust_path: "::google_protobuf_roto".to_string(),
            }],
        };

        let protos = DiscoveredProtos {
            include_dir: PathBuf::from("/workspace/telemetry/proto"),
            proto_files: vec!["telemetry/sensor.proto".to_string()],
        };

        let inv = ProtocInvocation::new(
            &config,
            &workspace,
            &protos,
            PathBuf::from("/usr/bin/protoc"),
            PathBuf::from("/workspace/target/debug/protoc-gen-roto"),
        );

        assert_eq!(inv.out_dir, PathBuf::from("/workspace/telemetry/src"));
        assert_eq!(
            inv.roto_opt,
            "mode=lib,extern_path=google/protobuf=::google_protobuf_roto"
        );
        assert_eq!(
            inv.args(),
            vec![
                OsString::from(
                    "--plugin=protoc-gen-roto=/workspace/target/debug/protoc-gen-roto"
                ),
                OsString::from(
                    "--roto_opt=mode=lib,extern_path=google/protobuf=::google_protobuf_roto"
                ),
                OsString::from("--roto_out=/workspace/telemetry/src"),
                OsString::from("-I"),
                OsString::from("/workspace/telemetry/proto"),
                OsString::from("-I"),
                OsString::from("/workspace/google-protobuf-roto/proto"),
                OsString::from("telemetry/sensor.proto"),
            ]
        );
    }
}
