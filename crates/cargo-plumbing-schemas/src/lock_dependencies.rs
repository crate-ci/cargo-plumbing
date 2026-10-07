//! Messages used by `cargo plumbing lock-dependencies` command

use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::MessageIter;
use crate::lockfile::{NormalizedDependency, NormalizedPatch};

/// Input messages for `cargo-plumbing lock-dependencies`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum LockDependenciesIn {
    /// The locked package from the lockfile
    ///
    /// Expected to be inputted in a lexicographical order based on the package
    /// name, matching the order of the `[[package]]` entries in a `Cargo.lock`
    /// file.
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl LockDependenciesIn {
    /// Creates an iterator to parse a stream of [`LockDependenciesIn`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

/// Output messages for `cargo-plumbing lock-dependencies`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum LockDependenciesOut {
    Lockfile {
        version: u32,
    },
    /// The locked package from the lockfile
    ///
    /// Expected to be outputted in a lexicographical order based on the package
    /// name, matching the order of the `[[package]]` entries in a `Cargo.lock`
    /// file.
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl LockDependenciesOut {
    /// Creates an iterator to parse a stream of [`LockDependenciesOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_lock_dependencies_schema() {
    let schema = schemars::schema_for!(LockDependenciesOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../lock-dependencies.out.schema.json").raw()
    );

    let schema = schemars::schema_for!(LockDependenciesIn);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../lock-dependencies.in.schema.json").raw()
    );
}
