use cargo_test_macro::cargo_test;
use cargo_test_support::registry::{Package, RegistryBuilder};
use cargo_test_support::{basic_manifest, git, project, str};
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

    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .exec_with_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdout)
        .with_status(0)
        .with_stdout_data(
            str!["[]"]
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

    let out = p
        .cargo_plumbing("plumbing read-lockfile")
        .arg("--lockfile-path")
        .arg(p.root().join("Cargo.lock"))
        .with_status(0)
        .exec_with_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();

    p.cargo_plumbing("plumbing resolve-features")
        .with_stdin(stdout)
        .with_status(0)
        .with_stdout_data(
            str!["[]"]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}
