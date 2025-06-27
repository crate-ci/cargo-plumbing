use std::path::PathBuf;

use cargo::{CargoResult, GlobalContext};
use cargo::core::{EitherManifest, SourceId};
use cargo::util::toml::read_manifest;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    manifest_path: PathBuf,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = args.manifest_path.canonicalize()?;
    let source_id = SourceId::for_manifest_path(&manifest_path)?;

    match read_manifest(&manifest_path, source_id, gctx)? {
        EitherManifest::Real(manifest) => {
            gctx.shell().print_json(manifest.normalized_toml())?;
        }
        EitherManifest::Virtual(manifest) => {
            gctx.shell().print_json(manifest.normalized_toml())?;
        }
    };

    Ok(())
}
