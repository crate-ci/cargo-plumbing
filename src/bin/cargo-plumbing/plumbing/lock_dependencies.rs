use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::{CliFeatures, HasDevUnits};
use cargo::core::Workspace;
use cargo::ops::resolve_with_previous;
use cargo::sources::SourceConfigMap;
use cargo::{CargoResult, GlobalContext};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = args
        .manifest_path
        .unwrap_or(env::current_dir()?.join("Cargo.toml"));
    let path = gctx.cwd().join(manifest_path);
    let ws = Workspace::new(&path, gctx)?;
    let previous_resolve = None;

    let source_config = SourceConfigMap::new(gctx)?;
    let mut registry = PackageRegistry::new_with_source_config(gctx, source_config)?;

    let resolve = resolve_with_previous(
        &mut registry,
        &ws,
        &CliFeatures::new_all(true),
        HasDevUnits::Yes,
        previous_resolve,
        None,
        &[],
        true,
    )?;

    gctx.shell().print_json(&resolve)?;

    Ok(())
}
