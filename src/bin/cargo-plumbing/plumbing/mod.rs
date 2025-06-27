use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

pub(crate) mod locate_manifest;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    /// Locate the root manifest file
    #[command()]
    LocateManifest(locate_manifest::Args),
}

impl Plumbing {
    pub(crate) fn exec(self, gctx: &GlobalContext) -> CargoResult<()> {
        match self {
            Self::LocateManifest(args) => locate_manifest::exec(gctx, args),
        }
    }
}
