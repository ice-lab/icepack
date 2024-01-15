use std::path::PathBuf;
use std::collections::HashMap;
use swc_core::ecma::{
    visit::as_folder,
    transforms::testing::{FixtureTestConfig, test_fixture},
};
use swc_plugin_change_package_import::{
    ModuleImportVisitor,
    Config,
    SpecificConfigs,
    MapProperty,
    ImportType,
};

#[testing::fixture("tests/fixture/single_literal_transform/input.js")]
fn test_single_literal_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![Config::LiteralConfig(String::from("y"))]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/multi_literal_transform/input.js")]
fn test_multi_literal_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::LiteralConfig(String::from("z")),
                Config::LiteralConfig(String::from("o")),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/single_specific_transform/input.js")]

fn test_single_specific_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("y"),
                        map: HashMap::from([
                            (
                                "x".to_string(),
                                MapProperty {
                                    to: String::from("m/n"),
                                    import_type: Some(ImportType::Named),
                                    name: Some(String::from("a")),
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/single_specific_transform_2/input.js")]
fn test_single_specific_transform_2(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("y"),
                        map: HashMap::from([
                            (
                                "x".to_string(),
                                MapProperty {
                                    to: String::from("m/n"),
                                    import_type: Some(ImportType::Named),
                                    name: Some(String::from("a")),
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/mix_specific_transform/input.js")]
fn test_mix_specific_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::LiteralConfig(String::from("antd")),
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "a".to_string(),
                                MapProperty {
                                    to: String::from("@ice/x/y"),
                                    import_type: None,
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}


#[testing::fixture("tests/fixture/multi_specific_transform/input.js")]
fn test_multi_specific_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("e"),
                        map: HashMap::from([
                            (
                                "a".to_string(),
                                MapProperty {
                                    to: String::from("@e/x"),
                                    import_type: Some(ImportType::Default),
                                    name: None,
                                }
                            ),
                            (
                                "b".to_string(),
                                MapProperty {
                                    to: String::from("e"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            )
                        ]),
                    },
                ),
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("k"),
                        map: HashMap::from([
                            (
                                "j".to_string(),
                                MapProperty {
                                    to: String::from("@e/k"),
                                    import_type: Some(ImportType::Named),
                                    name: Some(String::from("jj")),
                                }
                            ),
                        ]),
                    },
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_basic_transform/input.js")]
fn test_ice_basic_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "runApp".to_string(),
                                MapProperty {
                                    to: String::from("@ice/runtime"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_as_transform/input.js")]
fn test_ice_as_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "runApp".to_string(),
                                MapProperty {
                                    to: String::from("@ice/runtime"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_alias_transform/input.js")]
fn test_ice_alias_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "Head".to_string(),
                                MapProperty {
                                    to: String::from("react-helmet"),
                                    import_type: Some(ImportType::Default),
                                    name: Some("Helmet".to_string()),
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_alias_with_as_transform/input.js")]
fn test_ice_alias_with_as_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "Head".to_string(),
                                MapProperty {
                                    to: String::from("react-helmet"),
                                    import_type: Some(ImportType::Default),
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_multiple_transform/input.js")]
fn test_ice_multiple_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "request".to_string(),
                                MapProperty {
                                    to: String::from("axios"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                            (
                                "test".to_string(),
                                MapProperty {
                                    to: String::from("axios"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                            (
                                "store".to_string(),
                                MapProperty {
                                    to: String::from("@ice/store"),
                                    import_type: Some(ImportType::Default),
                                    name: None,
                                }
                            )
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_matched_transform/input.js")]
fn test_ice_matched_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "runApp".to_string(),
                                MapProperty {
                                    to: String::from("@ice/runtime"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/fixture/ice_miss_match_transform/input.js")]
fn test_ice_miss_match_transform(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Default::default(),
        &|_t| {
            as_folder(ModuleImportVisitor::new(vec![
                Config::SpecificConfig(
                    SpecificConfigs {
                        name: String::from("ice"),
                        map: HashMap::from([
                            (
                                "runApp".to_string(),
                                MapProperty {
                                    to: String::from("@ice/runtime"),
                                    import_type: Some(ImportType::Named),
                                    name: None,
                                }
                            ),
                        ]),
                    }
                ),
            ]))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}