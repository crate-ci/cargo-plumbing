use std::env;
use std::fmt::Debug;

use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::GlobalContext;
use cargo_plumbing::CargoResult;
use cargo_plumbing_schemas::locate_manifest::LocateManifestMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let working_dir = env::current_dir()?;
    let root_manifest = find_root_manifest_for_wd(&working_dir)?;

    let root_manifest = root_manifest.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "your package path contains characters \
             not representable in Unicode"
        )
    })?;

    let location = LocateManifestMessage::ManifestLocation {
        manifest_path: String::from(root_manifest),
    };

    gctx.shell().print_json(&location)?;

    Ok(())
}
