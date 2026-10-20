use std::path::{Path, PathBuf};

use crate::metadata::MetadataOptions;

/// Code generation output mode forwarded to `protoc-gen-roto`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputMode {
    /// Emit a single cohesive `lib.rs` containing the nested package module hierarchy (`mode=lib`).
    #[default]
    Lib,
    /// Emit per-package module files alongside `lib.rs` (`mode=mod`).
    Mod,
}

impl OutputMode {
    /// Parses `"lib"` or `"mod"` into an [`OutputMode`].
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim() {
            "lib" => Ok(Self::Lib),
            "mod" => Ok(Self::Mod),
            other => Err(format!(
                "invalid `--mode` value '{other}': expected 'lib' or 'mod'"
            )),
        }
    }

    /// Returns the canonical option value (`"lib"` or `"mod"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lib => "lib",
            Self::Mod => "mod",
        }
    }

    /// Formats this mode as a `protoc-gen-roto` option (`mode=lib` or `mode=mod`).
    pub fn to_option_string(self) -> String {
        format!("mode={}", self.as_str())
    }
}

/// Parsed CLI configuration for `cargo-roto` and `roto-build`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliConfig {
    /// Path to target crate's `Cargo.toml` (`--manifest-path <PATH>`).
    /// Defaults to `./Cargo.toml` in the current working directory when `None`.
    pub manifest_path: Option<PathBuf>,
    /// Override source `.proto` directory (`--proto-dir <DIR>`).
    /// Defaults to `<crate_root>/proto` when `None`.
    pub proto_dir: Option<PathBuf>,
    /// Override output directory passed to `--roto_out` (`--out-dir <DIR>`).
    /// Defaults to `<crate_root>/src` when `None`.
    pub out_dir: Option<PathBuf>,
    /// Explicit path to the `protoc-gen-roto` binary (`--plugin <PATH>`).
    pub plugin: Option<PathBuf>,
    /// Enable opt-in gRPC service and client code generation (`--grpc`).
    pub grpc: bool,
    /// Explicit path to the `protoc-gen-roto-grpc` binary (`--grpc-plugin <PATH>`).
    pub grpc_plugin: Option<PathBuf>,
    /// Output mode forwarded to `protoc-gen-roto` (`--mode <lib|mod>`).
    pub mode: OutputMode,
    /// Forward `--offline` to `cargo metadata`.
    pub offline: bool,
    /// Forward `--locked` to `cargo metadata`.
    pub locked: bool,
    /// Forward `--frozen` to `cargo metadata`.
    pub frozen: bool,
}

impl CliConfig {
    /// Returns the effective manifest path, defaulting to `./Cargo.toml` when omitted.
    pub fn effective_manifest_path(&self) -> PathBuf {
        self.manifest_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("Cargo.toml"))
    }

    /// Converts CLI flags into [`MetadataOptions`] for `cargo metadata`.
    pub fn metadata_options(&self) -> MetadataOptions {
        MetadataOptions {
            manifest_path: Some(self.effective_manifest_path()),
            offline: self.offline,
            locked: self.locked,
            frozen: self.frozen,
        }
    }
}

/// Top-level parsed CLI command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliCommand {
    /// Execute code generation with the parsed configuration.
    Run(CliConfig),
    /// Display usage information to `stdout` and exit `0`.
    Help(String),
}

/// Parses CLI arguments from [`std::env::args`].
pub fn parse_env_args() -> Result<CliCommand, String> {
    parse_args(std::env::args())
}

/// Parses an iterator of CLI arguments (including `argv[0]`).
pub fn parse_args<I, S>(args: I) -> Result<CliCommand, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let raw_args: Vec<String> = args.into_iter().map(Into::into).collect();
    let bin_name = raw_args
        .first()
        .and_then(|s| Path::new(s).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("cargo-roto");

    let mut idx = if raw_args.is_empty() { 0 } else { 1 };
    let mut invoked_as_cargo_subcommand = false;

    // Handle Cargo custom subcommand invocation (`cargo roto ...` passes `"roto"` as argv[1])
    if idx < raw_args.len() && raw_args[idx] == "roto" {
        invoked_as_cargo_subcommand = true;
        idx += 1;
    }

    // Handle optional subcommand verbs (`build` or `generate`)
    if idx < raw_args.len() && matches!(raw_args[idx].as_str(), "build" | "generate") {
        idx += 1;
    }

    let mut config = CliConfig::default();

    while idx < raw_args.len() {
        let arg = &raw_args[idx];

        if arg == "-h" || arg == "--help" {
            return Ok(CliCommand::Help(usage_text(
                bin_name,
                invoked_as_cargo_subcommand,
            )));
        } else if arg == "--offline" {
            config.offline = true;
            idx += 1;
        } else if arg == "--locked" {
            config.locked = true;
            idx += 1;
        } else if arg == "--frozen" {
            config.frozen = true;
            idx += 1;
        } else if arg == "--grpc" {
            config.grpc = true;
            idx += 1;
        } else if let Some((val, consumed)) =
            take_flag_value(&raw_args, idx, "--manifest-path")?
        {
            config.manifest_path = Some(PathBuf::from(val));
            idx += consumed;
        } else if let Some((val, consumed)) = take_flag_value(&raw_args, idx, "--proto-dir")? {
            config.proto_dir = Some(PathBuf::from(val));
            idx += consumed;
        } else if let Some((val, consumed)) = take_flag_value(&raw_args, idx, "--out-dir")? {
            config.out_dir = Some(PathBuf::from(val));
            idx += consumed;
        } else if let Some((val, consumed)) = take_flag_value(&raw_args, idx, "--plugin")? {
            config.plugin = Some(PathBuf::from(val));
            idx += consumed;
        } else if let Some((val, consumed)) = take_flag_value(&raw_args, idx, "--grpc-plugin")? {
            config.grpc_plugin = Some(PathBuf::from(val));
            idx += consumed;
        } else if let Some((val, consumed)) = take_flag_value(&raw_args, idx, "--mode")? {
            config.mode = OutputMode::parse(&val)?;
            idx += consumed;
        } else if arg.starts_with('-') {
            return Err(format!("unknown flag '{arg}' (pass --help for usage)"));
        } else {
            return Err(format!(
                "unexpected positional argument '{arg}' (pass --help for usage)"
            ));
        }
    }

    Ok(CliCommand::Run(config))
}

fn take_flag_value(
    args: &[String],
    idx: usize,
    flag: &str,
) -> Result<Option<(String, usize)>, String> {
    let current = &args[idx];
    if current == flag {
        let next = args
            .get(idx + 1)
            .ok_or_else(|| format!("flag '{flag}' requires a value"))?;
        if next.is_empty() {
            return Err(format!("flag '{flag}' value cannot be empty"));
        }
        return Ok(Some((next.clone(), 2)));
    }

    if let Some(rest) = current.strip_prefix(flag) {
        if let Some(eq_val) = rest.strip_prefix('=') {
            if eq_val.is_empty() {
                return Err(format!("flag '{flag}' value cannot be empty"));
            }
            return Ok(Some((eq_val.to_string(), 1)));
        }
    }

    Ok(None)
}

fn usage_text(bin_name: &str, invoked_as_cargo_subcommand: bool) -> String {
    let invocation = if invoked_as_cargo_subcommand || bin_name.starts_with("cargo-roto") {
        "cargo roto [build|generate] [OPTIONS]"
    } else {
        "roto-build [build|generate] [OPTIONS]"
    };

    format!(
        "\
Usage: {invocation}

Options:
  --manifest-path <PATH>  Path to target Cargo.toml (default: ./Cargo.toml)
  --proto-dir <DIR>       Override source .proto directory (default: <crate_root>/proto)
  --out-dir <DIR>         Override output directory for --roto_out (default: <crate_root>/src)
  --plugin <PATH>         Explicit path to protoc-gen-roto binary
  --grpc                  Enable opt-in gRPC service and client code generation
  --grpc-plugin <PATH>    Explicit path to protoc-gen-roto-grpc binary
  --mode <lib|mod>        Generator output mode: 'lib' (default) or 'mod'
  --offline               Forward --offline to cargo metadata
  --locked                Forward --locked to cargo metadata
  --frozen                Forward --frozen to cargo metadata
  -h, --help              Print this help message
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_subcommand_and_direct_invocations() {
        // `cargo roto`
        let cmd = parse_args(["cargo-roto", "roto"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        // `cargo roto build`
        let cmd = parse_args(["cargo-roto", "roto", "build"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        // `cargo roto generate`
        let cmd = parse_args(["cargo-roto", "roto", "generate"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        // `cargo-roto` direct
        let cmd = parse_args(["cargo-roto"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        // `cargo-roto build`
        let cmd = parse_args(["cargo-roto", "build"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        // `roto-build` direct and subcommands
        let cmd = parse_args(["roto-build"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        let cmd = parse_args(["roto-build", "build"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));

        let cmd = parse_args(["roto-build", "generate"]).unwrap();
        assert_eq!(cmd, CliCommand::Run(CliConfig::default()));
    }

    #[test]
    fn parses_space_and_equals_flags() {
        let cmd = parse_args([
            "cargo-roto",
            "roto",
            "build",
            "--manifest-path",
            "crates/sensor/Cargo.toml",
            "--proto-dir=custom_proto",
            "--out-dir",
            "custom_src",
            "--plugin=/usr/local/bin/protoc-gen-roto",
            "--grpc",
            "--grpc-plugin=/usr/local/bin/protoc-gen-roto-grpc",
            "--mode",
            "mod",
            "--offline",
            "--locked",
            "--frozen",
        ])
        .unwrap();

        assert_eq!(
            cmd,
            CliCommand::Run(CliConfig {
                manifest_path: Some(PathBuf::from("crates/sensor/Cargo.toml")),
                proto_dir: Some(PathBuf::from("custom_proto")),
                out_dir: Some(PathBuf::from("custom_src")),
                plugin: Some(PathBuf::from("/usr/local/bin/protoc-gen-roto")),
                grpc: true,
                grpc_plugin: Some(PathBuf::from("/usr/local/bin/protoc-gen-roto-grpc")),
                mode: OutputMode::Mod,
                offline: true,
                locked: true,
                frozen: true,
            })
        );

        let cmd_eq = parse_args([
            "roto-build",
            "--manifest-path=foo/Cargo.toml",
            "--proto-dir",
            "bar/proto",
            "--out-dir=bar/src",
            "--plugin",
            "bin/protoc-gen-roto",
            "--grpc-plugin",
            "bin/protoc-gen-roto-grpc",
            "--mode=lib",
        ])
        .unwrap();

        assert_eq!(
            cmd_eq,
            CliConfig {
                manifest_path: Some(PathBuf::from("foo/Cargo.toml")),
                proto_dir: Some(PathBuf::from("bar/proto")),
                out_dir: Some(PathBuf::from("bar/src")),
                plugin: Some(PathBuf::from("bin/protoc-gen-roto")),
                grpc: false,
                grpc_plugin: Some(PathBuf::from("bin/protoc-gen-roto-grpc")),
                mode: OutputMode::Lib,
                offline: false,
                locked: false,
                frozen: false,
            }
            .into()
        );
    }

    impl From<CliConfig> for CliCommand {
        fn from(cfg: CliConfig) -> Self {
            Self::Run(cfg)
        }
    }

    #[test]
    fn parses_short_and_long_help_flags() {
        let help1 = parse_args(["cargo-roto", "roto", "-h"]).unwrap();
        match help1 {
            CliCommand::Help(text) => {
                assert!(text.contains("Usage: cargo roto [build|generate] [OPTIONS]"));
                assert!(text.contains("--manifest-path"));
                assert!(text.contains("--out-dir"));
                assert!(text.contains("--grpc"));
                assert!(text.contains("--grpc-plugin"));
            }
            other => panic!("expected Help, got {other:?}"),
        }

        let help2 = parse_args(["roto-build", "--help"]).unwrap();
        match help2 {
            CliCommand::Help(text) => {
                assert!(text.contains("Usage: roto-build [build|generate] [OPTIONS]"));
            }
            other => panic!("expected Help, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_and_malformed_flags() {
        assert!(parse_args(["cargo-roto", "--unknown-flag"]).is_err());
        assert!(parse_args(["cargo-roto", "roto", "unknown-subcmd"]).is_err());
        assert!(parse_args(["cargo-roto", "--manifest-path"]).is_err());
        assert!(parse_args(["cargo-roto", "--manifest-path="]).is_err());
        assert!(parse_args(["cargo-roto", "--mode", "invalid"]).is_err());
        assert!(parse_args(["cargo-roto", "--mode=flat"]).is_err());
        assert!(parse_args(["cargo-roto", "--plugin"]).is_err());
        assert!(parse_args(["cargo-roto", "--grpc-plugin"]).is_err());
        assert!(parse_args(["cargo-roto", "--grpc-plugin="]).is_err());
        assert!(parse_args(["cargo-roto", "--out-dir="]).is_err());
        assert!(parse_args(["cargo-roto", "--proto-dir"]).is_err());
    }
}
