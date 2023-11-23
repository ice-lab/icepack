use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use rspack_core::{
  Plugin,PluginContext, PluginProcessAssetsOutput,ProcessAssetsArgs,
  CompilationAsset, PublicPath,
  rspack_sources::{RawSource, SourceExt},
};
// use rspack_error::Result;

#[derive(Debug)]
pub struct ManifestPlugin;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetsManifest {
  pub pages: HashMap<String, Vec<String>>,
  pub entries: HashMap<String, Vec<String>>,
  pub public_path: String,
}

const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";

impl ManifestPlugin {
  pub fn new() -> Self {
    Self {}
  }
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
    let public_path = match &compilation.options.output.public_path {
      PublicPath::String(p) => p,
      PublicPath::Auto => AUTO_PUBLIC_PATH_PLACEHOLDER,
    };
    let mut assets_mainfest = AssetsManifest {
      pages: HashMap::new(),
      entries: HashMap::new(),
      public_path: public_path.to_string(),
    };
    let entry_points = &compilation.entrypoints;
    let assets = &compilation.assets();
    
    entry_points.iter().for_each(|(name, _entry)| {
      let mut files: Vec<String> = Vec::new();
      compilation.entrypoint_by_name(name).chunks.iter().for_each(|chunk| {
        
        if let Some(chunk) = compilation.chunk_by_ukey.get(chunk) {
          chunk.files.iter().for_each(|file| {
            if let Some(asset) = assets.get(file) {
              if !asset.info.hot_module_replacement && !asset.info.development {
                files.push(file.to_string());
              }
            } else {
              files.push(file.to_string());
            }
          });
        };
      });
      assets_mainfest.entries.insert(name.to_string(), files);
    });
    let page_chunk_name_regex = regex::Regex::new(r"^p_").unwrap();
    compilation
      .chunk_by_ukey
      .values()
      .for_each(|c| {
        if let Some(name) = &c.id {
          if !c.has_entry_module(&compilation.chunk_graph) && !c.can_be_initial(&compilation.chunk_group_by_ukey) {
            assets_mainfest.pages.insert(
              page_chunk_name_regex.replace(name, "").to_string(),
              Vec::from_iter(
                c.files
                  .iter()
                  // Only collect js and css files.
                  .filter(|f| f.ends_with(".js") || f.ends_with(".css"))
                  .cloned()
              ),
            );
          }
        }
      });
    let json_string = serde_json::to_string(&assets_mainfest).unwrap();
    let output_path = compilation
      .options
      .output
      .path
    .join("assets-manifest.json".to_string()).to_string_lossy().to_string();
    compilation.emit_asset(
      output_path,
      CompilationAsset::from(RawSource::from(json_string).boxed()),
    );
    Ok(())
  }
}
