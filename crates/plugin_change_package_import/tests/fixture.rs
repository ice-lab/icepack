use std::path::PathBuf;
use swc_core::ecma::{
    visit::as_folder,
    transforms::testing::{FixtureTestConfig, test_fixture},
};
use swc_plugin_change_package_import::{
    ModuleImportVisitor,
    Config,
};

#[testing::fixture("tests/fixture/1/input.js")]
fn test_it(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|t| {
            as_folder(ModuleImportVisitor::new(vec![Config::LiteralConfig(String::from("y"))]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}