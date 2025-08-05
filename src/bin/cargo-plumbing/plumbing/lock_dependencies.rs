use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::{CliFeatures, HasDevUnits};
use cargo::core::{ResolveVersion, Workspace};
use cargo::ops::resolve_with_previous;
use cargo::sources::SourceConfigMap;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::cargo::core::resolver::encode::{
    encodable_package_id, encodable_resolve_node, encodable_source_id, EncodableDependency,
    EncodeState,
};
use cargo_plumbing_schemas::lockfile::NormalizedPatch;
use cargo_plumbing_schemas::lockfile_contents::LockfileContentsMessage;

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

    let version = match resolve.version() {
        ResolveVersion::V5 => Some(5),
        ResolveVersion::V4 => Some(4),
        ResolveVersion::V3 => Some(3),
        ResolveVersion::V2 | ResolveVersion::V1 => None,
    };
    gctx.shell()
        .print_json(&LockfileContentsMessage::Lockfile { version })?;

    let mut ids: Vec<_> = resolve.iter().collect();
    ids.sort();

    let state = EncodeState::new(&resolve);

    let packages = ids
        .iter()
        .map(|&id| encodable_resolve_node(id, &resolve, &state))
        .collect::<Vec<_>>();

    let mut metadata = resolve.metadata().clone();

    if resolve.version() == ResolveVersion::V1 {
        for &id in ids.iter().filter(|id| !id.source_id().is_path()) {
            let checksum = match resolve.checksums()[&id] {
                Some(ref s) => &s[..],
                None => "<none>",
            };
            let id = encodable_package_id(id, &state, resolve.version());
            metadata.insert(format!("checksum {id}"), checksum.to_owned());
        }
    }

    for package in packages {
        let package = package.normalize()?;
        let msg = LockfileContentsMessage::LockedPackage { package };
        gctx.shell().print_json(&msg)?;
    }

    if !metadata.is_empty() {
        let msg = LockfileContentsMessage::Metadata { metadata };
        gctx.shell().print_json(&msg)?;
    }

    let unused: Vec<_> = resolve
        .unused_patches()
        .iter()
        .map(|id| {
            EncodableDependency {
                name: id.name().to_string(),
                version: id.version().to_string(),
                source: encodable_source_id(id.source_id(), resolve.version()),
                dependencies: None,
                replace: None,
                checksum: if resolve.version() >= ResolveVersion::V2 {
                    resolve.checksums().get(id).and_then(|x| x.clone())
                } else {
                    None
                },
            }
            .normalize()
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !unused.is_empty() {
        let unused = NormalizedPatch { unused };
        let msg = LockfileContentsMessage::UnusedPatches { unused };
        gctx.shell().print_json(&msg)?;
    }

    Ok(())
}
