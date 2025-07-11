use std::fmt::Debug;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Context as _;
use cargo::core::resolver::EncodableResolve;
use cargo::util::Filesystem;
use cargo::{CargoResult, GlobalContext};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the lockfile
    #[clap(long)]
    lockfile_path: PathBuf,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let lock_path = gctx.cwd().join(args.lockfile_path);
    let root = lock_path.parent().expect("Lockfile path can't be root");
    let lock_root = Filesystem::new(root.to_owned());

    let mut lock_f = lock_root.open_ro_shared("Cargo.lock", gctx, "Cargo.lock file")?;
    let mut lock_s = String::new();
    lock_f
        .read_to_string(&mut lock_s)
        .with_context(|| format!("failed to read file: {}", lock_f.path().display()))?;

    let v: EncodableResolve = toml::from_str(&lock_s)?;
    gctx.shell().print_json(&v)?;

    Ok(())
}
