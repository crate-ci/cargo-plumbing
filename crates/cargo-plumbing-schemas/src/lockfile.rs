//! Lockfile definitions for cargo-plumbing.
//!
//! These are the primary means of communication between lockfile-related commands in Cargo
//! plumbing.
//!
//! As of writing, these lockfile definitions are only used for the `read-lockfile` command. In the
//! future, they will be used for other lockfile-related commands, such as `lock-dependencies`
//! and `write-lockfile`.

use cargo_util_schemas::core::PackageIdSpec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct NormalizedResolve {
    pub version: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub package: Vec<NormalizedDependency>,
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
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<Vec<String>>"))]
    pub dependencies: Option<Vec<PackageIdSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
    pub replace: Option<PackageIdSpec>,
}
