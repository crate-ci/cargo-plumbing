use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::GlobalContext;
use cargo_plumbing::CargoResult;
use serde::Serialize;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
struct ProjectLocation<'a> {
    manifest_path: &'a str,
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

    let location = ProjectLocation {
        manifest_path: root_manifest,
    };

    gctx.shell().print_json(&location)?;

    Ok(())
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_project_location_schema() {
    let schema = schemars::schema_for!(ProjectLocation<'_>);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../../../../project-location.schema.json").raw()
    );
}
