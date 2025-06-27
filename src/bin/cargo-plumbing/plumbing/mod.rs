use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

pub(crate) mod read_manifest;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    ReadManifest(read_manifest::Args),
}

impl Plumbing {
    pub(crate) fn exec(self, gctx: &GlobalContext) -> CargoResult<()> {
        match self {
            Self::ReadManifest(args) => read_manifest::exec(gctx, args),
        }
    }
}
