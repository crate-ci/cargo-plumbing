use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use camino::Utf8PathBuf;
use cargo::core::PackageIdSpec;
use cargo::util::interning::InternedString;
use cargo::CargoResult;
use cargo_plumbing_schemas::locate_manifest::LocateManifestOut;
use cargo_plumbing_schemas::read_manifest::ReadManifestOut;
use cargo_util_schemas::manifest::{Hints, RustVersion, VecStringOrBool};
use clap::Parser;
use semver::Version;
use serde::Serialize;

const VERSION: u32 = 1;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    #[arg(long)]
    lockfile_path: Option<PathBuf>,

    /// Output information only about the workspace members and don't fetch dependencies
    #[arg(long)]
    no_deps: bool,
    /// Format version [possible values: 1]
    #[arg(long, value_name = "VERSION")]
    format_version: u32,
    /// Only include resolve dependencies matching the given target-triple
    #[arg(long, value_name = "TRIPLE")]
    filter_platform: Vec<String>,
}

fn main() -> CargoResult<()> {
    let args = Args::parse();

    if args.format_version != VERSION {
        anyhow::bail!("unknown format version");
    }
    if !args.no_deps {
        anyhow::bail!("metadata with deps hasn't been implemented");
    }

    // We first locate the manifest based on the provided path from `--manifest-path`.
    let manifest_path = {
        let mut cmd = cargo_plumbing_bin();
        cmd.args(["plumbing", "locate-manifest"])
            .stdout(Stdio::piped());

        // The `--manifest-path` flag is used to help cargo-plumbing locate the manifest path by
        // giving it a starting point. When not using `--manifest-path`, cargo-plumbing uses the
        // current directory in which the command is invoked.
        if let Some(manifest_path) = &args.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }

        let mut child = cmd.spawn().expect("failed to run locate-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = LocateManifestOut::parse_stream(BufReader::new(stdout));

        let mut manifest_path: Option<Utf8PathBuf> = None;

        // The output of `locate-manifest` is in the form of a `ManifestLocation` message. We
        // extract this information from stdout.
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

    // Next, we read the workspace members. Since this example is a `cargo metadata --no-deps`
    // example, the packages are from the workspace.
    let ws_members: Vec<ReadManifestOut> = {
        let mut cmd = cargo_plumbing_bin();
        cmd.args(["plumbing", "read-manifest"])
            .args(["--manifest-path", manifest_path.as_str()])
            .arg("--workspace")
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to run read-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = ReadManifestOut::parse_stream(BufReader::new(stdout));

        child.wait().expect("failed to wait for read-manifest");
        messages.collect::<Result<Vec<_>, _>>()?
    };

    let mut workspace_root = None;
    let mut workspace_members = Vec::new();
    let mut workspace_default_members = Vec::new();
    let mut workspace_metadata = None;
    let mut packages = Vec::new();
    let resolve: Option<()> = None;

    let mut expanded_members = Vec::new();
    let mut expanded_default_members = Vec::new();

    // Here, we are now extracting information from the `read-manifest` invocation.
    for m in ws_members.iter() {
        match m {
            ReadManifestOut::Manifest {
                path,
                pkg_id,
                manifest,
                ..
            } => {
                if let Some(ws) = &manifest.workspace {
                    // We are now reading the workspace manifest.

                    workspace_root = Some(path.clone());
                    workspace_metadata = ws.metadata.as_ref();

                    // If the workspace manifest is a package, we add that to the members list,
                    // both the members and default members list.
                    if let Some(ref id) = pkg_id {
                        workspace_members.push(id.clone());
                        workspace_default_members.push(id.clone());
                    }

                    // The path is passed with a `Cargo.toml` suffix. We want to resolve globs
                    // based on this path. Having a suffix would break the logic.
                    let mut path = path.clone().into_std_path_buf();
                    if path.ends_with("Cargo.toml") {
                        path.pop();
                    }

                    if let Some(ref m) = ws.members {
                        expanded_members = m.clone();
                    }
                    if let Some(ref m) = ws.default_members {
                        expanded_default_members = m.clone();
                    }
                } else {
                    // We are now reading the member's manifest.

                    // Workspace member should have a package ID. Not having a package ID would
                    // mean the implementation is incorrect.
                    let id = pkg_id.clone().unwrap();

                    // If the current manifest a part of the workspace member, we add that to the
                    // list of members.
                    for m_path in &expanded_members {
                        if m_path == path {
                            workspace_members.push(id.clone());
                            continue;
                        }
                    }

                    // If the current manifest is a part of the default workspace member, we add
                    // that to the list of default members.
                    for m_path in &expanded_default_members {
                        if m_path == path {
                            workspace_default_members.push(id.clone());
                            continue;
                        }
                    }
                }

                // We need to add this package into the list of package. The format used by
                // cargo-plumbing and format used by `cargo metadata` differs. We have to translate
                // them first.
                if let Some(id) = pkg_id {
                    if let Some(package) = &manifest.package {
                        packages.push(SerializedPackage {
                            id,
                            name: package.normalized_name()?.to_string(),
                            version: package.normalized_version()?.unwrap(),
                            license: package.normalized_license()?,
                            license_file: package.normalized_license_file()?,
                            description: package.normalized_description()?,
                            source: (),
                            dependencies: Vec::new(),
                            targets: Vec::new(),
                            features: BTreeMap::new(),
                            manifest_path: path.clone().into_std_path_buf(),
                            metadata: package.metadata.as_ref(),
                            publish: match package.normalized_publish()? {
                                Some(VecStringOrBool::VecString(v)) => Some(v),
                                _ => None,
                            },
                            authors: package.normalized_authors()?.cloned().unwrap_or_default(),
                            categories: package
                                .normalized_categories()?
                                .cloned()
                                .unwrap_or_default(),
                            keywords: package.normalized_keywords()?.cloned().unwrap_or_default(),
                            readme: package.normalized_readme()?,
                            repository: package.normalized_repository()?,
                            homepage: package.normalized_homepage()?,
                            documentation: package.normalized_documentation()?,
                            edition: package
                                .normalized_edition()?
                                .cloned()
                                .unwrap_or(String::from("2015")),
                            links: package.links.as_ref(),
                            metabuild: package
                                .metabuild
                                .as_ref()
                                .map(|metabuild| metabuild.0.clone()),
                            default_run: package.default_run.as_ref(),
                            rust_version: package.normalized_rust_version()?,
                            hints: manifest.hints.as_ref(),
                        });
                    }
                }
            }
        }
    }

    let workspace_root = workspace_root.unwrap_or(manifest_path).into_std_path_buf();

    // Lastly, we print out the constructed `cargo metadata` output.
    let out = CargoMetadataOutput {
        packages,
        workspace_members,
        workspace_default_members,
        resolve,
        target_directory: PathBuf::new(),
        build_directory: PathBuf::new(),
        version: VERSION,
        workspace_root,
        metadata: workspace_metadata,
    };
    println!("{}", serde_json::to_string(&out).unwrap());

    Ok(())
}

/// The serialized package of `cargo metadata` output.
///
/// This implementation differs slightly from cargo's. Here, we're borrowing most of the fields,
/// whereas in cargo, we own them.
#[derive(Serialize)]
struct SerializedPackage<'a> {
    id: &'a PackageIdSpec,
    name: String,
    version: &'a Version,
    license: Option<&'a String>,
    license_file: Option<&'a String>,
    description: Option<&'a String>,
    source: (),
    dependencies: Vec<()>,
    targets: Vec<String>,
    features: BTreeMap<String, Vec<InternedString>>,
    manifest_path: PathBuf,
    metadata: Option<&'a toml::Value>,
    publish: Option<&'a Vec<String>>,
    authors: Vec<String>,
    categories: Vec<String>,
    keywords: Vec<String>,
    readme: Option<&'a String>,
    repository: Option<&'a String>,
    homepage: Option<&'a String>,
    documentation: Option<&'a String>,
    edition: String,
    links: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metabuild: Option<Vec<String>>,
    default_run: Option<&'a String>,
    rust_version: Option<&'a RustVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hints: Option<&'a Hints>,
}

/// The output data of `cargo metadata`.
#[derive(Serialize)]
struct CargoMetadataOutput<'a> {
    packages: Vec<SerializedPackage<'a>>,
    workspace_members: Vec<PackageIdSpec>,
    workspace_default_members: Vec<PackageIdSpec>,
    resolve: Option<()>,
    target_directory: PathBuf,
    build_directory: PathBuf,
    version: u32,
    workspace_root: PathBuf,
    metadata: Option<&'a toml::Value>,
}

/// [`Command`] for launching cargo-plumbing binary
///
/// The `CARGO_PLUMBING_BIN` env variable should point to a `cargo-plumbing` binary. It is used
/// mostly for testing examples.
///
/// See <https://github.com/crate-ci/cargo-plumbing/issues/108>
fn cargo_plumbing_bin() -> Command {
    if let Ok(bin) = std::env::var("CARGO_PLUMBING_BIN") {
        Command::new(bin)
    } else {
        let mut cmd = Command::new("cargo");
        cmd.arg("run");
        cmd
    }
}
