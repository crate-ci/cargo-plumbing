use cargo_test_macro::cargo_test;
use cargo_test_support::registry::Package;
use cargo_test_support::{main_file, project, str};
use snapbox::IntoData;

use crate::ProjectExt;

#[cargo_test]
fn package_with_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/main.rs", &main_file("Hello", &[]))
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

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": "1.0.0"
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": "1.0.0"
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn workspace_package_with_inherited_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"

                [workspace.dependencies]
                a = "1.0.0"

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a.workspace = true
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": null,
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": null,
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn workspace_package_with_members() {
    let p = project()
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
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
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": null,
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": null,
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate2",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate2",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate2/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate2#0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn virtual_workspace_with_members() {
    let p = project()
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
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
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": null,
      "bin": null,
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": null,
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": null,
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": null,
      "workspace": {
        "default-members": null,
        "dependencies": null,
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": null,
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": null,
      "bin": null,
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": null,
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": null,
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": null,
      "workspace": {
        "default-members": null,
        "dependencies": null,
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": null,
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate2",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate2",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate2/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate2#0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn workspace_member_with_inherited_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
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
            "#,
        )
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]

                [workspace.dependencies]
                a = "1.0.0"

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [
        {
          "bench": null,
          "crate-type": null,
          "crate_type": null,
          "doc": null,
          "doc-scrape-examples": null,
          "doctest": null,
          "edition": null,
          "filename": null,
          "harness": null,
          "name": "read-manifest-test",
          "path": "src/main.rs",
          "proc-macro": null,
          "proc_macro": null,
          "required-features": null,
          "test": null
        }
      ],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": [
          "crate1",
          "crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate2",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate2",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": null
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate2/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate2#0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn workspace_member_via_package_workspace_key() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file(
            "workspace-root/Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["../crate1", "../crate2"]

                [workspace.dependencies]
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
                workspace = "../workspace-root"

                [dependencies]
                a.workspace = true
            "#,
        )
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
                workspace = "../workspace-root"

                [dependencies]
                a.workspace = true
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": "../workspace-root"
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": null,
      "bin": null,
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": null,
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": null,
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": null,
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": [
          "../crate1",
          "../crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/workspace-root/Cargo.toml",
    "pkg_id": null,
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .arg("--workspace")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest": {
      "badges": null,
      "bench": null,
      "bin": null,
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": null,
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": null,
      "features": null,
      "hints": null,
      "lib": null,
      "lints": null,
      "package": null,
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": null,
      "workspace": {
        "default-members": null,
        "dependencies": {
          "a": "1.0.0"
        },
        "exclude": null,
        "lints": null,
        "members": [
          "../crate1",
          "../crate2"
        ],
        "metadata": null,
        "package": null,
        "resolver": "3"
      }
    },
    "path": "[ROOT]/foo/workspace-root/Cargo.toml",
    "pkg_id": null,
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate1",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": "../workspace-root"
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate1/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate1#0.1.0",
    "reason": "manifest"
  },
  {
    "manifest": {
      "badges": null,
      "bench": [],
      "bin": [],
      "build-dependencies": null,
      "build_dependencies": null,
      "cargo-features": null,
      "dependencies": {
        "a": {
          "artifact": null,
          "base": null,
          "branch": null,
          "default-features": null,
          "default_features": null,
          "features": null,
          "git": null,
          "lib": null,
          "optional": null,
          "package": null,
          "path": null,
          "public": null,
          "registry": null,
          "registry-index": null,
          "rev": null,
          "tag": null,
          "target": null,
          "version": "1.0.0"
        }
      },
      "dev-dependencies": null,
      "dev_dependencies": null,
      "example": [],
      "features": null,
      "hints": null,
      "lib": {
        "bench": null,
        "crate-type": null,
        "crate_type": null,
        "doc": null,
        "doc-scrape-examples": null,
        "doctest": null,
        "edition": null,
        "filename": null,
        "harness": null,
        "name": "crate2",
        "path": "src/lib.rs",
        "proc-macro": null,
        "proc_macro": null,
        "required-features": null,
        "test": null
      },
      "lints": null,
      "package": {
        "authors": [],
        "autobenches": false,
        "autobins": false,
        "autoexamples": false,
        "autolib": false,
        "autotests": false,
        "build": false,
        "categories": null,
        "default-run": null,
        "default-target": null,
        "description": null,
        "documentation": null,
        "edition": "2024",
        "exclude": null,
        "forced-target": null,
        "homepage": null,
        "im-a-teapot": null,
        "include": null,
        "keywords": null,
        "license": null,
        "license-file": null,
        "links": null,
        "metabuild": null,
        "metadata": null,
        "name": "crate2",
        "publish": null,
        "readme": false,
        "repository": null,
        "resolver": null,
        "rust-version": null,
        "version": "0.1.0",
        "workspace": "../workspace-root"
      },
      "patch": null,
      "profile": null,
      "project": null,
      "replace": null,
      "target": null,
      "test": [],
      "workspace": null
    },
    "path": "[ROOT]/foo/crate2/Cargo.toml",
    "pkg_id": "path+[ROOTURL]/foo/crate2#0.1.0",
    "reason": "manifest"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_status(0)
        .run();
}
