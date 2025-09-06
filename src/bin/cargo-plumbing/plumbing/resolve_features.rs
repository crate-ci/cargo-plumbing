use std::env;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

use cargo::core::compiler::{CompileKind, RustcTargetData};
use cargo::core::resolver::features::{FeatureOpts, FeatureResolver};
use cargo::core::resolver::{CliFeatures, HasDevUnits};
use cargo::core::Workspace;
use cargo::ops::{get_resolved_packages, resolve_with_previous};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::ops::resolve::into_resolve;
use cargo_plumbing_schemas::resolve_features::{ResolveFeaturesIn, ResolveFeaturesOut};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    /// List of features to activate
    #[arg(long, short = 'F')]
    features: Vec<String>,
    /// Activate all available features
    #[arg(long)]
    all_features: bool,
    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
    /// Target triple
    #[arg(long)]
    target: Vec<String>,
    /// Include this package's library
    #[arg(long)]
    lib: bool,
    /// Include all binaries
    #[arg(long)]
    bins: bool,
    /// Include only the specified binaries
    #[arg(long)]
    bin: Vec<String>,
    /// Include all examples
    #[arg(long)]
    examples: bool,
    /// Include only the specified examples
    #[arg(long)]
    example: Vec<String>,
    /// Include all tests
    #[arg(long)]
    tests: bool,
    /// Include only the specified tests
    #[arg(long)]
    test: Vec<String>,
    /// Include all benches
    #[arg(long)]
    benches: bool,
    /// Include only the specified benches
    #[arg(long)]
    bench: Vec<String>,
    /// Include all targets
    #[arg(long)]
    all_targets: bool,
}

pub(crate) fn exec(gctx: &mut GlobalContext, args: Args) -> CargoResult<()> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        anyhow::bail!("input must be piped from a file or another command");
    }

    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    let manifest_path = args
        .manifest_path
        .unwrap_or(env::current_dir()?.join("Cargo.toml"));
    let path = gctx.cwd().join(manifest_path);
    let ws = Workspace::new(&path, gctx)?;

    let messages = ResolveFeaturesIn::parse_stream(BufReader::new(stdin));

    let mut ws_manifest = None;
    let mut locked_packages = Vec::new();
    let mut unused_patches = None;
    let mut specs = Vec::new();

    for message in messages {
        match message? {
            ResolveFeaturesIn::LockedPackage { package } => locked_packages.push(package),
            ResolveFeaturesIn::UnusedPatches { unused } => unused_patches = Some(unused),
            ResolveFeaturesIn::Manifest {
                pkg_id,
                manifest,
                workspace,
                ..
            } => {
                if workspace {
                    if ws_manifest.is_some() {
                        anyhow::bail!("duplicate workspace manifest input.");
                    }
                    ws_manifest = Some(manifest);
                }
                if let Some(id) = pkg_id {
                    specs.push(id);
                }
            }
        }
    }

    if locked_packages.is_empty() {
        anyhow::bail!("incomplete input. no packages found.");
    }
    let Some(ws_manifest) = ws_manifest else {
        anyhow::bail!("missing workspace manifest input.")
    };

    let mut requested_features = args.features;
    if let Some(bins) = ws_manifest.bin {
        let bins = bins
            .into_iter()
            .filter(|bin| {
                args.all_targets
                    || args.bins
                    || bin
                        .name
                        .as_ref()
                        .map(|name| args.bin.contains(name))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        for bin in bins.iter() {
            if let Some(name) = &bin.name {
                gctx.shell().print_json(&ResolveFeaturesOut::Target {
                    name: name.clone(),
                    kind: "bin".to_owned(),
                })?;
            }
        }
        let bins = bins
            .into_iter()
            .filter_map(|bin| bin.required_features)
            .flatten()
            .collect::<Vec<_>>();
        requested_features.extend(bins);
    }
    if let Some(tests) = ws_manifest.test {
        let tests = tests
            .into_iter()
            .filter(|test| {
                args.all_targets
                    || args.tests
                    || test
                        .name
                        .as_ref()
                        .map(|name| args.test.contains(name))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        for test in tests.iter() {
            if let Some(name) = &test.name {
                gctx.shell().print_json(&ResolveFeaturesOut::Target {
                    name: name.clone(),
                    kind: "test".to_owned(),
                })?;
            }
        }
        let tests = tests
            .into_iter()
            .filter_map(|test| test.required_features)
            .flatten()
            .collect::<Vec<_>>();
        requested_features.extend(tests);
    }
    if let Some(benches) = ws_manifest.bench {
        let benches = benches
            .into_iter()
            .filter(|bench| {
                args.all_targets
                    || args.benches
                    || bench
                        .name
                        .as_ref()
                        .map(|name| args.bench.contains(name))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        for bench in benches.iter() {
            if let Some(name) = &bench.name {
                gctx.shell().print_json(&ResolveFeaturesOut::Target {
                    name: name.clone(),
                    kind: "bench".to_owned(),
                })?;
            }
        }
        let benches = benches
            .into_iter()
            .filter_map(|bench| bench.required_features)
            .flatten()
            .collect::<Vec<_>>();
        requested_features.extend(benches);
    }
    if let Some(examples) = ws_manifest.example {
        let examples = examples
            .into_iter()
            .filter(|example| {
                args.all_targets
                    || args.examples
                    || example
                        .name
                        .as_ref()
                        .map(|name| args.example.contains(name))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        for example in examples.iter() {
            if let Some(name) = &example.name {
                gctx.shell().print_json(&ResolveFeaturesOut::Target {
                    name: name.clone(),
                    kind: "example".to_owned(),
                })?;
            }
        }
        let examples = examples
            .into_iter()
            .filter_map(|example| example.required_features)
            .flatten()
            .collect::<Vec<_>>();
        requested_features.extend(examples);
    }

    let resolve = into_resolve(&ws, locked_packages, unused_patches.unwrap_or_default())?;
    let cli_features = CliFeatures::from_command_line(
        &requested_features,
        args.all_features,
        !args.no_default_features,
    )?;

    // Determine if we should include dev units from the selected targets.
    let has_dev_units = if args.examples
        || args.tests
        || args.benches
        || !args.example.is_empty()
        || !args.test.is_empty()
        || !args.bench.is_empty()
        || args.all_targets
    {
        HasDevUnits::Yes
    } else {
        HasDevUnits::No
    };

    let mut registry = ws.package_registry()?;
    let add_patches = true;
    let resolve_with_overrides = resolve_with_previous(
        &mut registry,
        &ws,
        &cli_features,
        has_dev_units,
        Some(&resolve),
        None,
        &specs,
        add_patches,
    )?;

    let requested_kinds = CompileKind::from_requested_targets(gctx, &args.target)?;
    let mut target_data = RustcTargetData::new(&ws, &requested_kinds)?;
    let members_with_features = ws.members_with_features(&specs, &cli_features)?;
    let member_ids = members_with_features
        .iter()
        .map(|(p, _fts)| p.package_id())
        .collect::<Vec<_>>();
    let force_all_targets = cargo::core::resolver::ForceAllTargets::No;

    // HACK: The resolver must download packages before it can resolve features. This is a
    // workaround for a known limitation of the feature resolver.
    //
    // See: https://github.com/rust-lang/cargo/issues/15834
    let pkg_set = get_resolved_packages(&resolve_with_overrides, registry)?;
    pkg_set.download_accessible(
        &resolve_with_overrides,
        &member_ids,
        has_dev_units,
        &requested_kinds,
        &target_data,
        force_all_targets,
    )?;

    let feature_opts = FeatureOpts::new(&ws, has_dev_units, force_all_targets)?;
    let resolved_features = FeatureResolver::resolve(
        &ws,
        &mut target_data,
        &resolve_with_overrides,
        &pkg_set,
        &cli_features,
        &specs,
        &requested_kinds,
        feature_opts,
    )?;

    for ((id, feat_for), feats) in resolved_features.activated_features {
        let id = id.to_spec();
        let features_for = feat_for.to_string();
        let features = feats.iter().map(|feat| feat.to_string()).collect();
        gctx.shell().print_json(&ResolveFeaturesOut::Activated {
            id,
            features_for,
            features,
        })?;
    }

    Ok(())
}
