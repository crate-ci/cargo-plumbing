//! Lockfile definitions for cargo-plumbing.
//!
//! These are the primary means of communication between lockfile-related commands in Cargo
//! plumbing.
//!
//! As of writing, these lockfile definitions are only used for the `read-lockfile` command. In the
//! future, they will be used for other lockfile-related commands, such as `lock-dependencies`
//! and `write-lockfile`.

use std::{collections::BTreeMap, fmt};

use cargo_util_schemas::core::PackageIdSpec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub type Metadata = BTreeMap<String, String>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
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
#[serde(transparent)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedPatch {
    pub unused: Vec<NormalizedDependency>,
}

impl NormalizedPatch {
    pub fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedDependency {
    #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
    pub id: PackageIdSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub rev: Option<Precise>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub dependencies: Option<Vec<PackageIdSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub replace: Option<PackageIdSpec>,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash, Ord, PartialOrd)]
pub enum Precise {
    Locked,
    GitUrlFragment(String),
}

impl fmt::Display for Precise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precise::Locked => "locked".fmt(f),
            Precise::GitUrlFragment(s) => s.fmt(f),
        }
    }
}

impl Serialize for Precise {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Precise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "locked" {
            Ok(Precise::Locked)
        } else {
            Ok(Precise::GitUrlFragment(s))
        }
    }
}
