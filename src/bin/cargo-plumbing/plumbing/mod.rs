use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

pub(crate) mod locate_project;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    /// Locate the root manifest file
    #[command()]
    LocateProject(locate_project::Args),
}

impl Plumbing {
    pub(crate) fn exec(self, gctx: &GlobalContext) -> CargoResult<()> {
        match self {
            Self::LocateProject(args) => locate_project::exec(gctx, args),
        }
    }
}
