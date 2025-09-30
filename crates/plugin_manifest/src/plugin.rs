use std::{collections::HashMap, path::Path};

use rspack_core::{
  CompilationAsset, Plugin,
  PublicPath, Compilation,
  CompilationProcessAssets,
  ApplyContext,
};
use rspack_sources::{RawSource, SourceExt};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

#[plugin]
#[derive(Debug)]
pub struct ManifestPlugin;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetsManifest {
  pub pages: HashMap<String, Vec<String>>,
  pub entries: HashMap<String, Vec<String>>,
  pub assets: HashMap<String, String>,
  pub public_path: String,
  pub data_loader: Option<String>,
}

const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_CSS_AUTO_PUBLIC_PATH__";

impl Default for ManifestPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl ManifestPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilationProcessAssets for ManifestPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONS)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let public_path = match &compilation.options.output.public_path {
    PublicPath::Filename(p) => p.template(),
    PublicPath::Auto => Some(AUTO_PUBLIC_PATH_PLACEHOLDER),
  };
  let mut assets_manifest = AssetsManifest {
    pages: HashMap::new(),
    entries: HashMap::new(),
    assets: HashMap::new(),
    public_path: public_path.unwrap_or_default().to_string(),
    data_loader: None,
  };
  let entry_points = &compilation.entrypoints;
  let assets = &compilation.assets();

  assets.iter().for_each(|(_, asset)| {
    let version = &asset.info.version;
    let source_file = &asset.info.source_filename;
    if let Some(name) = source_file {
      assets_manifest
        .assets
        .insert(name.to_string(), version.to_string());
    }
  });
  
  entry_points.iter().for_each(|(name, _entry)| {
    let mut files: Vec<String> = Vec::new();
    let entrypoint = compilation.entrypoint_by_name(name);
    entrypoint
      .chunks
      .iter()
      .for_each(|chunk| {
        if let Some(chunk) = compilation.chunk_by_ukey.get(chunk) {
          chunk.files().iter().for_each(|file| {
            if let Some(asset) = assets.get(file) {
              if !asset.info.hot_module_replacement.unwrap_or(false) && !asset.info.development.unwrap_or(false) {
                files.push(file.to_string());
              }
            } else {
              files.push(file.to_string());
            }
          });
        }
      });
    assets_manifest.entries.insert(name.to_string(), files);
  });
  
  // Check .ice/data-loader.ts is exists
  let data_loader_file =
    Path::new(&compilation.options.context.as_str()).join(".ice/data-loader.ts");
  if data_loader_file.exists() {
    assets_manifest.data_loader = Some("js/data-loader.js".to_string());
  }

  let page_chunk_name_regex = regex::Regex::new(r"^p_").unwrap();
  compilation.chunk_by_ukey.values().for_each(|c| {
    if let Some(name) = c.name() {
      if !c.has_entry_module(&compilation.chunk_graph)
        && !c.can_be_initial(&compilation.chunk_group_by_ukey)
      {
        assets_manifest.pages.insert(
          page_chunk_name_regex.replace(name, "").to_string(),
          Vec::from_iter(
            c.files()
              .iter()
              // Only collect js and css files.
              .filter(|f| f.ends_with(".js") || f.ends_with(".css"))
              .cloned(),
          ),
        );
      }
    }
  });
  
  let json_string = serde_json::to_string(&assets_manifest).unwrap();
  compilation.emit_asset(
    "assets-manifest.json".to_string(),
    CompilationAsset::from(RawSource::from(json_string).boxed()),
  );
  Ok(())
}

impl Plugin for ManifestPlugin {
  fn name(&self) -> &'static str {
    "ManifestPlugin"
  }

  fn apply(
    &self,
    ctx: &mut ApplyContext,
  ) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}