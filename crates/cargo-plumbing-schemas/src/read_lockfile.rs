//! Messages used by `cargo plumbing read-lockfile` command

use std::io::Read;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::MessageIter;
use crate::lockfile::{NormalizedDependency, NormalizedPatch};

/// Output messages for `cargo-plumbing read-lockfile`.
#[derive(Deserialize, Serialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum ReadLockfileOut {
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

impl ReadLockfileOut {
    /// Creates an iterator to parse a stream of [`ReadLockfileOut`]s.
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
    let schema = schemars::schema_for!(ReadLockfileOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../read-lockfile.out.schema.json").raw()
    );
}
