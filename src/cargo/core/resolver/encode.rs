//! Definition of how to encode a `Resolve` into a TOML `Cargo.lock` file
//!
//! This module is a temporary copy from the cargo codebase.

use std::collections::{HashMap, HashSet};

use cargo::core::{
    Dependency, GitReference, Package, PackageId, Resolve, ResolveVersion, SourceId, Workspace,
};
use cargo::util::interning::InternedString;
use cargo::CargoResult;
use cargo_util_schemas::lockfile::{
    TomlLockfileDependency, TomlLockfilePackageId, TomlLockfileSourceId,
};

pub fn build_path_deps(
    ws: &Workspace<'_>,
) -> CargoResult<HashMap<String, HashMap<semver::Version, SourceId>>> {
    // If a crate is **not** a path source, then we're probably in a situation
    // such as `cargo install` with a lock file from a remote dependency. In
    // that case we don't need to fixup any path dependencies (as they're not
    // actually path dependencies any more), so we ignore them.
    let members = ws
        .members()
        .filter(|p| p.package_id().source_id().is_path())
        .collect::<Vec<_>>();

    let mut ret: HashMap<String, HashMap<semver::Version, SourceId>> = HashMap::new();
    let mut visited = HashSet::new();
    for member in members.iter() {
        ret.entry(member.package_id().name().to_string())
            .or_default()
            .insert(
                member.package_id().version().clone(),
                member.package_id().source_id(),
            );
        visited.insert(member.package_id().source_id());
    }
    for member in members.iter() {
        build_pkg(member, ws, &mut ret, &mut visited);
    }
    for deps in ws.root_patch()?.values() {
        for dep in deps {
            build_dep(dep, ws, &mut ret, &mut visited);
        }
    }
    for (_, dep) in ws.root_replace() {
        build_dep(dep, ws, &mut ret, &mut visited);
    }

    return Ok(ret);

    fn build_pkg(
        pkg: &Package,
        ws: &Workspace<'_>,
        ret: &mut HashMap<String, HashMap<semver::Version, SourceId>>,
        visited: &mut HashSet<SourceId>,
    ) {
        for dep in pkg.dependencies() {
            build_dep(dep, ws, ret, visited);
        }
    }

    fn build_dep(
        dep: &Dependency,
        ws: &Workspace<'_>,
        ret: &mut HashMap<String, HashMap<semver::Version, SourceId>>,
        visited: &mut HashSet<SourceId>,
    ) {
        let id = dep.source_id();
        if visited.contains(&id) || !id.is_path() {
            return;
        }
        let path = match id.url().to_file_path() {
            Ok(p) => p.join("Cargo.toml"),
            Err(_) => return,
        };
        let Ok(pkg) = ws.load(&path) else { return };
        ret.entry(pkg.package_id().name().to_string())
            .or_default()
            .insert(
                pkg.package_id().version().clone(),
                pkg.package_id().source_id(),
            );
        visited.insert(pkg.package_id().source_id());
        build_pkg(&pkg, ws, ret, visited);
    }
}

pub struct EncodeState<'a> {
    counts: Option<HashMap<InternedString, HashMap<&'a semver::Version, usize>>>,
}

impl<'a> EncodeState<'a> {
    pub fn new(resolve: &'a Resolve) -> EncodeState<'a> {
        let counts = if resolve.version() >= ResolveVersion::V2 {
            let mut map = HashMap::new();
            for id in resolve.iter() {
                let slot = map
                    .entry(id.name())
                    .or_insert_with(HashMap::new)
                    .entry(id.version())
                    .or_insert(0);
                *slot += 1;
            }
            Some(map)
        } else {
            None
        };
        EncodeState { counts }
    }
}

pub fn encodable_resolve_node(
    id: PackageId,
    resolve: &Resolve,
    state: &EncodeState<'_>,
) -> TomlLockfileDependency {
    let (replace, deps) = match resolve.replacement(id) {
        Some(id) => (
            Some(encodable_package_id(id, state, resolve.version())),
            None,
        ),
        None => {
            let mut deps = resolve
                .deps_not_replaced(id)
                .map(|(id, _)| encodable_package_id(id, state, resolve.version()))
                .collect::<Vec<_>>();
            deps.sort();
            (None, if deps.is_empty() { None } else { Some(deps) })
        }
    };

    TomlLockfileDependency {
        name: id.name().to_string(),
        version: id.version().to_string(),
        source: encodable_source_id(id.source_id(), resolve.version()),
        dependencies: deps,
        replace,
        checksum: if resolve.version() >= ResolveVersion::V2 {
            resolve.checksums().get(&id).and_then(|s| s.clone())
        } else {
            None
        },
    }
}

pub fn encodable_package_id(
    id: PackageId,
    state: &EncodeState<'_>,
    resolve_version: ResolveVersion,
) -> TomlLockfilePackageId {
    let mut version = Some(id.version().to_string());
    let mut id_to_encode = id.source_id();
    if resolve_version <= ResolveVersion::V2 {
        if let Some(GitReference::Branch(b)) = id_to_encode.git_reference() {
            if b == "master" {
                id_to_encode =
                    SourceId::for_git(id_to_encode.url(), GitReference::DefaultBranch).unwrap();
            }
        }
    }
    let mut source = encodable_source_id(id_to_encode.without_precise(), resolve_version);
    if let Some(counts) = &state.counts {
        let version_counts = &counts[&id.name()];
        if version_counts[&id.version()] == 1 {
            source = None;
            if version_counts.len() == 1 {
                version = None;
            }
        }
    }
    TomlLockfilePackageId {
        name: id.name().to_string(),
        version,
        source,
    }
}

pub fn encodable_source_id(id: SourceId, version: ResolveVersion) -> Option<TomlLockfileSourceId> {
    if id.is_path() {
        None
    } else {
        Some(
            if version >= ResolveVersion::V4 {
                TomlLockfileSourceId::new(id.as_url().to_string())
            } else {
                TomlLockfileSourceId::new(id.as_url().to_string())
            }
            .unwrap(),
        )
    }
}
