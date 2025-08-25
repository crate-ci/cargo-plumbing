use cargo_plumbing_schemas::read_manifest::ReadManifestOut;
use cargo_plumbing_schemas::resolve_features::ResolveFeaturesIn;
use cargo_test_support::prelude::*;
use cargo_test_support::registry::Package;
use cargo_test_support::*;

use crate::ProjectExt;

#[cargo_test]
fn package_with_lib_and_main() {
    let p = project()
        .file("src/lib.rs", "fn f() -> () { () }")
        .file("src/main.rs", "fn main() -> () { () }")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "plan-build-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_global("build").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter_map(|msg| match msg {
            ReadManifestOut::Manifest { pkg_id, .. } => {
                pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
            }
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());

    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#plan-build-test@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_test",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "plan_build_test",
        "index": 0
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-test@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "bin"
      ],
      "doctest": false,
      "edition": "2024",
      "kind": [
        "bin"
      ],
      "name": "plan-build-test",
      "src_path": "[ROOT]/foo/src/main.rs",
      "test": true
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
                name = "plan-build-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                b = "1.0.0"
            "#,
        )
        .build();

    p.cargo_global("build").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter_map(|msg| match msg {
            ReadManifestOut::Manifest { pkg_id, .. } => {
                pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
            }
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());

    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2015",
      "kind": [
        "lib"
      ],
      "name": "a",
      "src_path": "[ROOT]/home/.cargo/registry/src/-[HASH]/a-1.0.0/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "a",
        "index": 0
      }
    ],
    "id": "registry+https://github.com/rust-lang/crates.io-index#b@1.0.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2015",
      "kind": [
        "lib"
      ],
      "name": "b",
      "src_path": "[ROOT]/home/.cargo/registry/src/-[HASH]/b-1.0.0/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "b",
        "index": 1
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-test@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_test",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
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
        .file("crates/crate1/src/lib.rs", "")
        .file(
            "crates/crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("crates/crate2/src/lib.rs", "")
        .file(
            "crates/crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
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
                name = "plan-build-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                crate1.path = "crates/crate1"
                crate2.path = "crates/crate2"
            "#,
        )
        .build();

    p.cargo_global("build").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter_map(|msg| match msg {
            ReadManifestOut::Manifest { pkg_id, .. } => {
                pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
            }
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());

    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo/crates/crate1#0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "crate1",
      "src_path": "[ROOT]/foo/crates/crate1/src/lib.rs",
      "test": true
    }
  },
  {
    "id": "path+[ROOTURL]/foo/crates/crate2#0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "crate2",
      "src_path": "[ROOT]/foo/crates/crate2/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "crate1",
        "index": 0
      },
      {
        "extern_crate_name": "crate2",
        "index": 1
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-test@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_test",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
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
fn package_with_deps_with_features() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "plan-build-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [features]
                feat-a = ["dep:a"]

                [dependencies]
                a = { version = "1.0.0", optional = true }
            "#,
        )
        .build();

    p.cargo_global("build").run();

    let mut base_stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter_map(|msg| match msg {
            ReadManifestOut::Manifest { pkg_id, .. } => {
                pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
            }
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    base_stdin.push_str(&out);
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    base_stdin.push_str(&String::from_utf8(out.stdout).unwrap());

    let mut stdin = base_stdin.clone();
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#plan-build-tests@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_tests",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
    }
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();

    let mut stdin = base_stdin.clone();
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .arg("-F")
        .arg("feat-a")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .arg("-F")
        .arg("feat-a")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "path+[ROOTURL]/foo#plan-build-tests@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_tests",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
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
fn package_with_build_scripts() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "plan-build-tests"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [build-dependencies]
                a = "1.0.0"
            "#,
        )
        .file("build.rs", "fn main() { }")
        .build();

    p.cargo_global("build").run();

    let mut stdin = String::new();
    let out = p
        .cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_status(0)
        .run();
    let out: String = ReadManifestOut::parse_stream(&*out.stdout)
        .filter_map(Result::ok)
        .filter_map(|msg| match msg {
            ReadManifestOut::Manifest { pkg_id, .. } => {
                pkg_id.map(|id| ResolveFeaturesIn::Manifest { id })
            }
        })
        .map(|msg| serde_json::to_string(&msg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    stdin.push_str(&out);
    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());
    let out = p
        .cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdin.clone())
        .with_status(0)
        .run();
    stdin.push_str(&String::from_utf8(out.stdout).unwrap());

    p.cargo_plumbing("plumbing plan-build")
        .arg("--intent")
        .arg("build")
        .with_status(0)
        .with_stdin(stdin)
        .with_stdout_data(
            str![[r#"
[
  {
    "id": "registry+https://github.com/rust-lang/crates.io-index#a@1.0.0",
    "platform": "host",
    "reason": "unit",
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2015",
      "kind": [
        "lib"
      ],
      "name": "a",
      "src_path": "[ROOT]/home/.cargo/registry/src/-[HASH]/a-1.0.0/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "build_script_build",
        "index": 2
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-tests@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "root": true,
    "target": {
      "crate_types": [
        "lib"
      ],
      "doctest": true,
      "edition": "2024",
      "kind": [
        "lib"
      ],
      "name": "plan_build_tests",
      "src_path": "[ROOT]/foo/src/lib.rs",
      "test": true
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "build_script_build",
        "index": 3
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-tests@0.1.0",
    "platform": "[HOST_TARGET]",
    "reason": "unit",
    "target": {
      "crate_types": [
        "bin"
      ],
      "doctest": false,
      "edition": "2024",
      "kind": [
        "custom-build"
      ],
      "name": "build-script-build",
      "src_path": "[ROOT]/foo/build.rs",
      "test": false
    }
  },
  {
    "deps": [
      {
        "extern_crate_name": "a",
        "index": 0
      }
    ],
    "id": "path+[ROOTURL]/foo#plan-build-tests@0.1.0",
    "platform": "host",
    "reason": "unit",
    "target": {
      "crate_types": [
        "bin"
      ],
      "doctest": false,
      "edition": "2024",
      "kind": [
        "custom-build"
      ],
      "name": "build-script-build",
      "src_path": "[ROOT]/foo/build.rs",
      "test": false
    }
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}
