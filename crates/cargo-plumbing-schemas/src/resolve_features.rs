use serde::{Deserialize, Serialize};

/// Represents the messages outputted by the `cargo-plumbing resolve-features` command.
///
/// This enum captures all possible JSON objects that the command can emit. The `reason`
/// field acts as a discriminant to distinguish between different message types.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ResolveFeaturesMessage {}
