use std::{collections::HashMap, path::Path};

use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  CompilationAsset, Plugin,
  PublicPath, Compilation,
  CompilationProcessAssets,
};
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
  let mut assets_mainfest = AssetsManifest {
    pages: HashMap::new(),
    entries: HashMap::new(),
    assets: HashMap::new(),
    public_path: public_path.unwrap_or_default().to_string(),
    data_loader: None,
  };
  let entry_points = &compilation.entrypoints;
  let assets = &compilation.assets();

  assets.into_iter().for_each(|(_, asset)| {
    let version = &asset.info.version;
    let source_file = &asset.info.source_filename;
    if let Some(name) = source_file {
      assets_mainfest
        .assets
        .insert(name.to_string(), version.to_string());
    }
  });
  entry_points.iter().for_each(|(name, _entry)| {
    let mut files: Vec<String> = Vec::new();
    compilation
      .entrypoint_by_name(name)
      .chunks
      .iter()
      .for_each(|chunk| {
        let chunk = compilation.chunk_by_ukey.expect_get(chunk);
        chunk.files().iter().for_each(|file| {
          if let Some(asset) = assets.get(file) {
            if !asset.info.hot_module_replacement.unwrap_or(false) && !asset.info.development.unwrap_or(false) {
              files.push(file.to_string());
            }
          } else {
            files.push(file.to_string());
          }
        });
      });
    assets_mainfest.entries.insert(name.to_string(), files);
  });
  // Check .ice/data-loader.ts is exists
  let data_loader_file =
    Path::new(&compilation.options.context.as_str()).join(".ice/data-loader.ts");
  if data_loader_file.exists() {
    assets_mainfest.data_loader = Some("js/data-loader.js".to_string());
  }

  let page_chunk_name_regex = regex::Regex::new(r"^p_").unwrap();
  compilation.chunk_by_ukey.values().for_each(|c| {
    if let Some(name) = c.id() {
      if !c.has_entry_module(&compilation.chunk_graph)
        && !c.can_be_initial(&compilation.chunk_group_by_ukey)
      {
        assets_mainfest.pages.insert(
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
  let json_string = serde_json::to_string(&assets_mainfest).unwrap();
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
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
