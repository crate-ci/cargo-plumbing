use cargo::{CargoResult, GlobalContext};

pub(crate) mod locate_manifest;
pub(crate) mod lock_dependencies;
pub(crate) mod read_lockfile;
pub(crate) mod read_manifest;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    /// Locate the root manifest file
    #[command()]
    LocateManifest(locate_manifest::Args),
    /// Read the lockfile
    #[command()]
    ReadLockfile(read_lockfile::Args),
    /// Read the manifest file
    #[command()]
    ReadManifest(read_manifest::Args),
    /// Lock the dependencies
    #[command()]
    LockDependencies(lock_dependencies::Args),
}

impl Plumbing {
    pub(crate) fn exec(self, gctx: &mut GlobalContext) -> CargoResult<()> {
        match self {
            Self::LocateManifest(args) => locate_manifest::exec(gctx, args),
            Self::ReadLockfile(args) => read_lockfile::exec(gctx, args),
            Self::ReadManifest(args) => read_manifest::exec(gctx, args),
            Self::LockDependencies(args) => lock_dependencies::exec(gctx, args),
        }
    }
}
