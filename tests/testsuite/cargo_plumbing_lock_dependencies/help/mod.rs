use cargo_test_support::file;
use cargo_test_support::prelude::*;

use crate::CargoCommandExt;

#[cargo_test]
fn case() {
    snapbox::cmd::Command::cargo_ui()
        .arg("plumbing")
        .arg("lock-dependencies")
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(file!["stdout.term.svg"])
        .stderr_eq(file!["stderr.term.svg"]);
}
