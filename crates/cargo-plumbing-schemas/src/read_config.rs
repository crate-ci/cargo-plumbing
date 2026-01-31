//! Messages used by `cargo plumbing read-config` command

use std::io::{Read, BufRead};
use std::marker::PhantomData;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::MessageIter;

/// Output messages for `cargo-plumbing read-config`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ReadConfigOut {
    /// A message containing the build and target directory configuration.
    Config {
        /// The path to the workspace or package manifest.
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        manifest_path: Utf8PathBuf,
        /// The target directory where build artifacts are placed.
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        target_directory: Utf8PathBuf,
        /// The build directory (typically same as target directory for most configurations).
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        build_directory: Utf8PathBuf,
    },
}

impl ReadConfigOut {
    /// Creates an iterator to parse a stream of [`ReadConfigOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}
