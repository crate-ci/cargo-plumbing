automod::dir!("tests/testsuite");

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
