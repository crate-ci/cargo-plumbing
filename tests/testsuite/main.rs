mod cargo_plumbing_locate_manifest;
mod locate_manifest;

use cargo_test_support::{execs, process, ArgLineCommandExt, Execs, Project};

pub fn cargo_plumbing_exe() -> std::path::PathBuf {
    snapbox::cmd::cargo_bin("cargo-plumbing")
}

pub trait ProjectExt {
    fn cargo_plumbing(&self, cmd: &str) -> Execs;
}

impl ProjectExt for Project {
    fn cargo_plumbing(&self, cmd: &str) -> Execs {
        let cargo_plumbing = cargo_plumbing_exe();

        let mut p = process(&cargo_plumbing);
        p.cwd(self.root()).arg_line(cmd);

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
