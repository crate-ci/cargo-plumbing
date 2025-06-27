use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::GlobalContext;
use cargo_plumbing::CargoResult;
use cargo_plumbing_schemas::locate_manifest::ManifestLocation;
use cargo_plumbing_schemas::Message;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let path = args.manifest_path.unwrap_or(env::current_dir()?);
    let root_manifest = find_root_manifest_for_wd(&path)?;

    let root_manifest = root_manifest.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "your package path contains characters \
             not representable in Unicode"
        )
    })?;

    let location = ManifestLocation {
        manifest_path: String::from(root_manifest),
    };

    writeln!(gctx.shell().out(), "{}", location.to_json_string())?;

    Ok(())
}
