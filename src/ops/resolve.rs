use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{anyhow, Context as _};
use cargo::core::{
    PackageId, PackageIdSpec, Resolve, ResolveVersion, SourceId, SourceKind, Workspace,
};
use cargo::util::Graph;
use cargo::CargoResult;
use cargo_plumbing_schemas::lockfile::{NormalizedDependency, NormalizedPatch, NormalizedResolve};

use crate::cargo::core::resolver::encode::{
    build_path_deps, EncodableDependency, EncodablePackageId, EncodableResolve, Metadata, Patch,
};

/// Converts plumbing messages into an incomplete [`Resolve`]
///
/// The `features` and `summaries` fields of the returned struct is empty.
pub fn into_resolve(
    ws: &Workspace<'_>,
    packages: Vec<NormalizedDependency>,
    patch: NormalizedPatch,
) -> CargoResult<Resolve> {
    let path_deps = build_path_deps(ws)?;

    let mut checksums = HashMap::new();

    let live_pkgs = {
        let mut all_pkgs = HashSet::new();
        let mut live_pkgs = HashMap::new();
        for pkg in packages.iter() {
            if !all_pkgs.insert(pkg.id.clone()) {
                anyhow::bail!("package `{}` is specified twice", pkg.id.name());
            }

            let source_id = get_path_deps_source_id(&path_deps, pkg.id.name(), pkg.id.version());
            let Ok(Some(id)) = spec_to_id(pkg.id.clone(), source_id, pkg.rev.clone()) else {
                continue;
            };

            if let Some(cksum) = &pkg.checksum {
                checksums.insert(id, Some(cksum.clone()));
            }

            live_pkgs.insert(pkg.id.clone(), (id, pkg));
        }
        live_pkgs
    };

    // When decoding a V2 version the edges in `dependencies` aren't
    // guaranteed to have either version or source information. This `map`
    // is used to find package ids even if dependencies have missing
    // information. This map is from name to version to source to actual
    // package ID. (various levels to drill down step by step)
    let mut map = HashMap::new();
    for (id, _) in live_pkgs.values() {
        map.entry(id.name().as_str())
            .or_insert_with(HashMap::new)
            .entry(id.version())
            .or_insert_with(HashMap::new)
            .insert(id.source_id(), *id);
    }

    let lookup_id = |pkg_id: &PackageIdSpec, source_id: Option<&SourceId>| -> Option<PackageId> {
        let by_version = map.get(pkg_id.name())?;

        let by_source = match &pkg_id.version() {
            Some(version) => by_version.get(version)?,
            None => {
                if by_version.len() == 1 {
                    by_version.values().next().unwrap()
                } else {
                    return None;
                }
            }
        };

        match &source_id {
            Some(source) => by_source.get(source).cloned(),
            None => {
                let mut path_packages = by_source.values().filter(|p| p.source_id().is_path());
                if let Some(path) = path_packages.next() {
                    if path_packages.next().is_some() {
                        None
                    } else {
                        Some(*path)
                    }
                } else if by_source.len() == 1 {
                    Some(*by_source.values().next().unwrap())
                } else {
                    None
                }
            }
        }
    };

    let graph = {
        let mut g = Graph::new();

        for (id, _) in live_pkgs.values() {
            g.add(*id);
        }

        for &(ref id, pkg) in live_pkgs.values() {
            let Some(ref deps) = pkg.dependencies else {
                continue;
            };

            for edge in deps.iter() {
                let package_id = spec_to_id(edge.clone(), None, None)?;
                let source_id = package_id.map(|p| p.source_id());
                if let Some(to_depend_on) = lookup_id(edge, source_id.as_ref()) {
                    g.link(*id, to_depend_on);
                }
            }
        }
        g
    };

    let replacements = {
        let mut replacements = HashMap::new();
        for &(ref id, pkg) in live_pkgs.values() {
            if let Some(ref replace) = pkg.replace {
                assert!(pkg.dependencies.is_none());
                let source_id = id.source_id();
                if let Some(replace_id) = lookup_id(replace, Some(&source_id)) {
                    replacements.insert(*id, replace_id);
                }
            }
        }
        replacements
    };

    let unused_patches = {
        let mut unused_patches = Vec::new();
        for pkg in patch.unused {
            let source_id = get_path_deps_source_id(&path_deps, pkg.id.name(), pkg.id.version());
            let Ok(Some(id)) = spec_to_id(pkg.id.clone(), source_id, pkg.rev) else {
                continue;
            };
            unused_patches.push(id);
        }
        unused_patches
    };

    let metadata = BTreeMap::new();
    let features = HashMap::new();
    let summaries = HashMap::new();

    // We use a separate schema from cargo's lockfile versions, where it is comparable to the V4
    // lockfile version.
    let version = ResolveVersion::V4;

    Ok(Resolve::new(
        graph,
        replacements,
        features,
        checksums,
        metadata,
        unused_patches,
        version,
        summaries,
    ))
}

pub fn get_path_deps_source_id<'a>(
    path_deps: &'a HashMap<String, HashMap<semver::Version, SourceId>>,
    package_name: &str,
    package_version: Option<semver::Version>,
) -> Option<&'a SourceId> {
    path_deps.iter().find_map(|(name, version_source)| {
        if name != package_name || version_source.is_empty() {
            return None;
        }

        if version_source.len() == 1 {
            return Some(version_source.values().next().unwrap());
        }

        if let Some(pkg_version) = &package_version {
            if let Some(source_id) = version_source.get(pkg_version) {
                return Some(source_id);
            }
        }

        None
    })
}

pub fn spec_to_id(
    spec: PackageIdSpec,
    source_id: Option<&SourceId>,
    git_rev: Option<String>,
) -> CargoResult<Option<PackageId>> {
    if let Some(kind) = spec.kind() {
        if let Some(url) = spec.url() {
            if let Some(version) = spec.version() {
                let name = spec.name();
                let source_id = match kind {
                    // We're splitting the git reference into a separate field called `rev`. This
                    // means the GitReference from source itself may or may not have what we need.
                    // Therefore, we need a `git_rev` to construct the source ID.
                    SourceKind::Git(git_reference) => {
                        let source_id = SourceId::for_git(url, git_reference.clone())?
                            .with_git_precise(git_rev);
                        Ok(source_id)
                    }
                    SourceKind::Registry | SourceKind::SparseRegistry => {
                        SourceId::for_registry(url)
                    }
                    SourceKind::Path => SourceId::for_path(
                        &url.to_file_path().map_err(|_| anyhow!("invalid path"))?,
                    ),
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

/// Normalizes [`EncodableResolve`] into [`NormalizedResolve`].
///
/// This is used when outputting a message containing a resolve result, i.e. for `read-lockfile`
/// plumbing command using [`ReadLockfileOut`].
///
/// [`ReadLockfileOut`]: cargo_plumbing_schemas::read_lockfile::ReadLockfileOut
pub fn normalize_resolve(resolve: EncodableResolve) -> CargoResult<NormalizedResolve> {
    let mut version: u32 = resolve.version.unwrap_or(1);
    let package = normalize_packages(
        resolve.root,
        resolve.package,
        resolve.metadata,
        Some(&mut version),
    )?;

    Ok(NormalizedResolve {
        version,
        package,
        patch: normalize_patch(resolve.patch)?,
    })
}

/// Normalizes a set of packages and their checksums
///
/// Old lockfile versions have:
/// - packages' checksums separated in a `[metadata]` table
/// - the root package separated from the list of packages in `[root]`
///
/// This function normalizes these packages by moving the root package into the list of packages
/// and move the checksums to be with their respective packages instead of separated in another
/// table.
///
/// Arguments:
/// - `version` -- `Some` infers lockfile version, while `None` doesn't.
pub fn normalize_packages(
    root: Option<EncodableDependency>,
    packages: Option<Vec<EncodableDependency>>,
    metadata: Option<Metadata>,
    mut version: Option<&mut u32>,
) -> CargoResult<Vec<NormalizedDependency>> {
    // We first parse the checksums to be indexable by `PackageIdSpec`. The metadata table
    // itself has keys prefixed with "checksum " then followed by an `EncodablePackageId`.
    let mut metadata_map = {
        let mut metadata_map = HashMap::new();
        if let Some(metadata) = metadata {
            let prefix = "checksum ";
            for (k, v) in metadata {
                let k = k.strip_prefix(prefix).unwrap();
                let id = k
                    .parse::<EncodablePackageId>()
                    .with_context(|| "invalid encoding of checksum in lockfile")?;
                let id = normalize_package_id(id)?;
                metadata_map.insert(id, v);
            }
        }
        metadata_map
    };

    let package = {
        let mut normalized_packages = Vec::new();

        // We first combine the set of packages with the separate `root` package, if it exists.
        let mut all_packages = packages.unwrap_or_default();
        if let Some(pkg) = root {
            all_packages.push(pkg);
            // We only sort here because pushing an additional package may disturb the
            // lexicographical ordering.
            all_packages.sort_unstable();
        }

        for pkg in all_packages {
            let mut pkg = normalize_dependency(pkg)?;
            // We check if the checksum exist already or not. If the checksum already exists,
            // we are not dealing with an old lockfile and can safely continue.
            if pkg.checksum.is_none() {
                // If the checksum does not exist, we take it from the parsed metadata table we
                // created earlier.
                pkg.checksum = metadata_map.remove(&pkg.id);
            } else {
                // Here, it means that the checksum is directly on package, which means that
                // the lockfile version is at least V2.
                if let Some(v) = version.as_mut() {
                    **v = (**v).max(2);
                }
            }
            normalized_packages.push(pkg);
        }

        normalized_packages
    };

    Ok(package)
}

/// Normalizes unused patch entries in the lockfile into a [`NormalizedPatch`].
///
/// The unused patches have the same format as the packages since they're serialized using
/// [`EncodableDependency`].
pub fn normalize_patch(patch: Patch) -> CargoResult<NormalizedPatch> {
    let unused = patch
        .unused
        .into_iter()
        .map(normalize_dependency)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NormalizedPatch { unused })
}

/// Normalizes [`EncodableDependency`] into a [`NormalizedDependency`].
///
/// This function normalizes a [`EncodableDependency`] as-is. It doesn't verify if the dependency
/// is valid other than checking the package ID format.
///
/// To handle old lockfile versions where the package information is scattered throughout the
/// lockfile, use [`normalize_packages`], which uses this function internally.
pub fn normalize_dependency(dep: EncodableDependency) -> CargoResult<NormalizedDependency> {
    let mut id = PackageIdSpec::new(dep.name).with_version(dep.version.parse()?);
    let mut source = None;

    if let Some(s) = dep.source {
        id = id.with_url(s.url.clone()).with_kind(s.kind.clone());
        source = Some(s);
    }

    let dependencies = match dep.dependencies {
        Some(deps) => Some(
            deps.into_iter()
                .map(normalize_package_id)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    let replace = match dep.replace {
        Some(replace) => Some(normalize_package_id(replace)?),
        None => None,
    };

    let rev = match source {
        Some(s) if matches!(s.kind, SourceKind::Git(..)) => s.precise,
        _ => None,
    };

    Ok(NormalizedDependency {
        id,
        rev,
        checksum: dep.checksum,
        dependencies,
        replace,
    })
}

/// Normalizes [`EncodablePackageId`] into a [`PackageIdSpec`].
pub fn normalize_package_id(package_id: EncodablePackageId) -> CargoResult<PackageIdSpec> {
    let mut id = PackageIdSpec::new(package_id.name);

    if let Some(version) = package_id.version {
        id = id.with_version(version.parse()?);
    }

    if let Some(source) = package_id.source {
        id = id.with_url(source.url).with_kind(source.kind);
    }

    Ok(id)
}
