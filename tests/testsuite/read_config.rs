use std::path::Path;

use cargo_plumbing_schemas::read_config::ReadConfigOut;

use super::common::{run_cmd, MANIFEST_PATH};

pub fn run() -> crate::TestResult {
    let output = run_cmd("read-config", &[("--manifest-path", MANIFEST_PATH)])?;
    let messages: Vec<ReadConfigOut> = output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(messages.len(), 1);

    match &messages[0] {
        ReadConfigOut::Config {
            manifest_path,
            target_directory,
            build_directory,
        } => {
            assert!(manifest_path.ends_with("Cargo.toml"));
            assert!(target_directory.ends_with("target") || target_directory.contains("target"));
            assert!(build_directory.ends_with("target") || build_directory.contains("target"));
        }
    }

    Ok(())
}
