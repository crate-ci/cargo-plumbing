use std::path::PathBuf;
use std::process::Command;

use cargo_plumbing::CargoResult;
use cargo_plumbing_schemas::locate_manifest::ManifestLocation;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

fn run(args: &Args) -> CargoResult<()> {
    let locate_manifest: ManifestLocation = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "locate-manifest"]);

        if let Some(manifest_path) = &args.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }

        let output = cmd.output().expect("failed to run locate-project");
        let stdout = String::from_utf8(output.stdout)?;

        serde_json::from_str(&stdout).expect("failed to parse JSON from locate-project")
    };

    let manifest_path = locate_manifest.manifest_path;

    anyhow::bail!("check for {manifest_path} is not implemented!");
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {e}"),
    }
}
