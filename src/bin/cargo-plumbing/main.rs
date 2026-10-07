use cargo::{GlobalContext, core::Shell};
use clap::Parser as _;

mod cli;
mod plumbing;

fn main() {
    let args = cli::Command::parse();

    let mut gctx = match GlobalContext::default() {
        Ok(gctx) => gctx,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell);
        }
    };

    if let Err(e) = args.exec(&mut gctx) {
        cargo::exit_with_error(e.into(), &mut gctx.shell());
    }
}
