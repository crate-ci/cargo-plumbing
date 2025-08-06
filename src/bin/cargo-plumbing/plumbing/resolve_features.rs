use std::collections::HashMap;
use std::env;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

use cargo::core::{PackageId, PackageIdSpec, SourceId, SourceKind, Workspace};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::cargo::core::resolver::encode::build_path_deps;
use cargo_plumbing_schemas::lockfile::NormalizedDependency;
use cargo_plumbing_schemas::read_lockfile::ReadLockfileMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

pub(crate) fn exec(gctx: &mut GlobalContext, args: Args) -> CargoResult<()> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        anyhow::bail!("input must be piped from a file or another command");
    }

    let messages = ReadLockfileMessage::parse_stream(BufReader::new(stdin));

    let manifest_path = args
        .manifest_path
        .unwrap_or(env::current_dir()?.join("Cargo.toml"));
    let path = gctx.cwd().join(manifest_path);
    let ws = Workspace::new(&path, gctx)?;

    let path_deps = build_path_deps(&ws)?;
    let mut lock_version = None;
    let mut packages = Vec::new();

    for message in messages {
        match message? {
            ReadLockfileMessage::Lockfile { version } => lock_version = Some(version),
            ReadLockfileMessage::LockedPackage { package } => {
                let source_id = get_path_deps_source_id(&path_deps, &package);
                if let Ok(Some(pkg)) = spec_to_id(package.id, source_id) {
                    packages.push(pkg);
                };
            }
            _ => {}
        }
    }

    let _ = lock_version;

    Ok(())
}

fn get_path_deps_source_id<'a>(
    path_deps: &'a HashMap<String, HashMap<semver::Version, SourceId>>,
    package: &NormalizedDependency,
) -> Option<&'a SourceId> {
    path_deps.iter().find_map(|(name, version_source)| {
        if name != package.id.name() || version_source.len() == 0 {
            return None;
        }

        if version_source.len() == 1 {
            return Some(version_source.values().next().unwrap());
        }

        if let Some(pkg_version) = package.id.version() {
            if let Some(source_id) = version_source.get(&pkg_version) {
                return Some(source_id);
            }
        }

        None
    })
}

fn spec_to_id(spec: PackageIdSpec, source_id: Option<&SourceId>) -> CargoResult<Option<PackageId>> {
    if let Some(kind) = spec.kind() {
        if let Some(url) = spec.url() {
            if let Some(version) = spec.version() {
                let name = spec.name();
                let source_id = match kind {
                    SourceKind::Git(git_ref) => SourceId::for_git(url, git_ref.clone()),
                    SourceKind::Registry | SourceKind::SparseRegistry => {
                        SourceId::for_registry(url)
                    }
                    _ => anyhow::bail!("unsupported source"),
                }?;

                return Ok(Some(PackageId::new(name.into(), version, source_id)));
            }
        }
    }

    if let Some(source_id) = source_id {
        if let Some(version) = spec.version() {
            let name = spec.name();
            return Ok(Some(PackageId::new(name.into(), version, *source_id)));
        }
    }

    Ok(None)
}
