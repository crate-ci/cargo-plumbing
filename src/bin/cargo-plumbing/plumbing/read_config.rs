use std::path::PathBuf;

use camino::Utf8PathBuf;
use cargo::core::Workspace;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_config::ReadConfigOut;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
}

pub(crate) fn exec(gctx: &mut GlobalContext, args: Args) -> CargoResult<()> {
    let requested_manifest_path = gctx.cwd().join(&args.manifest_path);
    let workspace = Workspace::new(&requested_manifest_path, gctx)?;

    // Get the target directory from the global context
    let target_dir = gctx.target_dir();
    let target_directory =
        Utf8PathBuf::try_from(target_dir.as_path_unlocked().to_path_buf())?;

    // For now, build_directory is the same as target_directory
    // In the future, this could be different based on configuration
    let build_directory = target_directory.clone();

    let manifest_path = Utf8PathBuf::try_from(
        gctx.cwd().join(workspace.root_manifest())
    )?;

    let msg = ReadConfigOut::Config {
        manifest_path,
        target_directory,
        build_directory,
    };

    gctx.shell().print_json(&msg)?;
    Ok(())
}
