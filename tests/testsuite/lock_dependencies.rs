use cargo_test_support::git;
use cargo_test_support::prelude::*;
use cargo_test_support::project;
use cargo_test_support::registry::Package;
use cargo_test_support::str;

use crate::ProjectExt;

#[cargo_test]
fn package_with_no_deps() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data("")
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "read-manifest-test",
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
fn package_with_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index

"#]])
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "8ade940ad37d44e5895dd7dd5d1cd392bd0007a59a067b0ee1f4f114eec7ff41",
        "dependencies": [],
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a"
        ],
        "name": "read-manifest-test",
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
        .file("b/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                b = { path = "./b" }
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data("")
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "b",
        "replace": null,
        "source": null,
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "b"
        ],
        "name": "read-manifest-test",
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
fn package_with_git_deps() {
    let git_dep = git::new("b", |p| {
        p.file("src/lib.rs", "").file(
            "Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
    });

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "read-manifest-test"
                    version = "0.1.0"
                    authors = []
                    edition = "2024"

                    [dependencies]
                    b = {{ git = '{}' }}
                "#,
                git_dep.url()
            ),
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data(str![[r#"
[UPDATING] git repository `[ROOTURL]/b`

"#]])
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "b",
        "replace": null,
        "source": "git+[ROOTURL]/b#[..]",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "b"
        ],
        "name": "read-manifest-test",
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
