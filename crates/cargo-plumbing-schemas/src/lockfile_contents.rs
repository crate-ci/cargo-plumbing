//! Messages for outputting lockfile contents

use std::io::Read;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::lockfile::{Metadata, NormalizedDependency, NormalizedPatch};
use crate::MessageIter;

/// Messages used to output the contents of a lockfile.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum LockfileContentsMessage {
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

impl LockfileContentsMessage {
    /// Creates an iterator to parse a stream of [`LockfileContentsMessage`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_lockfile_contents_schema() {
    let schema = schemars::schema_for!(LockfileContentsMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../lockfile-contents.schema.json").raw()
    );
}
