use std::io::Read;
use std::marker::PhantomData;

use camino::Utf8PathBuf;
use cargo_util_schemas::core::PackageIdSpec;
use cargo_util_schemas::manifest::TomlManifest;
use serde::{Deserialize, Serialize};

use crate::MessageIter;
use crate::lockfile::{NormalizedDependency, NormalizedPatch};

/// Input messages for `cargo-plumbing resolve-features`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum ResolveFeaturesIn {
    Manifest {
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        workspace: bool,
        /// The path to the manifest file that was read.
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        path: Utf8PathBuf,
        /// The package ID specification.
        ///
        /// This command also takes in virtual manifests and virtual manifests don't have
        /// [`PackageIdSpec`], hence the use of [`Option`].
        #[cfg_attr(
            feature = "unstable-schema",
            schemars(with = "Option<String>", description = "The package ID specification")
        )]
        pkg_id: Option<PackageIdSpec>,
        /// The fully parsed and deserialized manifest content.
        manifest: TomlManifest,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl ResolveFeaturesIn {
    /// Creates an iterator to parse a stream of [`ResolveFeaturesIn`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ResolveFeaturesOut {
    Activated {
        id: PackageIdSpec,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        features_for: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        features: Vec<String>,
    },
    Target {
        name: String,
        kind: String,
    },
}

#[test]
fn activated_roundtrip() {
    let mut messages = Vec::new();
    for features_for in ["", "host"] {
        for features in [vec![], vec!["default".to_owned()]] {
            let message = ResolveFeaturesOut::Activated {
                id: PackageIdSpec::new("demo".to_owned()),
                features_for: features_for.to_owned(),
                features,
            };
            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: ResolveFeaturesOut = serde_json::from_str(&serialized).unwrap();
            messages.push(deserialized);
        }
    }

    snapbox::assert_data_eq!(
        serde_json::to_string_pretty(&messages).unwrap(),
        snapbox::str![[r#"
[
  {
    "reason": "activated",
    "id": "demo"
  },
  {
    "reason": "activated",
    "id": "demo",
    "features": [
      "default"
    ]
  },
  {
    "reason": "activated",
    "id": "demo",
    "features_for": "host"
  },
  {
    "reason": "activated",
    "id": "demo",
    "features_for": "host",
    "features": [
      "default"
    ]
  }
]
"#]]
    );
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_resolve_features_schema() {
    let schema = schemars::schema_for!(ResolveFeaturesIn);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../resolve-features.in.schema.json").raw()
    );

    let schema = schemars::schema_for!(ResolveFeaturesOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../resolve-features.out.schema.json").raw()
    );
}
