//! Messages used by `cargo plumbing plan-build` command

use std::path::PathBuf;
use std::{io::Read, marker::PhantomData};

use camino::Utf8PathBuf;
use cargo_util_schemas::core::PackageIdSpec;
use cargo_util_schemas::manifest::{TomlDebugInfo, TomlManifest};
use serde::{Deserialize, Serialize};

use crate::lockfile::{NormalizedDependency, NormalizedPatch};
use crate::MessageIter;

/// Input messages for `cargo-plumbing plan-build`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum PlanBuildIn {
    Manifest {
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        workspace: bool,
        /// The path to the manifest file that was read.
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        path: Utf8PathBuf,
        /// The package ID specification.
        ///
        /// This command also takes in virtual manifests and virtual manifests don't have
        /// [`PackageIdSpec`], hence the use of [`Option`].
        #[cfg_attr(
            feature = "unstable-schema",
            schemars(with = "Option<String>", description = "The package ID specification")
        )]
        pkg_id: Option<PackageIdSpec>,
        /// The fully parsed and deserialized manifest content.
        manifest: TomlManifest,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
    Activated {
        id: PackageIdSpec,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        features_for: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        features: Vec<String>,
    },
    Target {
        name: String,
        kind: String,
    },
}

impl PlanBuildIn {
    /// Creates an iterator to parse a stream of [`PlanBuildIn`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

/// Output messages for `cargo-plumbing plan-build`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum PlanBuildOut {
    Unit {
        id: usize,
        target: UnitTarget,
        platform: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deps: Vec<UnitDependency>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        root: bool,
        profile: UnitProfile,
        features: Vec<String>,
    },
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct UnitTarget {
    pub name: String,
    pub crate_types: Vec<String>,
    pub edition: String,
    pub kind: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_path: Option<PathBuf>,
    pub test: bool,
    pub doctest: bool,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct UnitDependency {
    pub index: usize,
    pub extern_crate_name: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub public: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub noprelude: bool,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct UnitProfile {
    pub name: String,
    pub opt_level: String,
    pub lto: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen_units: Option<u32>,
    pub debuginfo: TomlDebugInfo,
    pub debug_assertions: bool,
    pub overflow_checks: bool,
    pub rpath: bool,
    pub incremental: bool,
    pub panic: String,
}

impl PlanBuildOut {
    /// Creates an iterator to parse a stream of [`PlanBuildOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_schema() {
    let schema = schemars::schema_for!(PlanBuildIn);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../plan-build.in.schema.json").raw());

    let schema = schemars::schema_for!(PlanBuildOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../plan-build.out.schema.json").raw());
}
