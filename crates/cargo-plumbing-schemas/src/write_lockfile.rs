//! Messages used by `cargo plumbing write-lockfile` command

use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::MessageIter;
use crate::lockfile::{NormalizedDependency, NormalizedPatch};

/// Input messages for `cargo-plumbing write-lockfile`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum WriteLockfileIn {
    Lockfile {
        version: Option<u32>,
    },
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

impl WriteLockfileIn {
    /// Creates an iterator to parse a stream of [`WriteLockfileIn`]s.
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
    let schema = schemars::schema_for!(WriteLockfileIn);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../write-lockfile.in.schema.json").raw()
    );
}
