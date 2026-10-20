#![forbid(unsafe_code)]

pub mod discovery;
pub mod json;
pub mod metadata;

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

/// Entrypoint invoked by `cargo-roto` and `roto-build` binaries.
pub fn run_from_env() -> Result<(), String> {
    let _ = resolve_and_discover(&MetadataOptions::default())?;
    Ok(())
}
