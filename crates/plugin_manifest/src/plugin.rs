use rspack_core::{
  Plugin,PluginContext, PluginProcessAssetsOutput,ProcessAssetsArgs,
  CompilationAsset,
  rspack_sources::{RawSource, SourceExt},
};
// use rspack_error::Result;

#[derive(Debug)]
pub struct ManifestPlugin;

impl ManifestPlugin {
  pub fn new() -> Self {
    Self {}
  }
}

fn default_cotent() -> &'static str {
  r#"{"assets": []}"#
}

#[async_trait::async_trait]
impl Plugin for ManifestPlugin {
  fn name(&self) -> &'static str {
    "ManifestPlugin"
  }

  async fn process_assets_stage_additional(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  )-> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let entry_points = &compilation.entrypoints;
    println!("entry_points: {:?}", entry_points);
    Ok(())
  }

  async fn process_assets_stage_optimize_inline(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;

    let output_path = compilation
      .options
      .output
      .path
    .join("manifest.json".to_string()).to_string_lossy().to_string();
    println!("output_path: {}", output_path);
    compilation.emit_asset(
      output_path,
      CompilationAsset::from(RawSource::from(default_cotent().to_owned()).boxed()),
    );

    Ok(())
  }
}
