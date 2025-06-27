use serde::{Deserialize, Serialize};

use crate::Message;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub struct ManifestLocation {
    pub manifest_path: String,
}

impl Message for ManifestLocation {
    fn reason(&self) -> &str {
        "manifest-location"
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_project_location_schema() {
    let schema = schemars::schema_for!(ManifestLocation);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../manifest-location.schema.json").raw()
    );
}
