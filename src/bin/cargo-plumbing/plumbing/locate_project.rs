use std::fmt::Debug;

use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    anyhow::bail!("not implemented")
}
