use std::fs;
use std::process::Command;
use std::path::PathBuf;
use rspack_testing::{ fixture, test_fixture_insta };
use plugin_specilize_module_name::SpecilizeModuleNamePlugin;

#[fixture("tests/fixtures/basic")]
fn test_rspack_hook_invoke(fixture_path: PathBuf) {
    let _ = Command::new("npm")
        .args(&["install"])
        .current_dir(fixture_path.clone())
        .output()
        .expect("npm install failed");

    test_fixture_insta(
        &fixture_path,
        &(|_| false),
        Box::new(|plugins, _| {
            plugins.push(Box::new(SpecilizeModuleNamePlugin::new(Some(vec!["lodash", "utils"]))))
        })
    );

    let target_asset_path = fixture_path.join("./dist/main.js");

    assert!(target_asset_path.exists());

    let file_contents = fs::read_to_string(target_asset_path).expect("Failed to read file");

    assert!(file_contents.contains("utils.js?id="));
    assert!(file_contents.contains("lodash.js?id="));
}
