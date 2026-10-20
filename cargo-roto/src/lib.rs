#![forbid(unsafe_code)]

pub mod cli;
pub mod discovery;
pub mod driver;
pub mod json;
pub mod metadata;

use crate::cli::{parse_args, parse_env_args, CliCommand};
use crate::discovery::{discover_protos, DiscoveredProtos};
use crate::metadata::{resolve_workspace, MetadataOptions, ResolvedWorkspace};

/// Combined workspace metadata resolution and `.proto` discovery output for a target crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCrateProtos {
    pub workspace: ResolvedWorkspace,
    pub protos: DiscoveredProtos,
}

/// Resolves Cargo metadata and discovers target `.proto` files for `options`.
pub fn resolve_and_discover(options: &MetadataOptions) -> Result<ResolvedCrateProtos, String> {
    let workspace = resolve_workspace(options)?;
    let protos = discover_protos(&workspace.target_crate_root, &workspace.target_provides)?;
    Ok(ResolvedCrateProtos { workspace, protos })
}

/// Runs `cargo-roto` / `roto-build` using explicit CLI arguments (including `argv[0]`).
pub fn run_with_args<I, S>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    match parse_args(args)? {
        CliCommand::Help(usage) => {
            print!("{usage}");
            Ok(())
        }
        CliCommand::Run(config) => driver::run(&config),
    }
}

/// Entrypoint invoked by `cargo-roto` and `roto-build` binaries.
pub fn run_from_env() -> Result<(), String> {
    match parse_env_args()? {
        CliCommand::Help(usage) => {
            print!("{usage}");
            Ok(())
        }
        CliCommand::Run(config) => driver::run(&config),
    }
}
