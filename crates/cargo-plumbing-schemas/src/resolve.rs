//! `Cargo.lock` / Lock file schema definition

use std::{collections::BTreeMap, fmt, str::FromStr};

use cargo_util_schemas::core::{GitReference, PackageIdSpec, SourceKind};
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

use crate::read_lockfile::{
    NormalizedDependency, NormalizedPatch, NormalizedResolve, NormalizedSourceId,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct UrlParseError(#[from] UrlParseErrorKind);

#[derive(Debug, thiserror::Error)]
enum UrlParseErrorKind {
    #[error("invalid url `{0}`: {1}; try using `{2}` instead")]
    UrlSuggest(String, url::ParseError, String),

    #[error("invalid url `{0}`: {1}")]
    Url(String, url::ParseError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum EncodeError {
    Encode(#[from] EncodeErrorKind),
    UrlParse(#[from] UrlParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeErrorKind {
    #[error("invalid serialized PackageId")]
    InvalidSerializedPackageId,

    #[error("invalid source `{0}`")]
    InvalidSource(String),

    #[error("unsupported source protocol: {0}")]
    UnsupportedSourceProtocol(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct EncodableResolve {
    pub version: Option<u32>,
    pub package: Option<Vec<EncodableDependency>>,
    pub root: Option<EncodableDependency>,
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "EncodablePatch::is_empty")]
    pub patch: EncodablePatch,
}

impl EncodableResolve {
    pub fn normalize(self) -> NormalizedResolve {
        let mut package = if let Some(package) = self.package {
            package.into_iter().map(|p| p.normalize()).collect()
        } else {
            Vec::new()
        };

        if let Some(root) = self.root {
            package.push(root.normalize());
        }

        NormalizedResolve {
            package,
            metadata: self.metadata,
            patch: self.patch.normalize(),
        }
    }
}

pub type Metadata = BTreeMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct EncodablePatch {
    pub unused: Vec<EncodableDependency>,
}

impl EncodablePatch {
    pub fn normalize(self) -> NormalizedPatch {
        let unused = self.unused.into_iter().map(|u| u.normalize()).collect();
        NormalizedPatch { unused }
    }

    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct EncodableDependency {
    pub name: String,
    pub version: String,
    pub source: Option<EncodableSourceId>,
    pub checksum: Option<String>,
    pub dependencies: Option<Vec<EncodablePackageId>>,
    pub replace: Option<EncodablePackageId>,
}

impl EncodableDependency {
    pub fn normalize(self) -> NormalizedDependency {
        let mut id = PackageIdSpec::new(self.name).with_version(self.version.parse().unwrap());

        if let Some(source) = self.source {
            id = id.with_kind(source.kind).with_url(source.url);
        }

        let dependencies = self
            .dependencies
            .map(|dependencies| dependencies.into_iter().map(|d| d.normalize()).collect());

        let replace = self.replace.map(|r| r.normalize());

        NormalizedDependency {
            id,
            checksum: self.checksum,
            dependencies,
            replace,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct EncodablePackageId {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<EncodableSourceId>,
}

impl EncodablePackageId {
    pub fn normalize(self) -> PackageIdSpec {
        let mut id = PackageIdSpec::new(self.name);

        if let Some(version) = self.version {
            id = id.with_version(version.parse().unwrap());
        }

        if let Some(source) = self.source {
            id = id.with_url(source.url).with_kind(source.kind);
        }

        id
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
    type Err = EncodeError;

    fn from_str(s: &str) -> Result<EncodablePackageId, Self::Err> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if let Some(s) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    Some(EncodableSourceId::from_url(s)?)
                } else {
                    return Err(EncodeErrorKind::InvalidSerializedPackageId.into());
                }
            }
            None => None,
        };

        Ok(EncodablePackageId {
            name: name.to_owned(),
            version: version.map(|v| v.to_owned()),
            // Default to url encoded.
            source: source_id.map(|s| EncodableSourceId::new(s.url, s.kind)),
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

#[derive(Debug)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct EncodableSourceId {
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub url: Url,
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub kind: SourceKind,
    pub encoded: bool,
}

impl EncodableSourceId {
    pub fn new(url: Url, kind: SourceKind) -> Self {
        Self {
            url,
            kind,
            encoded: true,
        }
    }

    pub fn from_url(string: &str) -> Result<Self, EncodeError> {
        let (kind, url) = string
            .split_once('+')
            .ok_or_else(|| EncodeErrorKind::InvalidSource(string.to_owned()))?;

        match kind {
            "git" => {
                let mut url = str_to_url(url)?;
                let reference = GitReference::from_query(url.query_pairs());
                url.set_fragment(None);
                url.set_query(None);
                Ok(Self {
                    url,
                    kind: SourceKind::Git(reference),
                    encoded: false,
                })
            }
            "registry" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Registry,
                    encoded: false,
                })
            }
            "sparse" => {
                let url = str_to_url(string)?;
                Ok(Self {
                    url,
                    kind: SourceKind::SparseRegistry,
                    encoded: false,
                })
            }
            "path" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Path,
                    encoded: false,
                })
            }
            kind => Err(EncodeErrorKind::UnsupportedSourceProtocol(kind.to_string()).into()),
        }
    }

    fn is_path(&self) -> bool {
        self.kind == SourceKind::Path
    }

    pub fn normalize(self) -> NormalizedSourceId {
        NormalizedSourceId {
            url: self.url,
            kind: self.kind,
        }
    }
}

impl fmt::Display for EncodableSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(protocol) = self.kind.protocol() {
            write!(f, "{protocol}+")?;
        }
        write!(f, "{}", self.url)?;
        if let SourceKind::Git(ref reference) = self.kind {
            if let Some(pretty) = reference.pretty_ref(self.encoded) {
                write!(f, "?{pretty}")?;
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

fn str_to_url(string: &str) -> Result<Url, UrlParseError> {
    Url::parse(string).map_err(|s| {
        if string.starts_with("git@") {
            let suggestion = format!("ssh://{}", string.replacen(':', "/", 1));
            UrlParseErrorKind::UrlSuggest(string.to_string(), s, suggestion).into()
        } else {
            UrlParseErrorKind::Url(string.to_string(), s).into()
        }
    })
}
