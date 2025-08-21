//! Messages used by `cargo plumbing read-manifest` command

use std::{io::Read, marker::PhantomData, path::PathBuf};

use cargo_util_schemas::core::PackageIdSpec;
pub use cargo_util_schemas::manifest::TomlManifest;
use serde::{Deserialize, Serialize};

use crate::MessageIter;

/// Output messages for `cargo-plumbing read-manifest`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ReadManifestOut {
    /// A message containing the contents of a parsed `Cargo.toml` manifest.
    Manifest {
        workspace: bool,
        /// The path to the manifest file that was read.
        path: PathBuf,
        /// The package ID specification.
        ///
        /// This command also outputs virtual manifests and virtual manifests don't have
        /// [`PackageIdSpec`], hence the use of [`Option`].
        #[cfg_attr(
            feature = "unstable-schema",
            schemars(with = "Option<String>", description = "The package ID specification")
        )]
        pkg_id: Option<PackageIdSpec>,
        /// The fully parsed and deserialized manifest content.
        manifest: TomlManifest,
    },
}

impl ReadManifestOut {
    /// Creates an iterator to parse a stream of [`ReadManifestOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_read_manifest_schema() {
    let schema = schemars::schema_for!(ReadManifestOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../read-manifest.out.schema.json").raw()
    );
}
