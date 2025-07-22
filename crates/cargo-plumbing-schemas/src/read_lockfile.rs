//! Messages used by `cargo plumbing read-lockfile` command

use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;
use std::marker::PhantomData;

use cargo_util_schemas::core::{PackageIdSpec, SourceKind};
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

use crate::{
    resolve::{EncodableSourceId, Precise},
    MessageIter,
};

pub type Metadata = BTreeMap<String, String>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedResolve {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub package: Vec<NormalizedDependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "NormalizedPatch::is_empty")]
    pub patch: NormalizedPatch,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedPatch {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unused: Vec<NormalizedDependency>,
}

impl NormalizedPatch {
    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedDependency {
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub id: PackageIdSpec,
    pub source: Option<NormalizedSourceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub dependencies: Option<Vec<PackageIdSpec>>,
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<PackageIdSpec>,
}

#[derive(Debug)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedSourceId {
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub url: Url,
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub kind: SourceKind,
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub precise: Option<Precise>,
}

impl fmt::Display for NormalizedSourceId {
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

impl Serialize for NormalizedSourceId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for NormalizedSourceId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let source_id = EncodableSourceId::deserialize(d)?;
        Ok(source_id.normalize())
    }
}

/// Represents the messages outputted by the `cargo-plumbing read-lockfile` command.
///
/// This enum captures all possible JSON objects that the command can emit. The `reason`
/// field acts as a discriminant to distinguish between different message types.
#[derive(Deserialize, Serialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ReadLockfileMessage {
    Lockfile { lockfile: NormalizedResolve },
}

impl ReadLockfileMessage {
    /// Creates an iterator to parse a stream of [`ReadLockfileMessage`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_read_lockfile_schema() {
    let schema = schemars::schema_for!(ReadLockfileMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../read-lockfile.schema.json").raw());
}
