use cargo_test_macro::cargo_test;
use cargo_test_support::{main_file, project, str};
use cargo_test_support::registry::Package;
use snapbox::IntoData;

use crate::ProjectExt;

#[cargo_test]
fn simple() {
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
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(str![[r#"
{"badges":null,"bench":[],"bin":[{"bench":null,"crate-type":null,"crate_type":null,"doc":null,"doc-scrape-examples":null,"doctest":null,"edition":null,"filename":null,"harness":null,"name":"read-manifest-test","path":"src/main.rs","proc-macro":null,"proc_macro":null,"required-features":null,"test":null}],"build-dependencies":null,"build_dependencies":null,"cargo-features":null,"dependencies":{"a":"1.0.0"},"dev-dependencies":null,"dev_dependencies":null,"example":[],"features":null,"lib":null,"lints":null,"package":{"authors":[],"autobenches":false,"autobins":false,"autoexamples":false,"autolib":false,"autotests":false,"build":false,"categories":null,"default-run":null,"default-target":null,"description":null,"documentation":null,"edition":"2024","exclude":null,"forced-target":null,"homepage":null,"im-a-teapot":null,"include":null,"keywords":null,"license":null,"license-file":null,"links":null,"metabuild":null,"metadata":null,"name":"read-manifest-test","publish":null,"readme":false,"repository":null,"resolver":null,"rust-version":null,"version":"0.1.0","workspace":null},"patch":null,"profile":null,"project":null,"replace":null,"target":null,"test":[],"workspace":null}

"#]]
            .is_jsonlines(),
        )
        .with_status(0)
        .run();
}

#[cargo_test]
fn workspace_package() {
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
        .arg(p.root().join("Cargo.toml"))
        .with_stdout_data(str![[r#"
{"badges":null,"bench":[],"bin":[{"bench":null,"crate-type":null,"crate_type":null,"doc":null,"doc-scrape-examples":null,"doctest":null,"edition":null,"filename":null,"harness":null,"name":"read-manifest-test","path":"src/main.rs","proc-macro":null,"proc_macro":null,"required-features":null,"test":null}],"build-dependencies":null,"build_dependencies":null,"cargo-features":null,"dependencies":{"a":{"artifact":null,"base":null,"branch":null,"default-features":null,"default_features":null,"features":null,"git":null,"lib":null,"optional":null,"package":null,"path":null,"public":null,"registry":null,"registry-index":null,"rev":null,"tag":null,"target":null,"version":"1.0.0"}},"dev-dependencies":null,"dev_dependencies":null,"example":[],"features":null,"lib":null,"lints":null,"package":{"authors":[],"autobenches":false,"autobins":false,"autoexamples":false,"autolib":false,"autotests":false,"build":false,"categories":null,"default-run":null,"default-target":null,"description":null,"documentation":null,"edition":"2024","exclude":null,"forced-target":null,"homepage":null,"im-a-teapot":null,"include":null,"keywords":null,"license":null,"license-file":null,"links":null,"metabuild":null,"metadata":null,"name":"read-manifest-test","publish":null,"readme":false,"repository":null,"resolver":null,"rust-version":null,"version":"0.1.0","workspace":null},"patch":null,"profile":null,"project":null,"replace":null,"target":null,"test":[],"workspace":{"default-members":null,"dependencies":{"a":"1.0.0"},"exclude":null,"lints":null,"members":null,"metadata":null,"package":null,"resolver":"3"}}

"#]]
            .is_jsonlines(),
        )
        .with_status(0)
        .run();
}
