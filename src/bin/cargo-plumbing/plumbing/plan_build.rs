use std::collections::{BTreeSet, HashMap};
use std::io::{BufReader, IsTerminal};
use std::path::PathBuf;
use std::{env, io};

use cargo::core::compiler::unit_dependencies::build_unit_dependencies;
use cargo::core::compiler::unit_graph::UnitDep;
use cargo::core::compiler::{
    CompileKind, CompileTarget, RustcTargetData, Unit, UnitInterner, UserIntent,
};
use cargo::core::manifest::TargetSourcePath;
use cargo::core::profiles::{DebugInfo, Lto, PanicStrategy, Profiles};
use cargo::core::resolver::features::{ActivateMap, FeatureOpts, FeaturesFor, ResolvedFeatures};
use cargo::core::resolver::{CliFeatures, ForceAllTargets, HasDevUnits};
use cargo::core::{FeatureValue, PackageIdSpecQuery, TargetKind, Workspace};
use cargo::ops::{
    get_resolved_packages, resolve_with_previous, CompileFilter, Packages, UnitGenerator,
};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::ops::resolve::{into_resolve, spec_to_id};
use cargo_plumbing_schemas::plan_build::{
    PlanBuildIn, PlanBuildOut, UnitDependency, UnitProfile, UnitTarget,
};

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
    /// Profile for the unit graph
    #[arg(long)]
    profile: Option<String>,
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

    let mut locked_packages = Vec::new();
    let mut unused_patches = None;
    let mut specs = Vec::new();
    let mut activated_features = ActivateMap::new();
    let mut req_bins = Vec::new();
    let mut req_tests = Vec::new();
    let mut req_benches = Vec::new();
    let mut req_examples = Vec::new();

    for message in messages {
        match message? {
            PlanBuildIn::LockedPackage { package } => locked_packages.push(package),
            PlanBuildIn::UnusedPatches { unused } => unused_patches = Some(unused),
            PlanBuildIn::Manifest { pkg_id, .. } => {
                if let Some(id) = pkg_id {
                    specs.push(id);
                }
            }
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
            PlanBuildIn::Target { name, kind } => match &*kind {
                "bin" => req_bins.push(name),
                "test" => req_tests.push(name),
                "bench" => req_benches.push(name),
                "example" => req_examples.push(name),
                _ => anyhow::bail!("unknown kind: {}", kind),
            },
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

    let requested_profiles = Profiles::new(&ws, args.profile.unwrap_or("dev".to_owned()).into())?;

    // We're using an arbitrary `UserIntent` because target selection is already handled in
    // `resolve-features` command, which should cover what `UserIntent` is doing.
    let user_intent = UserIntent::Build;

    // We want the targets to be inputted from stdin using messages.
    let lib = false;
    let all_bins = false;
    let all_tests = false;
    let all_examples = false;
    let all_benches = false;
    let all_targets = false;

    // Determine if we should include dev units from the selected targets.
    //
    // Note that this is the same `has_dev_units` inference as `resolve-features`.
    let has_dev_units =
        if !req_examples.is_empty() || !req_tests.is_empty() || !req_benches.is_empty() {
            HasDevUnits::Yes
        } else {
            HasDevUnits::No
        };

    let filter = CompileFilter::from_raw_arguments(
        lib,
        req_bins,
        all_bins,
        req_tests,
        all_tests,
        req_examples,
        all_examples,
        req_benches,
        all_benches,
        all_targets,
    );

    let resolve = into_resolve(&ws, locked_packages, unused_patches.unwrap_or_default())?;

    let features = activated_features
        .iter()
        .filter(|((id, _), _)| specs.iter().any(|spec| spec.matches(*id)))
        .flat_map(|s| s.1)
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut registry = ws.package_registry()?;
    let add_patches = true;
    let cli_features = CliFeatures::from_command_line(&features, false, true)?;
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

    // Define the packages to be built based on the input given
    let spec = Packages::Packages(specs.iter().map(|spec| spec.name().to_owned()).collect());

    let mut activated_dependencies = ActivateMap::new();
    for (k, requested_fs) in &activated_features {
        let (id, _) = k;
        if let Some(pkg) = packages.iter().find(|p| p.package_id() == *id) {
            let fs = pkg.summary().features();

            let dependencies: BTreeSet<_> = requested_fs
                .iter()
                .filter_map(|requested_f| fs.get(requested_f))
                .flatten()
                .filter_map(|f| match f {
                    FeatureValue::Dep { dep_name } => Some(dep_name),
                    _ => None,
                })
                .cloned()
                .collect();

            activated_dependencies.insert(*k, dependencies);
        }
    }

    let opts = FeatureOpts::new(&ws, has_dev_units, ForceAllTargets::No)?;
    let resolved_features = ResolvedFeatures {
        activated_features,
        activated_dependencies,
        opts,
    };

    let unit_interner = UnitInterner::new();

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

    for (id, (unit, unit_deps, root)) in units.iter().enumerate() {
        emit_unit(gctx, id, *root, unit, unit_deps, &indices)?;
    }

    Ok(())
}

fn emit_unit(
    gctx: &GlobalContext,
    id: usize,
    root: bool,
    unit: &Unit,
    unit_deps: &[UnitDep],
    indices: &HashMap<&Unit, usize>,
) -> CargoResult<()> {
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

    let profile = UnitProfile {
        name: unit.profile.name.to_string(),
        opt_level: unit.profile.opt_level.to_string(),
        lto: match unit.profile.lto {
            Lto::Off => "off".to_owned(),
            Lto::Bool(b) => b.to_string(),
            Lto::Named(n) => n.to_string(),
        },
        codegen_units: unit.profile.codegen_units,
        debuginfo: match unit.profile.debuginfo {
            DebugInfo::Resolved(d) => d,
            DebugInfo::Deferred(d) => d,
        },
        debug_assertions: unit.profile.debug_assertions,
        overflow_checks: unit.profile.overflow_checks,
        rpath: unit.profile.rpath,
        incremental: unit.profile.incremental,
        panic: match unit.profile.panic {
            PanicStrategy::Abort => "abort",
            PanicStrategy::Unwind => "unwind",
        }
        .to_owned(),
    };

    let features = unit.features.iter().map(|f| f.to_string()).collect();

    let msg = PlanBuildOut::Unit {
        id,
        platform,
        profile,
        target,
        deps,
        features,
        root,
    };
    gctx.shell().print_json(&msg)?;

    Ok(())
}
