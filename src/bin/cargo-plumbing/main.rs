use cargo::{core::Shell, GlobalContext};
use clap::Parser as _;

mod cli;
mod plumbing;

fn main() {
    let args = cli::Command::parse();

    let gctx = match GlobalContext::default() {
        Ok(gctx) => gctx,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell);
        }
    };

    if let Err(e) = args.exec(&gctx) {
        cargo::exit_with_error(e.into(), &mut gctx.shell());
    }
}
