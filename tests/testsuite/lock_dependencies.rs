use cargo_test_support::basic_manifest;
use cargo_test_support::compare::assert_e2e;
use cargo_test_support::git;
use cargo_test_support::prelude::*;
use cargo_test_support::project;
use cargo_test_support::registry::Package;
use cargo_test_support::registry::RegistryBuilder;
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
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "read-manifest-test@0.1.0"
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
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "8ade940ad37d44e5895dd7dd5d1cd392bd0007a59a067b0ee1f4f114eec7ff41"
  },
  {
    "reason": "locked-package",
    "id": "read-manifest-test@0.1.0",
    "dependencies": [
      "a"
    ]
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
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "b@1.0.0"
  },
  {
    "reason": "locked-package",
    "id": "read-manifest-test@0.1.0",
    "dependencies": [
      "b"
    ]
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
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "git+[ROOTURL]/b#1.0.0",
    "rev": "[..]"
  },
  {
    "reason": "locked-package",
    "id": "read-manifest-test@0.1.0",
    "dependencies": [
      "b"
    ]
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
                    name = "lock-dependencies-test"
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

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "a@1.0.0"
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "sparse+http://127.0.0.1:[..]/index/#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
    "rev": "[..]"
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@2.0.0",
    "checksum": "50bc2065af6476063cea5b8d28dc20df4c7ad146759b4712b5e86a6d25d74ddc"
  },
  {
    "reason": "locked-package",
    "id": "git+[ROOTURL]/my-git-repo?rev=v2.0.0#a@2.0.0",
    "rev": "[..]"
  },
  {
    "reason": "locked-package",
    "id": "git+[ROOTURL]/my-git-repo#a@2.0.0",
    "rev": "[..]"
  },
  {
    "reason": "locked-package",
    "id": "lock-dependencies-test@0.1.0",
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
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_unused_patches() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("a/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "2.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "lock-dependencies-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"

                [patch.crates-io]
                a = { path = "a" }
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "lock-dependencies-test@0.1.0",
    "dependencies": [
      "a"
    ]
  },
  {
    "reason": "unused-patches",
    "unused": [
      {
        "id": "a@2.0.0"
      }
    ]
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

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837"
  },
  {
    "reason": "locked-package",
    "id": "crate1@0.1.0",
    "dependencies": [
      "a",
      "b"
    ]
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
                name = "lock-dependencies-test"
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

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "crate1@0.1.0"
  },
  {
    "reason": "locked-package",
    "id": "lock-dependencies-test@0.1.0",
    "dependencies": [
      "a",
      "crate1"
    ]
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn lock_dependencies_conservatively_using_previous_lock() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "lock-dependencies-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
            "#,
        )
        .build();
    let manifest_path = p.root().join("Cargo.toml");
    let lockfile_path = p.root().join("Cargo.lock");

    p.cargo_global("check").run();

    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(&lockfile_path)
        .run();
    let previous_lock = String::from_utf8(out.stdout).unwrap();

    assert_e2e().eq(
        &previous_lock,
        str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "locked-package"
  },
  {
    "dependencies": [
      "a"
    ],
    "id": "lock-dependencies-test@0.1.0",
    "reason": "locked-package"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
    );

    Package::new("a", "1.0.1").publish();

    let out = p
        .cargo_plumbing("plumbing lock-dependencies")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .with_stdin(previous_lock)
        .run();
    let latest_lock = String::from_utf8(out.stdout).unwrap();

    assert_e2e().eq(
        latest_lock,
        str![[r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "locked-package"
  },
  {
    "dependencies": [
      "a"
    ],
    "id": "lock-dependencies-test@0.1.0",
    "reason": "locked-package"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
    );
}

#[cargo_test]
fn lock_dependencies_with_git_deps_with_previous_lockfile() {
    let (git_p, git_r) = git::new_repo("my-git-repo", |p| {
        p.file("Cargo.toml", &basic_manifest("a", "1.0.0"))
            .file("src/lib.rs", "")
    });

    let ref_1 = "v1.0.0";
    git::tag(&git_r, ref_1);

    let branch_name = "master";
    let url = git_p.url();

    let locked_commit_hash = git_r.head().unwrap().target().unwrap().to_string();

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
                    a1 = {{ package = "a", git = "{url}", rev = "{ref_1}" }}
                    a2 = {{ package = "a", git = "{url}" }}
                    a3 = {{ package = "a", git = "{url}", branch = "{branch_name}" }}
                "#,
            ),
        )
        .build();
    let manifest_path = p.root().join("Cargo.toml");
    let lockfile_path = p.root().join("Cargo.lock");

    p.cargo_global("check").run();

    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(&lockfile_path)
        .run();
    let previous_lock_result = String::from_utf8(out.stdout).unwrap();

    let previous_lock_value = r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?branch=master#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "dependencies": [
      "git+[ROOTURL]/my-git-repo?branch=master#a@1.0.0",
      "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
      "git+[ROOTURL]/my-git-repo#a@1.0.0"
    ],
    "id": "read-lockfile-test@0.1.0",
    "reason": "locked-package"
  }
]
"#
    .replace("REV", &locked_commit_hash);

    assert_e2e().eq(
        &previous_lock_result,
        previous_lock_value.is_json().against_jsonlines(),
    );

    git_p.change_file("src/lib.rs", "# simulate change");
    git::commit(&git_r);

    let out = p
        .cargo_plumbing("plumbing lock-dependencies")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .with_stdin(previous_lock_result)
        .run();
    let latest_lock_result = String::from_utf8(out.stdout).unwrap();

    let latest_lock_value = r#"
[
  {
    "reason": "lockfile",
    "version": 4
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?branch=master#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo#a@1.0.0",
    "reason": "locked-package",
    "rev": "REV"
  },
  {
    "dependencies": [
      "git+[ROOTURL]/my-git-repo?branch=master#a@1.0.0",
      "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
      "git+[ROOTURL]/my-git-repo#a@1.0.0"
    ],
    "id": "read-lockfile-test@0.1.0",
    "reason": "locked-package"
  }
]
"#
    .replace("REV", &locked_commit_hash);

    assert_e2e().eq(
        latest_lock_result,
        latest_lock_value.is_json().against_jsonlines(),
    );
}

#[cargo_test]
fn lock_dependencies_conservatively_using_previous_lock_with_old_lockfile_version() {
    let cksum = Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "lock-dependencies-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
            "#,
        )
        .file(
            "Cargo.lock",
            &format!(
                r#"
                    [root]
                    name = "lock-dependencies-test"
                    version = "0.1.0"
                    dependencies = [
                     "a 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
                    ]

                    [[package]]
                    name = "a"
                    version = "1.0.0"
                    source = "registry+https://github.com/rust-lang/crates.io-index"

                    [metadata]
                    "checksum a 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)" = "{cksum}"
                "#,
            ),
        )
        .build();
    let manifest_path = p.root().join("Cargo.toml");
    let lockfile_path = p.root().join("Cargo.lock");

    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(&lockfile_path)
        .run();
    let previous_lock = String::from_utf8(out.stdout).unwrap();

    assert_e2e().eq(
        &previous_lock,
        str![[r#"
[
  {
    "reason": "lockfile",
    "version": 1
  },
  {
    "reason": "locked-package",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454"
  },
  {
    "reason": "locked-package",
    "id": "lock-dependencies-test@0.1.0",
    "dependencies": [
      "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0"
    ]
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
    );

    Package::new("a", "1.0.1").publish();

    let out = p
        .cargo_plumbing("plumbing lock-dependencies")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .with_stdin(previous_lock)
        .run();
    let latest_lock = String::from_utf8(out.stdout).unwrap();

    assert_e2e().eq(
        latest_lock,
        str![[r#"
[
  {
    "reason": "lockfile",
    "version": null
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "locked-package"
  },
  {
    "dependencies": [
      "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0"
    ],
    "id": "lock-dependencies-test@0.1.0",
    "reason": "locked-package"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
    );
}
