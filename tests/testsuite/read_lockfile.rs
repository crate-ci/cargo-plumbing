use cargo_test_macro::cargo_test;
use cargo_test_support::registry::{Package, RegistryBuilder};
use cargo_test_support::{basic_manifest, git, project, str};
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837"
        },
        {
          "id": "read-lockfile-test@0.1.0",
          "source": null,
          "dependencies": [
            "a",
            "b"
          ]
        }
      ]
    }
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "ee3b274199c39817bfb6018e6cbe07ca43dd18241c42400d800e2545b77fb23b",
          "dependencies": [
            "a"
          ]
        },
        {
          "id": "read-lockfile-test@0.1.0",
          "source": null,
          "dependencies": [
            "a",
            "b"
          ]
        }
      ]
    }
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "a@0.1.0",
          "source": null
        },
        {
          "id": "read-lockfile-test@0.1.0",
          "source": null,
          "dependencies": [
            "a"
          ]
        }
      ]
    }
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_varying_deps_sources() {
    Package::new("a", "1.0.0").publish();
    Package::new("a", "2.0.0").publish();

    let _registry = RegistryBuilder::new().http_index().alternative().build();
    Package::new("a", "1.0.0").alternative(true).publish();

    let ref_1 = "v1.0.0";
    let ref_2 = "v2.0.0";
    let url = {
        let (p, r) = git::new_repo("my-git-repo", |p| {
            p.file("Cargo.toml", &basic_manifest("a", "1.0.0"))
                .file("src/lib.rs", "")
        });

        git::tag(&r, ref_1);

        p.change_file("Cargo.toml", &basic_manifest("a", "2.0.0"));
        git::add(&r);
        git::commit(&r);
        git::tag(&r, ref_2);

        p.url()
    };

    let p = project()
        .file("a/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "read-lockfile-test"
                    version = "0.1.0"
                    authors = []
                    edition = "2024"

                    [dependencies]
                    a  = {{ path = "./a" }}
                    a1 = {{ package = "a", version = "1.0.0" }}
                    a2 = {{ package = "a", version = "2.0.0" }}
                    a3 = {{ package = "a", git = "{url}", rev = "{ref_1}" }}
                    a4 = {{ package = "a", git = "{url}", rev = "{ref_2}" }}
                    a5 = {{ package = "a", git = "{url}" }}
                    a6 = {{ package = "a", version = "1.0.0", registry = "alternative" }}
                "#,
            ),
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "a@1.0.0",
          "source": null
        },
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "sparse+http://127.0.0.1:[..]/index/#a@1.0.0",
          "source": "sparse+http://127.0.0.1:[..]/index/",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
          "source": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#[..]"
        },
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@2.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "50bc2065af6476063cea5b8d28dc20df4c7ad146759b4712b5e86a6d25d74ddc"
        },
        {
          "id": "git+[ROOTURL]/my-git-repo?rev=v2.0.0#a@2.0.0",
          "source": "git+[ROOTURL]/my-git-repo?rev=v2.0.0#[..]"
        },
        {
          "id": "git+[ROOTURL]/my-git-repo#a@2.0.0",
          "source": "git+[ROOTURL]/my-git-repo#[..]"
        },
        {
          "id": "read-lockfile-test@0.1.0",
          "source": null,
          "dependencies": [
            "a@1.0.0",
            "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
            "sparse+http://127.0.0.1:[..]/index/#a@1.0.0",
            "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
            "registry+https://github.com/rust-lang/crates.io-index#a@2.0.0",
            "git+[ROOTURL]/my-git-repo?rev=v2.0.0#a@2.0.0",
            "git+[ROOTURL]/my-git-repo#a@2.0.0"
          ]
        }
      ]
    }
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837"
        },
        {
          "id": "crate1@0.1.0",
          "source": null,
          "dependencies": [
            "a",
            "b"
          ]
        }
      ]
    }
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
    "reason": "lockfile",
    "lockfile": {
      "package": [
        {
          "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
        },
        {
          "id": "crate1@0.1.0",
          "source": null
        },
        {
          "id": "read-lockfile-test@0.1.0",
          "source": null,
          "dependencies": [
            "a",
            "crate1"
          ]
        }
      ]
    }
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn bad_lockfile_invalid_url_in_source_id() {
    let p = project()
        .file(
            "Cargo.lock",
            r#"
                version = 4

                [[package]]
                name = "a"
                version = "1.0.0"
                # the source is missing a `:`
                source = "registry+https//github.com/rust-lang/crates.io-index"
                checksum = "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"

                [[package]]
                name = "read-lockfile-test"
                version = "0.1.0"
                dependencies = [
                 "a"
                ]
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] TOML parse error at line 8, column 26
  |
8 |                 source = "registry+https//github.com/rust-lang/crates.io-index"
  |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
invalid url `https//github.com/rust-lang/crates.io-index`: relative URL without a base


"#]])
        .run();
}

#[cargo_test]
fn bad_lockfile_unsupported_source_protocol() {
    let p = project()
        .file(
            "Cargo.lock",
            r#"
                version = 4

                [[package]]
                name = "a"
                version = "1.0.0"
                # `invalid` is an invalid protocol
                source = "invalid+https://github.com/rust-lang/crates.io-index"
                checksum = "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"

                [[package]]
                name = "read-lockfile-test"
                version = "0.1.0"
                dependencies = [
                 "a"
                ]
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] TOML parse error at line 8, column 26
  |
8 |                 source = "invalid+https://github.com/rust-lang/crates.io-index"
  |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
unsupported source protocol: invalid


"#]])
        .run();
}

#[cargo_test]
fn bad_lockfile_invalid_semver() {
    let p = project()
        .file(
            "Cargo.lock",
            r#"
                version = 4

                [[package]]
                name = "a"
                version = "1.0.0.0.0.0.0.0.0.0.0"
                source = "registry+https://github.com/rust-lang/crates.io-index"
                checksum = "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"

                [[package]]
                name = "read-lockfile-test"
                version = "0.1.0"
                dependencies = [
                 "a"
                ]
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] expected a version like "1.32"

"#]])
        .run();
}
