use std::{io::Read, marker::PhantomData, path::PathBuf};

use cargo_util_schemas::core::PackageIdSpec;
pub use cargo_util_schemas::manifest::TomlManifest;
use serde::{Deserialize, Serialize};

use crate::MessageIter;

#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ReadManifestMessage {
    Manifest {
        path: PathBuf,
        #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
        pkg_id: Option<PackageIdSpec>,
        manifest: TomlManifest,
    },
}

impl ReadManifestMessage {
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            _m: PhantomData::<Self>,
            input,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_read_manifest_schema() {
    let schema = schemars::schema_for!(ReadManifestMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../read-manifest.schema.json").raw());
}
