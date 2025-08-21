use std::path::PathBuf;

use anyhow::Context as _;
use cargo::core::{
    find_workspace_root, EitherManifest, MaybePackage, SourceId, Workspace, WorkspaceConfig,
};
use cargo::util::toml::read_manifest;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_manifest::ReadManifestOut;
use cargo_util::paths;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
    /// Read all manifests in the workspace
    #[clap(long, default_value_t = false)]
    workspace: bool,
}

pub(crate) fn exec(gctx: &mut GlobalContext, args: Args) -> CargoResult<()> {
    let requested_manifest_path = gctx.cwd().join(args.manifest_path);

    if args.workspace {
        let workspace = Workspace::new(&requested_manifest_path, gctx)?;

        // Here, we print the root Package or the Virtual Manifest of the workspace.
        let msg = match workspace.root_maybe() {
            MaybePackage::Package(pkg) => ReadManifestOut::Manifest {
                workspace: true,
                path: gctx.cwd().join(pkg.manifest_path()),
                pkg_id: Some(pkg.package_id().to_spec()),
                manifest: pkg.manifest().normalized_toml().clone(),
            },
            MaybePackage::Virtual(v) => ReadManifestOut::Manifest {
                workspace: true,
                path: gctx.cwd().join(workspace.root_manifest()),
                pkg_id: None,
                manifest: v.normalized_toml().clone(),
            },
        };
        gctx.shell().print_json(&msg)?;

        // We don't want to print the same manifest twice. This is why we're filtering the
        // workspace members to make sure only those that are not root manifest are printed in this
        // stage.
        for member in workspace
            .members()
            .filter(|p| p.manifest_path() != workspace.root_manifest())
        {
            let msg = ReadManifestOut::Manifest {
                workspace: false,
                path: gctx.cwd().join(member.manifest_path()),
                pkg_id: Some(member.package_id().to_spec()),
                manifest: member.manifest().normalized_toml().clone(),
            };
            gctx.shell().print_json(&msg)?;
        }
    } else {
        // As this is the branch without `--workspace`, we want to only read one manifest.
        let source_id = SourceId::for_manifest_path(&requested_manifest_path)?;
        let (pkg_id, ws_config, manifest) =
            match read_manifest(&requested_manifest_path, source_id, gctx)? {
                EitherManifest::Real(r) => (
                    Some(r.package_id().to_spec()),
                    r.workspace_config().clone(),
                    r.normalized_toml().clone(),
                ),
                EitherManifest::Virtual(v) => (
                    None,
                    v.workspace_config().clone(),
                    v.normalized_toml().clone(),
                ),
            };

        // If the current manifest is a workspace member, find its workspace manifest path.
        let ws_manifest_path = match ws_config {
            WorkspaceConfig::Root(..) => {
                // The current manifest is already the workspace root, so there is no other
                // workspace root to find.
                None
            }
            WorkspaceConfig::Member {
                root: Some(path_to_root),
            } => {
                // The member's `[package.workspace]` key points to the root directory
                Some(
                    requested_manifest_path
                        .parent()
                        .context("manifest-path can't be root")?
                        .join(path_to_root)
                        .join("Cargo.toml"),
                )
            }
            WorkspaceConfig::Member { root: None } => {
                // Find the root directory by searching upwards the filesystem.
                find_workspace_root(&requested_manifest_path, gctx)?
            }
        };

        let msg = ReadManifestOut::Manifest {
            workspace: ws_manifest_path.is_none(),
            path: requested_manifest_path.clone(),
            pkg_id,
            manifest,
        };
        gctx.shell().print_json(&msg)?;

        if let Some(ws_manifest_path) = ws_manifest_path {
            let ws_manifest_path = paths::normalize_path(&gctx.cwd().join(ws_manifest_path));
            let source_id = SourceId::for_manifest_path(&ws_manifest_path)?;

            let (pkg_id, manifest) = match read_manifest(&ws_manifest_path, source_id, gctx)? {
                EitherManifest::Real(r) => {
                    (Some(r.package_id().to_spec()), r.normalized_toml().clone())
                }
                EitherManifest::Virtual(v) => (None, v.normalized_toml().clone()),
            };

            let msg = ReadManifestOut::Manifest {
                workspace: true,
                path: ws_manifest_path.clone(),
                pkg_id,
                manifest,
            };
            gctx.shell().print_json(&msg)?;
        }
    }

    Ok(())
}
