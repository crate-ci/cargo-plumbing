use cargo::GlobalContext;
use cargo_plumbing::CargoResult;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(bin_name = "cargo")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Command {
    #[command(subcommand)]
    Plumbing(crate::plumbing::Plumbing),
}

impl Command {
    pub(crate) fn exec(self, gctx: &GlobalContext) -> CargoResult<()> {
        match self {
            Self::Plumbing(plumbing) => plumbing.exec(gctx),
        }
    }
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Command::command().debug_assert();
}
