use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cargo_plumbing::CargoResult;
use cargo_plumbing_schemas::locate_manifest::LocateManifestMessage;
use cargo_plumbing_schemas::read_manifest::{ReadManifestMessage, TomlManifest};
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    workspace: bool,
}

fn run(args: &Args) -> CargoResult<()> {
    let manifest_path = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "locate-manifest"])
            .stdout(Stdio::piped());

        if let Some(manifest_path) = &args.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }

        let mut child = cmd.spawn().expect("failed to run locate-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = LocateManifestMessage::parse_stream(BufReader::new(stdout));

        let mut manifest_path: Option<String> = None;

        #[allow(clippy::never_loop)]
        for message in messages {
            match message.expect("failed to parse message") {
                LocateManifestMessage::ManifestLocation { manifest_path: m } => {
                    manifest_path = Some(m);
                    break;
                }
            }
        }

        child.wait().expect("failed to wait for locate-manifest");
        manifest_path.expect("failed to get manifest_path")
    };

    let _manifests: Vec<TomlManifest> = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "read-manifest"])
            .args(["--manifest-path", &manifest_path])
            .stdout(Stdio::piped());

        if args.workspace {
            cmd.arg("--workspace");
        }

        let mut child = cmd.spawn().expect("failed to run read-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = ReadManifestMessage::parse_stream(BufReader::new(stdout));

        let manifests = messages.map(|message| match message.expect("failed to parse message") {
            ReadManifestMessage::Manifest { manifest, .. } => manifest,
        });

        child.wait().expect("failed to wait for read-manifest");
        manifests.collect()
    };

    anyhow::bail!("check for {manifest_path} is not implemented!");
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {e}"),
    }
}
