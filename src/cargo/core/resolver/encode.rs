//! Definition of how to encode a `Resolve` into a TOML `Cargo.lock` file
//!
//! This module is a temporary copy from the cargo codebase.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use cargo::core::{
    Dependency, GitReference, Package, PackageId, Resolve, ResolveVersion, SourceId, SourceKind,
    Workspace,
};
use cargo::util::interning::InternedString;
use cargo::CargoResult;
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

/// The `Cargo.lock` structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableResolve {
    pub version: Option<u32>,
    pub package: Option<Vec<EncodableDependency>>,
    /// `root` is optional to allow backward compatibility.
    pub root: Option<EncodableDependency>,
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Patch::is_empty")]
    pub patch: Patch,
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
pub struct Patch {
    pub unused: Vec<EncodableDependency>,
}

pub type Metadata = BTreeMap<String, String>;

impl Patch {
    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodableDependency {
    pub name: String,
    pub version: String,
    pub source: Option<EncodableSourceId>,
    pub checksum: Option<String>,
    pub dependencies: Option<Vec<EncodablePackageId>>,
    pub replace: Option<EncodablePackageId>,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodableSourceId {
    pub kind: SourceKind,
    pub url: Url,
    pub precise: Option<String>,
}

impl EncodableSourceId {
    pub fn new(string: &str) -> CargoResult<Self> {
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
                    precise,
                })
            }
            "registry" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Registry,
                    precise: None,
                })
            }
            "sparse" => {
                let url = str_to_url(string)?;
                Ok(Self {
                    url,
                    kind: SourceKind::SparseRegistry,
                    precise: None,
                })
            }
            "path" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Path,
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
        Self::new(&string).map_err(de::Error::custom)
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodablePackageId {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<EncodableSourceId>,
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
                    Some(EncodableSourceId::new(s)?)
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
        Some(
            if version >= ResolveVersion::V4 {
                EncodableSourceId::new(&id.as_url().to_string())
            } else {
                EncodableSourceId::new(&id.as_url().to_string())
            }
            .unwrap(),
        )
    }
}
