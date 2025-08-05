use std::fmt::Debug;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Context as _;
use cargo::util::Filesystem;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::cargo::core::resolver::encode::EncodableResolve;
use cargo_plumbing_schemas::lockfile_contents::LockfileContentsMessage;

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

    let v: EncodableResolve = toml::from_str(&lock_s)?;
    gctx.shell()
        .print_json(&LockfileContentsMessage::Lockfile { version: v.version })?;

    let n = v.normalize()?;
    for package in n.package {
        gctx.shell()
            .print_json(&LockfileContentsMessage::LockedPackage { package })?;
    }
    if !n.patch.is_empty() {
        gctx.shell()
            .print_json(&LockfileContentsMessage::UnusedPatches { unused: n.patch })?;
    }
    if let Some(metadata) = n.metadata {
        gctx.shell()
            .print_json(&LockfileContentsMessage::Metadata { metadata })?;
    }

    Ok(())
}
