use std::path::PathBuf;

use cargo::core::{EitherManifest, MaybePackage, SourceId, Workspace};
use cargo::util::toml::read_manifest;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_manifest::ReadManifestMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
    /// Read all manifests in the workspace
    #[clap(long, default_value_t = false)]
    workspace: bool,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = gctx.cwd().join(args.manifest_path);

    if args.workspace {
        let workspace = Workspace::new(&manifest_path, gctx)?;

        let msg = match workspace.root_maybe() {
            MaybePackage::Package(pkg) => ReadManifestMessage::Manifest {
                path: gctx.cwd().join(pkg.manifest_path()),
                pkg_id: Some(pkg.package_id().to_spec()),
                manifest: pkg.manifest().normalized_toml().clone(),
            },
            MaybePackage::Virtual(v) => ReadManifestMessage::Manifest {
                path: gctx.cwd().join(workspace.root_manifest()),
                pkg_id: None,
                manifest: v.normalized_toml().clone(),
            },
        };
        gctx.shell().print_json(&msg)?;

        for member in workspace
            .members()
            .filter(|p| p.manifest_path() != workspace.root_manifest())
        {
            let msg = ReadManifestMessage::Manifest {
                path: gctx.cwd().join(member.manifest_path()),
                pkg_id: Some(member.package_id().to_spec()),
                manifest: member.manifest().normalized_toml().clone(),
            };
            gctx.shell().print_json(&msg)?;
        }
    } else {
        let source_id = SourceId::for_manifest_path(&manifest_path)?;

        let (pkg_id, manifest) = match read_manifest(&manifest_path, source_id, gctx)? {
            EitherManifest::Real(r) => {
                (Some(r.package_id().to_spec()), r.normalized_toml().clone())
            }
            EitherManifest::Virtual(v) => (None, v.normalized_toml().clone()),
        };

        let msg = ReadManifestMessage::Manifest {
            path: manifest_path,
            pkg_id,
            manifest,
        };
        gctx.shell().print_json(&msg)?;
    }

    Ok(())
}
