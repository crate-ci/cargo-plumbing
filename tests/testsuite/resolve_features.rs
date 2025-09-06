use cargo_plumbing_schemas::read_lockfile::ReadLockfileOut;
use cargo_plumbing_schemas::read_manifest::ReadManifestOut;
use cargo_test_macro::cargo_test;
use cargo_test_support::registry::{Dependency, Package, RegistryBuilder};
use cargo_test_support::{basic_manifest, cross_compile, git, project, str};
use snapbox::IntoData;

use crate::ProjectExt;

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

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#read-lockfile-test@0.1.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo/a#0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
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

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "git+[ROOTURL]/my-git-repo#a@2.0.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo/a#1.0.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@2.0.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo#read-lockfile-test@0.1.0",
    "reason": "activated"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?rev=v2.0.0#a@2.0.0",
    "reason": "activated"
  },
  {
    "id": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#a@1.0.0",
    "reason": "activated"
  },
  {
    "reason": "activated",
    "id": "sparse+http://127.0.0.1:[..]/index/#a@1.0.0"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_features() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [features]
                default = ["feat-b"]
                feat-a = ["dep:a"]
                feat-b = []
                feat-c = ["feat-b"]

                [dependencies]
                a = { version = "1.0.0", optional = true }
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "default",
      "feat-b"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--all-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "default",
      "feat-a",
      "feat-b",
      "feat-c"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--no-default-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--no-default-features")
        .arg("-F")
        .arg("feat-a")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "feat-a"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("-F")
        .arg("feat-a")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "default",
      "feat-a",
      "feat-b"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_optional_dep_without_features() {
    Package::new("b", "1.0.0").publish();
    Package::new("a", "1.0.0")
        .add_dep(Dependency::new("b", "1.0.0").optional(true))
        .feature("a-has-b", &["dep:b"])
        .publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = { version = "1.0.0", optional = true }
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--features")
        .arg("a")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--features")
        .arg("a/a-has-b")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "features": [
      "a-has-b"
    ],
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_proc_macro_deps_features() {
    Package::new("b", "1.0.0").publish();
    Package::new("a", "1.0.0")
        .add_dep(Dependency::new("b", "1.0.0").optional(true))
        .feature("a-has-b", &["dep:b"])
        .proc_macro(true)
        .publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [features]
                default = ["feat-a"]
                feat-a = ["dep:a", "a/a-has-b"]

                [dependencies]
                a = { version = "1.0.0", optional = true }
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features_for": "host",
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "a-has-b"
    ],
    "features_for": "host",
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "default",
      "feat-a"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_target_specific_dep() {
    let target = cross_compile::alternate();
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "resolve-features-tests"
                    version = "0.1.0"
                    authors = []
                    edition = "2024"

                    [target.{target}.dependencies]
                    a = {{ version = "1.0.0" }}
                "#,
            ),
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--target")
        .arg(target)
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn workspace_package_with_members_with_features() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["crate1"]

                [workspace.dependencies]
                crate1 = { path = "crate1" }

                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [features]
                default = ["c"]
                b = ["crate1/b"]
                c = ["crate1/c"]

                [dependencies]
                crate1 = { workspace = true, features = ["a"] }
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

                [features]
                a = []
                b = []
                c = []
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a",
      "c"
    ],
    "id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "activated"
  },
  {
    "features": [
      "c",
      "default"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--all-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "b",
      "c",
      "default"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "features": [
      "a",
      "b",
      "c"
    ],
    "id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--no-default-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a"
    ],
    "id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--no-default-features")
        .arg("-F")
        .arg("b")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a",
      "b"
    ],
    "id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "activated"
  },
  {
    "features": [
      "b"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("-F")
        .arg("b")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "a",
      "b",
      "c"
    ],
    "id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "activated"
  },
  {
    "features": [
      "b",
      "c",
      "default"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_dev_deps() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dev-dependencies]
                a = "1.0.0"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    // `--examples` activates dev dependencies
    p.cargo_plumbing("plumbing resolve-features")
        .arg("--examples")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn targets_with_required_features() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file("src/bin_a.rs", "fn main() -> () { () }")
        .file("src/bin_b.rs", "fn main() -> () { () }")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "resolve-features-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [features]
                feat-a = ["dep:a"]
                feat-b = ["dep:b"]

                [[bin]]
                name = "bin_a"
                path = "src/bin_a.rs"
                required-features = ["feat-a"]

                [[bin]]
                name = "bin_b"
                path = "src/bin_b.rs"
                required-features = ["feat-b"]

                [dependencies]
                a = { version = "1.0.0", optional = true }
                b = { version = "1.0.0", optional = true }
            "#,
        )
        .build();

    p.cargo_global("check").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    stdin.push('\n');
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    let out: String = ReadLockfileOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter(|msg| {
            matches!(
                msg,
                ReadLockfileOut::LockedPackage { .. } | ReadLockfileOut::UnusedPatches { .. }
            )
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .args(&["--bin", "bin_a"])
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "feat-a"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "kind": "bin",
    "name": "bin_a",
    "reason": "target"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .args(&["--bin", "bin_b"])
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "features": [
      "feat-b"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "reason": "activated"
  },
  {
    "kind": "bin",
    "name": "bin_b",
    "reason": "target"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();

    p.cargo_plumbing("plumbing resolve-features")
        .arg("--bins")
        .with_stdin(&stdin)
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "reason": "activated"
  },
  {
    "features": [
      "feat-a",
      "feat-b"
    ],
    "id": "path+[ROOTURL]/foo#resolve-features-tests@0.1.0",
    "reason": "activated"
  },
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "reason": "activated"
  },
  {
    "kind": "bin",
    "name": "bin_a",
    "reason": "target"
  },
  {
    "kind": "bin",
    "name": "bin_b",
    "reason": "target"
  }
]
"#]]
            .unordered()
            .is_json()
            .against_jsonlines(),
        )
        .run();
}
