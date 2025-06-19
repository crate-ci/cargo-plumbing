use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {}

impl Plumbing {
    pub(crate) fn exec(self, gctx: &GlobalContext) -> CargoResult<()> {
        anyhow::bail!("not implemented");
    }
}
