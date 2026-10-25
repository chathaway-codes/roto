use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::CliConfig;
use crate::discovery::{discover_protos_with_inputs, DiscoveredProtos};
use crate::metadata::{resolve_workspace, ResolvedWorkspace};

/// Fully resolved `protoc` invocation parameters for a target crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocInvocation {
    /// Path to the `protoc` binary.
    pub protoc_bin: PathBuf,
    /// Path to the `protoc-gen-roto` plugin binary.
    pub plugin_bin: PathBuf,
    /// Optional path to the `protoc-gen-roto-grpc` plugin binary when `--grpc` or `grpc = true` is enabled.
    pub grpc_plugin_bin: Option<PathBuf>,
    /// Target directory where generated `.rs` files will be written (`--roto_out`).
    pub out_dir: PathBuf,
    /// Comma-separated options passed via `--roto_opt` (`mode=...,extern_path=...`).
    pub roto_opt: String,
    /// Optional comma-separated options passed via `--roto-grpc_opt` when gRPC codegen is enabled.
    pub roto_grpc_opt: Option<String>,
    /// Ordered `-I` include directories (`<target_proto_dir>` followed by additional `proto_paths` and `<dep_root>/proto`).
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
            .or_else(|| workspace.target_out_dir.clone())
            .unwrap_or_else(|| workspace.target_crate_root.join("src"));

        let effective_mode = config.effective_mode(workspace.target_mode);
        let emit_descriptors = config.descriptors || workspace.target_descriptors;
        let mut opt_parts = Vec::with_capacity(2 + workspace.extern_paths.len());
        opt_parts.push(effective_mode.to_option_string());
        if emit_descriptors {
            opt_parts.push("descriptors=true".to_string());
        }
        opt_parts.extend(workspace.extern_path_options());
        let roto_opt = opt_parts.join(",");

        let mut include_dirs = Vec::with_capacity(
            1 + workspace.target_proto_paths.len() + workspace.include_dirs.len(),
        );
        include_dirs.push(protos.include_dir.clone());
        if config.proto_dir.is_none() {
            for target_inc in &workspace.target_proto_paths {
                if !include_dirs.contains(target_inc) {
                    include_dirs.push(target_inc.clone());
                }
            }
        }
        for dep_inc in &workspace.include_dirs {
            if !include_dirs.contains(dep_inc) {
                include_dirs.push(dep_inc.clone());
            }
        }

        Self {
            protoc_bin,
            plugin_bin,
            grpc_plugin_bin: None,
            out_dir,
            roto_opt,
            roto_grpc_opt: None,
            include_dirs,
            proto_files: protos.proto_files.clone(),
        }
    }

    /// Attaches a resolved `protoc-gen-roto-grpc` plugin binary and configures `--roto-grpc_opt`
    /// (`mode=mod,file=grpc.rs` + optional `descriptors=true` / `http=true` + `extern_path` mappings).
    pub fn with_grpc_plugin(
        mut self,
        grpc_plugin_bin: PathBuf,
        workspace: &ResolvedWorkspace,
    ) -> Self {
        let emit_descriptors = self.roto_opt.split(',').any(|p| p == "descriptors=true")
            || workspace.target_descriptors;
        let mut grpc_opt_parts = Vec::with_capacity(4 + workspace.extern_paths.len());
        grpc_opt_parts.push("mode=mod".to_string());
        grpc_opt_parts.push("file=grpc.rs".to_string());
        if emit_descriptors {
            grpc_opt_parts.push("descriptors=true".to_string());
        }
        if workspace.target_http {
            grpc_opt_parts.push("http=true".to_string());
        }
        grpc_opt_parts.extend(workspace.extern_path_options());
        self.grpc_plugin_bin = Some(grpc_plugin_bin);
        self.roto_grpc_opt = Some(grpc_opt_parts.join(","));
        self
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

        if let Some(ref grpc_bin) = self.grpc_plugin_bin {
            let mut grpc_plugin_arg = OsString::from("--plugin=protoc-gen-roto-grpc=");
            grpc_plugin_arg.push(grpc_bin.as_os_str());
            args.push(grpc_plugin_arg);

            let grpc_opt = self
                .roto_grpc_opt
                .as_deref()
                .unwrap_or("mode=mod,file=grpc.rs");
            args.push(OsString::from(format!("--roto-grpc_opt={grpc_opt}")));

            let mut grpc_out_arg = OsString::from("--roto-grpc_out=");
            grpc_out_arg.push(self.out_dir.as_os_str());
            args.push(grpc_out_arg);
        }

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

        if self.grpc_plugin_bin.is_some() && self.out_dir.join("grpc.rs").is_file() {
            let has_grpc_dir = self.out_dir.join("grpc").is_dir();
            for entry_name in ["lib.rs", "mod.rs"] {
                let entry_path = self.out_dir.join(entry_name);
                if entry_path.is_file() {
                    let mut content = fs::read_to_string(&entry_path).map_err(|e| {
                        format!("failed to read generated '{}': {e}", entry_path.display())
                    })?;
                    let already_has_grpc_mod =
                        content.lines().any(|line| line.trim() == "pub mod grpc;");
                    let already_has_services_mod = content
                        .lines()
                        .any(|line| line.trim() == "pub mod services;");
                    let has_inline_grpc_pkg = has_grpc_dir
                        || content
                            .lines()
                            .any(|line| line.trim().starts_with("pub mod grpc {"));
                    if !already_has_grpc_mod && !already_has_services_mod {
                        if !content.is_empty() && !content.ends_with('\n') {
                            content.push('\n');
                        }
                        if !content.is_empty() && !content.ends_with("\n\n") {
                            content.push('\n');
                        }
                        if has_inline_grpc_pkg {
                            content.push_str("#[path = \"grpc.rs\"]\npub mod services;\n");
                        } else {
                            content.push_str("pub mod grpc;\n");
                        }
                        fs::write(&entry_path, content).map_err(|e| {
                            format!("failed to update generated '{}': {e}", entry_path.display())
                        })?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Executes the full `cargo-roto` / `roto-build` pipeline for `config`.
pub fn run(config: &CliConfig) -> Result<(), String> {
    let protoc_bin = resolve_protoc()?;
    let plugin_bin = resolve_plugin(config.plugin.as_deref())?;
    let mut grpc_plugin_bin = if config.grpc || config.http || config.grpc_plugin.is_some() {
        Some(resolve_grpc_plugin(config.grpc_plugin.as_deref())?)
    } else {
        None
    };

    let mut workspace = resolve_workspace(&config.metadata_options())?;
    if config.http {
        workspace.target_http = true;
        workspace.target_grpc = true;
    }
    if workspace.target_grpc && grpc_plugin_bin.is_none() {
        grpc_plugin_bin = Some(resolve_grpc_plugin(config.grpc_plugin.as_deref())?);
    }

    let proto_dirs = match &config.proto_dir {
        Some(custom_proto_dir) => vec![custom_proto_dir.clone()],
        None if !workspace.target_proto_paths.is_empty() => workspace.target_proto_paths.clone(),
        None if workspace.target_proto_dir.is_some() => {
            vec![workspace.target_proto_dir.clone().unwrap()]
        }
        None => vec![workspace.target_crate_root.join("proto")],
    };
    let protos = discover_protos_with_inputs(
        &workspace.target_crate_root,
        &proto_dirs,
        &workspace.target_proto_paths,
        &workspace.target_provides,
        &workspace.target_inputs,
    )?;

    let mut invocation = ProtocInvocation::new(config, &workspace, &protos, protoc_bin, plugin_bin);
    ensure_bundled_google_proto_include(&mut invocation.include_dirs);
    let has_existing_grpc_rs = invocation.out_dir.join("grpc.rs").is_file();
    if (config.grpc
        || config.http
        || workspace.target_grpc
        || config.grpc_plugin.is_some()
        || has_existing_grpc_rs)
        && grpc_plugin_bin.is_none()
    {
        grpc_plugin_bin = Some(resolve_grpc_plugin(config.grpc_plugin.as_deref())?);
    }
    if let Some(grpc_bin) = grpc_plugin_bin {
        invocation = invocation.with_grpc_plugin(grpc_bin, &workspace);
    }
    invocation.execute()
}

fn ensure_bundled_google_proto_include(include_dirs: &mut Vec<PathBuf>) {
    if include_dirs
        .iter()
        .any(|d| d.join("google/api/annotations.proto").is_file())
    {
        return;
    }

    let mut candidates = Vec::with_capacity(2);
    if let Ok(exe) = std::env::current_exe() {
        if let Some(ws_root) = find_workspace_root_from_exe(&exe) {
            candidates.push(ws_root.join("google-protobuf-roto/proto"));
        }
    }
    if let Some(ws_root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        candidates.push(ws_root.join("google-protobuf-roto/proto"));
    }

    for candidate in candidates {
        if candidate.join("google/api/annotations.proto").is_file() {
            let canon = absolutize_path(&candidate);
            if !include_dirs.contains(&canon) && !include_dirs.contains(&candidate) {
                include_dirs.push(canon);
            }
            return;
        }
    }
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
    resolve_named_plugin_with(
        "protoc-gen-roto",
        "--plugin",
        "PROTOC_GEN_ROTO",
        explicit_plugin,
        plugin_env,
        current_exe,
        path_env,
    )
}

/// Resolves the `protoc-gen-roto-grpc` plugin binary using `--grpc-plugin`,
/// `$PROTOC_GEN_ROTO_GRPC`, sibling directory of `current_exe()`, or `$PATH`.
pub fn resolve_grpc_plugin(explicit_plugin: Option<&Path>) -> Result<PathBuf, String> {
    let plugin_env = std::env::var_os("PROTOC_GEN_ROTO_GRPC");
    let current_exe = std::env::current_exe().ok();
    let path_env = std::env::var_os("PATH");

    resolve_grpc_plugin_with(
        explicit_plugin,
        plugin_env.as_deref(),
        current_exe.as_deref(),
        path_env.as_deref(),
    )
}

/// Resolves the `protoc-gen-roto-grpc` binary from explicit inputs (testable without mutating process env).
pub fn resolve_grpc_plugin_with(
    explicit_plugin: Option<&Path>,
    plugin_env: Option<&OsStr>,
    current_exe: Option<&Path>,
    path_env: Option<&OsStr>,
) -> Result<PathBuf, String> {
    resolve_named_plugin_with(
        "protoc-gen-roto-grpc",
        "--grpc-plugin",
        "PROTOC_GEN_ROTO_GRPC",
        explicit_plugin,
        plugin_env,
        current_exe,
        path_env,
    )
}

fn resolve_named_plugin_with(
    plugin_crate_name: &str,
    flag_name: &str,
    env_var_name: &str,
    explicit_plugin: Option<&Path>,
    plugin_env: Option<&OsStr>,
    current_exe: Option<&Path>,
    path_env: Option<&OsStr>,
) -> Result<PathBuf, String> {
    // 1. Explicit CLI flag (`--plugin` or `--grpc-plugin`)
    if let Some(plugin_path) = explicit_plugin {
        if plugin_path.is_file() {
            return Ok(absolutize_path(plugin_path));
        }
        return Err(format!(
            "{flag_name} path '{}' does not exist or is not a file",
            plugin_path.display()
        ));
    }

    // 2. Environment variable (`$PROTOC_GEN_ROTO` or `$PROTOC_GEN_ROTO_GRPC`)
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
                "{env_var_name} environment variable points to '{}', which does not exist or is not a file",
                candidate.display()
            ));
        }
    }

    let exe_name = format!("{plugin_crate_name}{}", std::env::consts::EXE_SUFFIX);

    // 3. Sibling binary in the same directory as `current_exe()` (or parent of `deps/`).
    //    When invoked from a workspace `cargo-roto` binary (outside `deps/`), ensure the
    //    workspace plugin crate is up-to-date before returning the sibling binary so stale
    //    `target/<profile>/protoc-gen-roto*` binaries are never used.
    if let Some(exe) = current_exe {
        let in_deps = exe
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            == Some("deps");
        if !in_deps {
            if let Some(workspace_root) = find_workspace_root_from_exe(exe) {
                let plugin_manifest =
                    workspace_root.join(format!("{plugin_crate_name}/Cargo.toml"));
                if plugin_manifest.is_file() {
                    let cargo_bin =
                        std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
                    let _ = Command::new(cargo_bin)
                        .arg("build")
                        .arg("--manifest-path")
                        .arg(&plugin_manifest)
                        .arg("--quiet")
                        .status();
                }
            }
        }
        if let Some(found) = find_sibling_named_plugin(exe, &exe_name) {
            return Ok(found);
        }
    }

    // 4. `$PATH` lookup
    if let Some(found) = find_in_path(Path::new(&exe_name), path_env) {
        return Ok(found);
    }

    // 5. Workspace auto-build fallback
    if let Some(exe) = current_exe {
        if let Some(workspace_root) = find_workspace_root_from_exe(exe) {
            let plugin_manifest = workspace_root.join(format!("{plugin_crate_name}/Cargo.toml"));
            if plugin_manifest.is_file() {
                let cargo_bin =
                    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
                let status = Command::new(cargo_bin)
                    .arg("build")
                    .arg("--manifest-path")
                    .arg(&plugin_manifest)
                    .arg("--quiet")
                    .status();
                if let Ok(st) = status {
                    if st.success() {
                        if let Some(found) = find_sibling_named_plugin(exe, &exe_name) {
                            return Ok(found);
                        }
                    }
                }
            }
        }
    }

    Err(format!(
        "failed to locate `{plugin_crate_name}`: pass `{flag_name} <PATH>`, set `${env_var_name}`, build `{plugin_crate_name}` in the target directory, or install `{plugin_crate_name}` on `$PATH`"
    ))
}

#[cfg(test)]
fn plugin_exe_name() -> String {
    format!("protoc-gen-roto{}", std::env::consts::EXE_SUFFIX)
}

fn find_sibling_named_plugin(current_exe: &Path, exe_name: &str) -> Option<PathBuf> {
    let exe_dir = current_exe.parent()?;

    let direct_sibling = exe_dir.join(exe_name);
    if direct_sibling.is_file() {
        return Some(absolutize_path(&direct_sibling));
    }

    // When invoked from a Cargo test binary in `target/<profile>/deps/<test_bin>`,
    // check `target/<profile>/<exe_name>`.
    if exe_dir.file_name().and_then(|s| s.to_str()) == Some("deps") {
        if let Some(parent_dir) = exe_dir.parent() {
            let parent_sibling = parent_dir.join(exe_name);
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
        let fake_protoc = tmp
            .path
            .join(format!("protoc{}", std::env::consts::EXE_SUFFIX));
        fs::write(&fake_protoc, b"#!/bin/sh\n").unwrap();

        // 1. Explicit $PROTOC path
        let resolved = resolve_protoc_with_env(Some(fake_protoc.as_os_str()), None).unwrap();
        assert_eq!(resolved, fake_protoc);

        // 2. Fallback to $PATH when $PROTOC is unset
        let resolved_from_path = resolve_protoc_with_env(None, Some(tmp.path.as_os_str())).unwrap();
        assert_eq!(resolved_from_path, fake_protoc);

        // 3. Invalid $PROTOC returns clear error
        let missing_protoc = tmp.path.join("nonexistent-protoc");
        let err =
            resolve_protoc_with_env(Some(missing_protoc.as_os_str()), Some(tmp.path.as_os_str()))
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

        for f in [
            &explicit_bin,
            &env_bin,
            &sibling_bin,
            &path_bin,
            &fake_cargo_roto,
            &fake_test_exe,
        ] {
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
        let r3b = resolve_plugin_with(None, None, Some(&fake_test_exe), Some(path_dir.as_os_str()))
            .unwrap();
        assert_eq!(r3b, sibling_bin);

        // 4. Fallback to `$PATH` when no sibling exists
        let isolated_exe = empty_dir.join("cargo-roto");
        fs::write(&isolated_exe, b"#!/bin/sh\n").unwrap();
        let r4 = resolve_plugin_with(None, None, Some(&isolated_exe), Some(path_dir.as_os_str()))
            .unwrap();
        assert_eq!(r4, path_bin);

        // 5. Error diagnostics when missing
        let err_flag =
            resolve_plugin_with(Some(&empty_dir.join("missing-plugin")), None, None, None)
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

        let err_none =
            resolve_plugin_with(None, None, Some(&isolated_exe), Some(empty_dir.as_os_str()))
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
            grpc: false,
            http: false,
            descriptors: false,
            grpc_plugin: None,
            mode: Some(OutputMode::Lib),
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
            target_proto_dir: None,
            target_proto_paths: Vec::new(),
            target_inputs: Vec::new(),
            target_out_dir: None,
            target_mode: None,
            target_grpc: false,
            target_http: false,
            target_descriptors: false,
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
                OsString::from("--plugin=protoc-gen-roto=/workspace/target/debug/protoc-gen-roto"),
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

        let inv_grpc = inv.with_grpc_plugin(
            PathBuf::from("/workspace/target/debug/protoc-gen-roto-grpc"),
            &workspace,
        );
        assert_eq!(
            inv_grpc.args(),
            vec![
                OsString::from(
                    "--plugin=protoc-gen-roto=/workspace/target/debug/protoc-gen-roto"
                ),
                OsString::from(
                    "--roto_opt=mode=lib,extern_path=google/protobuf=::google_protobuf_roto"
                ),
                OsString::from("--roto_out=/workspace/telemetry/src"),
                OsString::from(
                    "--plugin=protoc-gen-roto-grpc=/workspace/target/debug/protoc-gen-roto-grpc"
                ),
                OsString::from(
                    "--roto-grpc_opt=mode=mod,file=grpc.rs,extern_path=google/protobuf=::google_protobuf_roto"
                ),
                OsString::from("--roto-grpc_out=/workspace/telemetry/src"),
                OsString::from("-I"),
                OsString::from("/workspace/telemetry/proto"),
                OsString::from("-I"),
                OsString::from("/workspace/google-protobuf-roto/proto"),
                OsString::from("telemetry/sensor.proto"),
            ]
        );
    }

    #[test]
    fn honors_metadata_out_dir_mode_and_proto_paths_with_cli_override_precedence() {
        let workspace = ResolvedWorkspace {
            target_package_id: "sensor#0.1.0".to_string(),
            target_package_name: "sensor".to_string(),
            target_manifest_path: PathBuf::from("/workspace/sensor/Cargo.toml"),
            target_crate_root: PathBuf::from("/workspace/sensor"),
            target_provides: Vec::new(),
            target_proto_dir: Some(PathBuf::from("/workspace/sensor/proto")),
            target_proto_paths: vec![
                PathBuf::from("/workspace/sensor/proto"),
                PathBuf::from("/workspace/sensor/vendor_proto"),
            ],
            target_inputs: vec!["proto/sensor.proto".to_string()],
            target_out_dir: Some(PathBuf::from("/workspace/sensor/src/generated")),
            target_mode: Some(OutputMode::Mod),
            target_grpc: true,
            target_http: true,
            target_descriptors: true,
            dependencies: Vec::new(),
            include_dirs: Vec::new(),
            extern_paths: Vec::new(),
        };

        let protos = DiscoveredProtos {
            include_dir: PathBuf::from("/workspace/sensor/proto"),
            proto_files: vec!["sensor.proto".to_string()],
        };

        // 1. Default CLI config uses metadata out_dir, mode, descriptors, http, and proto_paths
        let default_cli = CliConfig::default();
        let inv_meta = ProtocInvocation::new(
            &default_cli,
            &workspace,
            &protos,
            PathBuf::from("/usr/bin/protoc"),
            PathBuf::from("/workspace/target/debug/protoc-gen-roto"),
        )
        .with_grpc_plugin(
            PathBuf::from("/workspace/target/debug/protoc-gen-roto-grpc"),
            &workspace,
        );
        assert_eq!(
            inv_meta.out_dir,
            PathBuf::from("/workspace/sensor/src/generated")
        );
        assert_eq!(inv_meta.roto_opt, "mode=mod,descriptors=true");
        assert_eq!(
            inv_meta.roto_grpc_opt.as_deref(),
            Some("mode=mod,file=grpc.rs,descriptors=true,http=true")
        );
        assert_eq!(
            inv_meta.include_dirs,
            vec![
                PathBuf::from("/workspace/sensor/proto"),
                PathBuf::from("/workspace/sensor/vendor_proto"),
            ]
        );

        // 2. Explicit CLI flags (--out-dir, --mode, --proto-dir) override metadata
        let override_cli = CliConfig {
            manifest_path: None,
            proto_dir: Some(PathBuf::from("/workspace/sensor/cli_proto")),
            out_dir: Some(PathBuf::from("/workspace/sensor/cli_out")),
            plugin: None,
            grpc: false,
            http: false,
            descriptors: false,
            grpc_plugin: None,
            mode: Some(OutputMode::Lib),
            offline: true,
            locked: false,
            frozen: false,
        };
        let cli_protos = DiscoveredProtos {
            include_dir: PathBuf::from("/workspace/sensor/cli_proto"),
            proto_files: vec!["sensor.proto".to_string()],
        };
        let inv_cli = ProtocInvocation::new(
            &override_cli,
            &workspace,
            &cli_protos,
            PathBuf::from("/usr/bin/protoc"),
            PathBuf::from("/workspace/target/debug/protoc-gen-roto"),
        );
        assert_eq!(inv_cli.out_dir, PathBuf::from("/workspace/sensor/cli_out"));
        assert_eq!(inv_cli.roto_opt, "mode=lib,descriptors=true");
        assert_eq!(
            inv_cli.include_dirs,
            vec![PathBuf::from("/workspace/sensor/cli_proto")]
        );
    }
}
