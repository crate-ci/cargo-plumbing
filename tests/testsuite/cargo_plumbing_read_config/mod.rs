use super::super::ProjectExt;
use cargo_test_support::Project;

#[test]
fn read_config_basic() {
    let p = Project::from_template(
        snapbox::file!["../../../crates/cargo-plumbing-schemas/fixtures/default"],
    );

    p.cargo_plumbing("plumbing read-config --manifest-path Cargo.toml")
        .run();
}
