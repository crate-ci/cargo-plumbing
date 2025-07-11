use cargo_test_macro::cargo_test;
use cargo_test_support::registry::Package;
use cargo_test_support::{project, str};
use snapbox::IntoData;

use crate::ProjectExt;

#[cargo_test]
fn package_with_deps() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-lockfile-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
                b = "1.0.0"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837",
        "dependencies": null,
        "name": "b",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a",
          "b"
        ],
        "name": "read-lockfile-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_transitive_deps() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").dep("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-lockfile-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
                b = "1.0.0"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": "ee3b274199c39817bfb6018e6cbe07ca43dd18241c42400d800e2545b77fb23b",
        "dependencies": [
          "a"
        ],
        "name": "b",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a",
          "b"
        ],
        "name": "read-lockfile-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_path_deps() {
    let p = project()
        .file("a/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-lockfile-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = { path = "./a" }
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a"
        ],
        "name": "read-lockfile-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_varying_deps_version() {
    Package::new("a", "1.0.0").publish();
    Package::new("a", "2.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-lockfile-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a1 = { version = "1.0.0", package = "a" }
                a2 = { version = "2.0.0", package = "a" }
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": "50bc2065af6476063cea5b8d28dc20df4c7ad146759b4712b5e86a6d25d74ddc",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "2.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a 1.0.0",
          "a 2.0.0"
        ],
        "name": "read-lockfile-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn workspace_member_with_inherited_deps() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1"]

                [workspace.dependencies]
                a = "1.0.0"
                b = "1.0.0"
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a.workspace = true
                b.workspace = true
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile")
        .cwd(p.root().join("crate1"))
        .run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837",
        "dependencies": null,
        "name": "b",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a",
          "b"
        ],
        "name": "crate1",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn workspace_package_depend_on_workspace_member() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1"]

                [package]
                name = "read-lockfile-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                crate1 = { path = "./crate1" }
                a = "1.0.0"
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": null,
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": null,
        "name": "crate1",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a",
          "crate1"
        ],
        "name": "read-lockfile-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}
