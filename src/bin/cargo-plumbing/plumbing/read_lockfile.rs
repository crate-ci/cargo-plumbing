use std::fmt::Debug;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Context as _;
use cargo::util::Filesystem;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::ops::resolve::normalize_resolve;
use cargo_plumbing_schemas::read_lockfile::ReadLockfileOut;
use cargo_util_schemas::lockfile::TomlLockfile;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the lockfile
    #[clap(long)]
    lockfile_path: PathBuf,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let lock_path = gctx.cwd().join(args.lockfile_path);
    if let Some(file_name) = lock_path.file_name() {
        if file_name != "Cargo.lock" {
            anyhow::bail!("lockfile name should be `Cargo.lock`");
        }
    }

    let root = lock_path.parent().expect("Lockfile path can't be root");
    let lock_root = Filesystem::new(root.to_owned());

    let mut lock_f = lock_root.open_ro_shared("Cargo.lock", gctx, "Cargo.lock file")?;
    let mut lock_s = String::new();
    lock_f
        .read_to_string(&mut lock_s)
        .with_context(|| format!("failed to read file: {}", lock_f.path().display()))?;

    let v: TomlLockfile = toml::from_str(&lock_s)?;
    let n = normalize_resolve(v)?;

    gctx.shell()
        .print_json(&ReadLockfileOut::Lockfile { version: n.version })?;
    for package in n.package {
        gctx.shell()
            .print_json(&ReadLockfileOut::LockedPackage { package })?;
    }
    if !n.patch.is_empty() {
        gctx.shell()
            .print_json(&ReadLockfileOut::UnusedPatches { unused: n.patch })?;
    }

    Ok(())
}
