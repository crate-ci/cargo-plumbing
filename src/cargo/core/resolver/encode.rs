//! Definition of how to encode a `Resolve` into a TOML `Cargo.lock` file
//!
//! This module is a temporary copy from the cargo codebase.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use cargo::core::{Dependency, Package, Workspace};
use cargo::util::internal;
use cargo::util::interning::InternedString;
use cargo::{
    core::{GitReference, PackageId, PackageIdSpec, Resolve, ResolveVersion, SourceId, SourceKind},
    CargoResult,
};
use cargo_plumbing_schemas::lockfile::{
    NormalizedDependency, NormalizedPatch, NormalizedResolve, Precise,
};
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

/// The `Cargo.lock` structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableResolve {
    pub version: Option<u32>,
    package: Option<Vec<EncodableDependency>>,
    /// `root` is optional to allow backward compatibility.
    root: Option<EncodableDependency>,
    metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Patch::is_empty")]
    patch: Patch,
}

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

#[derive(Serialize, Deserialize, Debug, Default)]
struct Patch {
    unused: Vec<EncodableDependency>,
}

impl EncodableResolve {
    pub fn normalize(self) -> CargoResult<NormalizedResolve> {
        let package = normalize_packages(self.root, self.package, self.metadata)?;

        Ok(NormalizedResolve {
            package,
            patch: self.patch.normalize()?,
        })
    }
}

pub fn normalize_packages(
    root: Option<EncodableDependency>,
    packages: Option<Vec<EncodableDependency>>,
    metadata: Option<Metadata>,
) -> CargoResult<Vec<NormalizedDependency>> {
    let mut metadata_map = {
        let mut metadata_map = HashMap::new();
        if let Some(metadata) = metadata {
            let prefix = "checksum ";
            for (k, v) in metadata {
                let k = k.strip_prefix(prefix).unwrap();
                let id = k
                    .parse::<EncodablePackageId>()
                    .with_context(|| internal("invalid encoding of checksum in lockfile"))?
                    .normalize()?;
                metadata_map.insert(id, v);
            }
        }
        metadata_map
    };

    let package = {
        let mut normalized_packages = Vec::new();
        if let Some(pkgs) = packages {
            for pkg in pkgs {
                let mut pkg = pkg.normalize()?;
                if pkg.checksum.is_none() {
                    pkg.checksum = metadata_map.remove(&pkg.id);
                }
                normalized_packages.push(pkg);
            }
        }
        if let Some(pkg) = root {
            let mut pkg = pkg.normalize()?;
            if pkg.checksum.is_none() {
                pkg.checksum = metadata_map.remove(&pkg.id);
            }
            normalized_packages.push(pkg);
        }
        normalized_packages
    };

    Ok(package)
}

pub type Metadata = BTreeMap<String, String>;

impl Patch {
    pub(crate) fn normalize(self) -> CargoResult<NormalizedPatch> {
        let unused = self
            .unused
            .into_iter()
            .map(|u| u.normalize())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NormalizedPatch { unused })
    }

    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableDependency {
    pub name: String,
    pub version: String,
    pub source: Option<EncodableSourceId>,
    pub checksum: Option<String>,
    pub dependencies: Option<Vec<EncodablePackageId>>,
    pub replace: Option<EncodablePackageId>,
}

impl EncodableDependency {
    pub fn normalize(self) -> CargoResult<NormalizedDependency> {
        let mut id = PackageIdSpec::new(self.name).with_version(self.version.parse()?);
        let mut source = None;

        if let Some(s) = self.source {
            id = id.with_url(s.url.clone()).with_kind(s.kind.clone());
            source = Some(s);
        }

        let dependencies = match self.dependencies {
            Some(deps) => Some(
                deps.into_iter()
                    .map(|d| d.normalize())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            None => None,
        };

        let replace = match self.replace {
            Some(replace) => Some(replace.normalize()?),
            None => None,
        };

        let rev = match source {
            Some(s) if matches!(s.kind, SourceKind::Git(..)) => s.precise,
            _ => None,
        };

        Ok(NormalizedDependency {
            id,
            rev,
            checksum: self.checksum,
            dependencies,
            replace,
        })
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodableSourceId {
    pub kind: SourceKind,
    pub url: Url,
    pub precise: Option<Precise>,
    pub encoded: bool,
}

impl EncodableSourceId {
    pub fn new(url: Url, precise: Option<&'static str>, kind: SourceKind) -> Self {
        Self {
            url,
            kind,
            encoded: true,
            precise: precise.map(|s| {
                if s == "locked" {
                    Precise::Locked
                } else {
                    Precise::GitUrlFragment(s.to_owned())
                }
            }),
        }
    }

    pub fn without_url_encoded(url: Url, precise: Option<&'static str>, kind: SourceKind) -> Self {
        Self {
            url,
            kind,
            encoded: false,
            precise: precise.map(|s| {
                if s == "locked" {
                    Precise::Locked
                } else {
                    Precise::GitUrlFragment(s.to_owned())
                }
            }),
        }
    }

    pub fn from_url(string: &str) -> CargoResult<Self> {
        let (kind, url) = string
            .split_once('+')
            .ok_or_else(|| anyhow::format_err!("invalid source `{}`", string))?;

        match kind {
            "git" => {
                let mut url = str_to_url(url)?;
                let reference = GitReference::from_query(url.query_pairs());
                let precise = url.fragment().map(|s| s.to_owned());
                url.set_fragment(None);
                url.set_query(None);
                Ok(Self {
                    url,
                    kind: SourceKind::Git(reference),
                    encoded: false,
                    precise: precise.map(Precise::GitUrlFragment),
                })
            }
            "registry" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Registry,
                    encoded: false,
                    precise: Some(Precise::Locked),
                })
            }
            "sparse" => {
                let url = str_to_url(string)?;
                Ok(Self {
                    url,
                    kind: SourceKind::SparseRegistry,
                    encoded: false,
                    precise: Some(Precise::Locked),
                })
            }
            "path" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Path,
                    encoded: false,
                    precise: None,
                })
            }
            kind => Err(anyhow::format_err!("unsupported source protocol: {}", kind)),
        }
    }

    fn is_path(&self) -> bool {
        self.kind == SourceKind::Path
    }
}

impl fmt::Display for EncodableSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(protocol) = self.kind.protocol() {
            write!(f, "{protocol}+")?;
        }
        write!(f, "{}", self.url)?;
        if let Self {
            kind: SourceKind::Git(ref reference),
            ref precise,
            ..
        } = self
        {
            if let Some(pretty) = reference.pretty_ref(true) {
                write!(f, "?{pretty}")?;
            }
            if let Some(precise) = precise.as_ref() {
                write!(f, "#{precise}")?;
            }
        }
        Ok(())
    }
}

impl Serialize for EncodableSourceId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if self.is_path() {
            None::<String>.serialize(s)
        } else {
            s.collect_str(self)
        }
    }
}

impl<'de> Deserialize<'de> for EncodableSourceId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        Self::from_url(&string).map_err(de::Error::custom)
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodablePackageId {
    name: String,
    version: Option<String>,
    source: Option<EncodableSourceId>,
}

impl EncodablePackageId {
    pub fn normalize(self) -> CargoResult<PackageIdSpec> {
        let mut id = PackageIdSpec::new(self.name);

        if let Some(version) = self.version {
            id = id.with_version(version.parse()?);
        }

        if let Some(source) = self.source {
            id = id.with_url(source.url).with_kind(source.kind);
        }

        Ok(id)
    }
}

impl fmt::Display for EncodablePackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(s) = &self.version {
            write!(f, " {s}")?;
        }
        if let Some(s) = &self.source {
            write!(f, " ({s})")?;
        }
        Ok(())
    }
}

impl FromStr for EncodablePackageId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> CargoResult<EncodablePackageId> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if let Some(s) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    Some(EncodableSourceId::from_url(s)?)
                } else {
                    anyhow::bail!("invalid serialized PackageId")
                }
            }
            None => None,
        };

        Ok(EncodablePackageId {
            name: name.to_owned(),
            version: version.map(|v| v.to_owned()),
            // Default to url encoded.
            source: source_id,
        })
    }
}

impl Serialize for EncodablePackageId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EncodablePackageId {
    fn deserialize<D>(d: D) -> Result<EncodablePackageId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(d).and_then(|string| {
            string
                .parse::<EncodablePackageId>()
                .map_err(de::Error::custom)
        })
    }
}

fn str_to_url(string: &str) -> CargoResult<Url> {
    Url::parse(string).map_err(|s| {
        if string.starts_with("git@") {
            anyhow::format_err!(
                "invalid url `{}`: {}; try using `{}` instead",
                string,
                s,
                format_args!("ssh://{}", string.replacen(':', "/", 1))
            )
        } else {
            anyhow::format_err!("invalid url `{}`: {}", string, s)
        }
    })
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
) -> EncodableDependency {
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

    EncodableDependency {
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
) -> EncodablePackageId {
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
    EncodablePackageId {
        name: id.name().to_string(),
        version,
        source,
    }
}

pub fn encodable_source_id(id: SourceId, version: ResolveVersion) -> Option<EncodableSourceId> {
    if id.is_path() {
        None
    } else {
        Some(if version >= ResolveVersion::V4 {
            EncodableSourceId::new(
                id.url().clone(),
                id.precise_git_fragment(),
                id.kind().clone(),
            )
        } else {
            EncodableSourceId::without_url_encoded(
                id.url().clone(),
                id.precise_git_fragment(),
                id.kind().clone(),
            )
        })
    }
}
