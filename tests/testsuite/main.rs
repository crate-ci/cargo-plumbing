mod cargo_plumbing;
mod cargo_plumbing_locate_manifest;
mod cargo_plumbing_lock_dependencies;
mod cargo_plumbing_read_lockfile;
mod cargo_plumbing_read_manifest;
mod cargo_plumbing_resolve_features;
mod cargo_plumbing_write_lockfile;
mod locate_manifest;
mod lock_dependencies;
mod read_lockfile;
mod read_manifest;
mod resolve_features;
mod write_lockfile;

use cargo_test_support::{execs, process, ArgLineCommandExt, Execs, Project, TestEnvCommandExt};
use cargo_util::ProcessBuilder;

pub fn cargo_plumbing_exe() -> std::path::PathBuf {
    snapbox::cmd::cargo_bin!("cargo-plumbing").to_path_buf()
}

pub trait ProjectExt {
    /// Creates an `Execs` instance to run the `cargo-plumbing` binary
    fn cargo_plumbing(&self, cmd: &str) -> Execs;
    /// Creates an `Execs` instance to run the globally installed `cargo` command
    fn cargo_global(&self, cmd: &str) -> Execs;
}

impl ProjectExt for Project {
    fn cargo_plumbing(&self, cmd: &str) -> Execs {
        let cargo_plumbing = cargo_plumbing_exe();

        let mut p = process(&cargo_plumbing);
        p.cwd(self.root()).arg_line(cmd);

        execs().with_process_builder(p)
    }

    fn cargo_global(&self, cmd: &str) -> Execs {
        let cargo = std::env::var_os("CARGO").unwrap_or("cargo".into());

        let mut p = ProcessBuilder::new(cargo);
        p.test_env().cwd(self.root()).arg_line(cmd);

        execs().with_process_builder(p)
    }
}

pub trait CargoCommandExt {
    fn cargo_ui() -> Self;
}

impl CargoCommandExt for snapbox::cmd::Command {
    fn cargo_ui() -> Self {
        use cargo_test_support::TestEnvCommandExt;
        Self::new(cargo_plumbing_exe())
            .with_assert(cargo_test_support::compare::assert_ui())
            .env("CARGO_TERM_COLOR", "always")
            .env("CARGO_TERM_HYPERLINKS", "true")
            .test_env()
    }
}

#[track_caller]
pub fn assert_exists(path: &std::path::Path) {
    assert!(
        path.exists(),
        "Expected `{}` to exist but was not found.",
        path.display()
    );
}

#[track_caller]
pub fn assert_not_exists(path: &std::path::Path) {
    assert!(
        !path.exists(),
        "Expected `{}` to NOT exist but was found.",
        path.display()
    );
}
