use std::path::PathBuf;
use swc_core::{
    ecma::{
        transforms::{
            testing::{
                test_fixture,
                FixtureTestConfig,
            }
        }
    }
};
use swc_plugin_change_package_import::{
    config::Config,
    change_package_import::ModuleImportVisitor
};

#[testing::fixture("tests/fixture/1/input.js")]
fn test_it(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|t| {
            ModuleImportVisitor::new(vec![Config::LiteralConfig(String::from("y"))])
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}