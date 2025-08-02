//! Messages used by `cargo plumbing lock-dependencies` command

use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::lockfile::{Metadata, NormalizedDependency, NormalizedPatch};
use crate::MessageIter;

/// Represents the messages outputted by the `cargo-plumbing lock-dependencies` command.
///
/// This enum captures all possible JSON objects that the command can emit. The `reason`
/// field acts as a discriminant to distinguish between different message types.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum LockDependenciesMessage {
    Lockfile {
        version: Option<u32>,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    Metadata {
        #[serde(flatten)]
        metadata: Metadata,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl LockDependenciesMessage {
    /// Creates an iterator to parse a stream of [`LockDependenciesMessage`]s.
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
    let schema = schemars::schema_for!(LockDependenciesMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../lock-dependencies.schema.json").raw()
    );
}
