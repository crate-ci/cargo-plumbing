use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cargo::CargoResult;
use cargo_plumbing_schemas::locate_manifest::LocateManifestOut;
use cargo_plumbing_schemas::read_manifest::ReadManifestOut;
use cargo_plumbing_schemas::resolve_features::ResolveFeaturesIn;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    #[arg(long)]
    lockfile_path: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    workspace: bool,
    #[arg(long, short = 'F')]
    features: Vec<String>,
    #[arg(long)]
    all_features: bool,
    #[arg(long)]
    no_default_features: bool,

    #[arg(long)]
    lib: bool,
    #[arg(long)]
    bins: bool,
    #[arg(long)]
    bin: Vec<String>,
    #[arg(long)]
    examples: bool,
    #[arg(long)]
    example: Vec<String>,
    #[arg(long)]
    tests: bool,
    #[arg(long)]
    test: Vec<String>,
    #[arg(long)]
    benches: bool,
    #[arg(long)]
    bench: Vec<String>,
    #[arg(long)]
    all_targets: bool,
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
        let messages = LocateManifestOut::parse_stream(BufReader::new(stdout));

        let mut manifest_path: Option<camino::Utf8PathBuf> = None;

        #[allow(clippy::never_loop)]
        for message in messages {
            match message.expect("failed to parse message") {
                LocateManifestOut::ManifestLocation { manifest_path: m } => {
                    manifest_path = Some(m);
                    break;
                }
            }
        }

        child.wait().expect("failed to wait for locate-manifest");
        manifest_path.expect("failed to get manifest_path")
    };

    let manifests: Vec<ReadManifestOut> = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "read-manifest"])
            .args(["--manifest-path", manifest_path.as_str()])
            .stdout(Stdio::piped());

        if args.workspace {
            cmd.arg("--workspace");
        }

        let mut child = cmd.spawn().expect("failed to run read-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = ReadManifestOut::parse_stream(BufReader::new(stdout));

        child.wait().expect("failed to wait for read-manifest");
        messages.collect::<Result<Vec<_>, _>>()?
    };

    let lockfile_path = args.lockfile_path.clone().unwrap_or_else(|| {
        let ws_manifest = manifests.iter().find(|m| {
            matches!(
                m,
                ReadManifestOut::Manifest {
                    workspace: true,
                    ..
                }
            )
        });

        let ReadManifestOut::Manifest { path, .. } = ws_manifest.unwrap();
        path.join("Cargo.lock")
    });

    let lockfile = {
        if lockfile_path.is_file() {
            let mut cmd = Command::new("cargo");
            cmd.args(["run", "plumbing", "read-lockfile"])
                .arg("--lockfile-path")
                .arg(&lockfile_path)
                .stdout(Stdio::piped());

            let out = cmd.output().expect("failed to run read-lockfile");

            if out.stdout.is_empty() {
                None
            } else {
                Some(out.stdout)
            }
        } else {
            None
        }
    };

    let locked_deps = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "lock-dependencies"])
            .arg("--manifest-path")
            .arg(&manifest_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to run read-manifest");
        let mut stdout = child.stdout.take().expect("failed to take stdout");
        let mut stdin = child.stdin.take().expect("failed to take stdin");

        if let Some(ref lockfile) = lockfile {
            stdin.write_all(lockfile).expect("failed to write to stdin");
            drop(stdin);
        }

        let mut buffer = Vec::new();
        stdout
            .read_to_end(&mut buffer)
            .expect("failed to read stdout");

        child.wait().expect("failed to wait for lock-dependencies");
        buffer
    };

    if lockfile.is_some_and(|lockfile| lockfile != locked_deps) {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "write-lockfile"])
            .arg("--lockfile-path")
            .arg(&lockfile_path)
            .stdin(Stdio::piped());

        let mut child = cmd.spawn().expect("failed spawn lock-dependencies");
        let mut stdin = child.stdin.take().expect("failed to take stdin");

        stdin
            .write_all(&locked_deps)
            .expect("failed to write to stdin");
        drop(stdin);

        child.wait().expect("failed to wait for lock-dependencies");
    }

    // HACK: We determine if we should include dev units or not. This workaround is implemented due
    // to cargo API limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/pull/68#discussion_r2277484208
    let has_dev_units = args.examples
        || args.tests
        || args.benches
        || !args.example.is_empty()
        || !args.test.is_empty()
        || !args.bench.is_empty()
        || args.all_targets;

    let _features = {
        let ids = manifests
            .into_iter()
            .filter_map(|manifest| match manifest {
                ReadManifestOut::Manifest { pkg_id, .. } => {
                    pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
                }
            })
            .map(|msg| serde_json::to_string(&msg))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");

        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "resolve-features"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        if has_dev_units {
            cmd.arg("--dev-units");
        }

        let mut child = cmd.spawn().expect("failed to spawn resolve-features");

        {
            let mut stdin = child.stdin.take().expect("failed to take stdin");
            stdin
                .write_all(ids.as_bytes())
                .expect("failed to write to stdin");
            stdin
                .write_all(&locked_deps)
                .expect("failed to write to stdin");
        }

        let messages = {
            let stdout = child.stdout.take().expect("failed to get stdout");
            LocateManifestOut::parse_stream(BufReader::new(stdout))
        };

        child.wait().expect("failed to wait for resolve-features");
        messages.collect::<Result<Vec<_>, _>>()?
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
