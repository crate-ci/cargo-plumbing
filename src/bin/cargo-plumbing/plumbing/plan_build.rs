use std::collections::HashMap;
use std::io::{BufReader, IsTerminal};
use std::path::PathBuf;
use std::{env, io};

use cargo::core::compiler::unit_dependencies::build_unit_dependencies;
use cargo::core::compiler::unit_graph::UnitDep;
use cargo::core::compiler::{
    CompileKind, CompileTarget, RustcTargetData, Unit, UnitInterner, UserIntent,
};
use cargo::core::manifest::TargetSourcePath;
use cargo::core::profiles::Profiles;
use cargo::core::resolver::features::{ActivateMap, FeatureOpts, FeaturesFor, ResolvedFeatures};
use cargo::core::resolver::{CliFeatures, ForceAllTargets, HasDevUnits};
use cargo::core::{TargetKind, Workspace};
use cargo::ops::{
    get_resolved_packages, resolve_with_previous, CompileFilter, Packages, UnitGenerator,
};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::ops::resolve::{into_resolve, spec_to_id};
use cargo_plumbing_schemas::plan_build::{PlanBuildIn, PlanBuildOut, UnitDependency, UnitTarget};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    /// Target triple
    #[arg(long)]
    target: Vec<String>,
    /// Include dev units
    //
    // HACK: We're asking the users if they want to include dev units or not. Ideally this should
    // be done through stdin messages. However, due to cargo API limitations, this workaround is necessary
    //
    // See: https://github.com/crate-ci/cargo-plumbing/pull/68#discussion_r2277484208
    #[arg(long, default_value_t = false)]
    dev_units: bool,
    /// List of features to activate
    #[arg(long, short = 'F')]
    features: Vec<String>,
    /// Activate all available features
    #[arg(long)]
    all_features: bool,
    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
    /// Profile for the unit graph
    #[arg(long)]
    profile: Option<String>,
    /// The intent to compile in
    #[arg(long)]
    intent: String,

    #[arg(long)]
    all: bool,
    #[arg(long)]
    exclude: Vec<String>,
    #[arg(long)]
    package: Vec<String>,

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

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
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

    let messages = PlanBuildIn::parse_stream(BufReader::new(stdin));

    let mut lock_version = None;
    let mut locked_packages = Vec::new();
    let mut unused_patches = None;
    let mut specs = Vec::new();
    let mut activated_features = ActivateMap::new();

    for message in messages {
        match message? {
            PlanBuildIn::Lockfile { version } => lock_version = version,
            PlanBuildIn::LockedPackage { package } => locked_packages.push(package),
            PlanBuildIn::UnusedPatches { unused } => unused_patches = Some(unused),
            PlanBuildIn::Manifest { id } => specs.push(id),
            PlanBuildIn::Activated {
                id,
                features,
                features_for,
            } => {
                let Ok(Some(pkg_id)) = spec_to_id(id.clone(), None, None) else {
                    continue;
                };

                let features_for = match &*features_for {
                    "host" => FeaturesFor::HostDep,
                    "" => FeaturesFor::NormalOrDev,
                    target => FeaturesFor::ArtifactDep(CompileTarget::new(target)?),
                };

                let k = (pkg_id, features_for);
                let v = features.into_iter().map(|feat| feat.into()).collect();
                activated_features.insert(k, v);
            }
        }
    }

    if locked_packages.is_empty() {
        anyhow::bail!("incomplete input. no packages found.");
    }

    if activated_features.is_empty() {
        anyhow::bail!("incomplete input. no activated features found.");
    }

    let requested_kinds = CompileKind::from_requested_targets(gctx, &args.target)?;
    let target_data = RustcTargetData::new(&ws, &requested_kinds)?;

    let explicit_host_kind = CompileKind::Target(CompileTarget::new(&target_data.rustc.host)?);

    let unit_interner = UnitInterner::new();

    // HACK: We're asking the users if they want to include dev units or not. Ideally this should
    // be done through stdin messages. However, due to cargo API limitations, this workaround is necessary
    //
    // See: https://github.com/crate-ci/cargo-plumbing/pull/68#discussion_r2277484208
    let has_dev_units = if args.dev_units {
        HasDevUnits::Yes
    } else {
        HasDevUnits::No
    };

    let requested_profiles = Profiles::new(&ws, args.profile.unwrap_or("dev".to_owned()).into())?;

    let user_intent = match &*args.intent {
        "check" => UserIntent::Check { test: false },
        "build" => UserIntent::Build,
        "test" => UserIntent::Test,
        "bench" => UserIntent::Bench,
        _ => anyhow::bail!("unknown intent."),
    };

    let resolve = into_resolve(
        &ws,
        lock_version,
        locked_packages,
        unused_patches.unwrap_or_default(),
    )?;

    let mut registry = ws.package_registry()?;
    let add_patches = true;
    let cli_features = CliFeatures::from_command_line(
        &args.features,
        args.all_features,
        !args.no_default_features,
    )?;
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

    let pkg_set = get_resolved_packages(&resolve_with_overrides, registry)?;

    let to_build_ids = resolve_with_overrides.specs_to_ids(&specs)?;
    let to_builds = pkg_set.get_many(to_build_ids)?;
    let spec_names = specs.iter().map(|spec| spec.name()).collect::<Vec<_>>();
    let packages = to_builds
        .iter()
        .filter(|package| spec_names.contains(&package.name().as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let spec = Packages::from_flags(args.all, args.exclude, args.package)?;

    let filter = CompileFilter::from_raw_arguments(
        args.lib,
        args.bin,
        args.bins,
        args.test,
        args.tests,
        args.example,
        args.examples,
        args.bench,
        args.benches,
        args.all_targets,
    );

    let opts = FeatureOpts::new(&ws, has_dev_units, ForceAllTargets::No)?;
    let resolved_features = ResolvedFeatures {
        activated_features,
        activated_dependencies: HashMap::new(),
        opts,
    };

    let unit_generator = UnitGenerator {
        ws: &ws,
        packages: &packages,
        spec: &spec,
        target_data: &target_data,
        filter: &filter,
        requested_kinds: &requested_kinds,
        explicit_host_kind,
        intent: user_intent,
        resolve: &resolve_with_overrides,
        workspace_resolve: &Some(resolve),
        resolved_features: &resolved_features,
        package_set: &pkg_set,
        profiles: &requested_profiles,
        interner: &unit_interner,
        has_dev_units,
    };
    let root_units = unit_generator.generate_root_units()?;

    let mut unit_graph = build_unit_dependencies(
        &ws,
        &pkg_set,
        &resolve_with_overrides,
        &resolved_features,
        None,
        &root_units,
        &[],
        &HashMap::new(),
        user_intent,
        &target_data,
        &requested_profiles,
        &unit_interner,
    )?;

    let mut units: Vec<_> = root_units
        .into_iter()
        .map(|root_unit| {
            let unit_deps = unit_graph
                .remove(&root_unit)
                .expect("BUG: root units not part of unit graph");
            (root_unit, unit_deps, true)
        })
        .collect();
    units.extend(
        unit_graph
            .into_iter()
            .map(|(unit, unit_deps)| (unit, unit_deps, false))
            .collect::<Vec<_>>(),
    );
    units.sort_unstable();

    let indices: HashMap<&Unit, usize> = units
        .iter()
        .enumerate()
        .map(|(i, val)| (&val.0, i))
        .collect();

    for (unit, unit_deps, root) in &units {
        emit_unit(gctx, *root, unit, unit_deps, &indices)?;
    }

    Ok(())
}

fn emit_unit(
    gctx: &GlobalContext,
    root: bool,
    unit: &Unit,
    unit_deps: &[UnitDep],
    indices: &HashMap<&Unit, usize>,
) -> CargoResult<()> {
    let id = unit.pkg.package_id().to_spec();

    let deps = unit_deps
        .iter()
        .map(|unit_dep| UnitDependency {
            index: indices[&unit_dep.unit],
            extern_crate_name: unit_dep.extern_crate_name.to_string(),
            public: unit_dep.public,
            noprelude: unit_dep.noprelude,
        })
        .collect::<Vec<_>>();

    let platform = match unit.kind {
        CompileKind::Host => "host".to_owned(),
        CompileKind::Target(target) => target.rustc_target().to_string(),
    };

    let target = UnitTarget {
        crate_types: unit
            .target
            .rustc_crate_types()
            .into_iter()
            .map(|ty| ty.as_str().to_owned())
            .collect(),
        edition: unit.target.edition().to_string(),
        kind: match unit.target.kind() {
            TargetKind::Lib(kinds) => kinds.iter().map(|kind| kind.to_string()).collect(),
            TargetKind::Bin => vec!["bin".to_owned()],
            TargetKind::ExampleBin | TargetKind::ExampleLib(_) => vec!["example".to_owned()],
            TargetKind::Test => vec!["test".to_owned()],
            TargetKind::CustomBuild => vec!["custom-build".to_owned()],
            TargetKind::Bench => vec!["bench".to_owned()],
        },
        name: unit.target.name().to_owned(),
        src_path: match unit.target.src_path() {
            TargetSourcePath::Path(path) => Some(path.clone()),
            TargetSourcePath::Metabuild => None,
        },
        test: unit.target.tested(),
        doctest: unit.target.doctested() && unit.target.doctestable(),
    };

    let msg = PlanBuildOut::Unit {
        id,
        platform,
        target,
        deps,
        root,
    };
    gctx.shell().print_json(&msg)?;

    Ok(())
}
