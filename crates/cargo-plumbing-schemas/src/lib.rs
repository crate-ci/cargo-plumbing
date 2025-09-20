//! # `cargo-plumbing-schemas`
//!
//! This library provides message and schema definitions for the inputs and outputs of
//! `cargo-plumbing` commands. We provide JSON schemas for each message type. Each message
//! is separated into a module corresponding to its plumbing subcommand.
//!
//! This library is intended to be used by tools that invoke `cargo-plumbing` commands and need
//! to parse the jsonlines output. The [`MessageIter`] provides a convenient way to handle the
//! jsonlines stream.

use std::io::{self, BufRead};
use std::marker::PhantomData;

use serde::de::DeserializeOwned;

pub mod locate_manifest;
pub mod lock_dependencies;
pub mod lockfile;
pub mod plan_build;
pub mod read_lockfile;
pub mod read_manifest;
pub mod resolve_features;
pub mod write_lockfile;

/// Iterator over deserialized jsonline messages
///
/// It is not intended to be constructed directly by users. Instead, users should use solutions
/// provided by the messages enums, such as [`LocateManifestMessage::parse_stream`] and other
/// messages.
///
/// # Type Parameters
///
/// - `R`: The buffered reader it reads from.
/// - `M`: The message type it deserializes each jsonline into.
///
/// [`LocateManifestMessage::parse_stream`]: [`locate_manifest::LocateManifestMessage::parse_stream`]
pub struct MessageIter<R, M> {
    input: R,
    _m: PhantomData<M>,
}

impl<R: BufRead, M: DeserializeOwned> Iterator for MessageIter<R, M> {
    type Item = io::Result<M>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        self.input
            .read_line(&mut line)
            .map(|n| {
                if n == 0 {
                    None
                } else {
                    if line.ends_with('\n') {
                        line.truncate(line.len() - 1);
                    }
                    let mut deserializer = serde_json::Deserializer::from_str(&line);
                    M::deserialize(&mut deserializer).ok()
                }
            })
            .transpose()
    }
}
