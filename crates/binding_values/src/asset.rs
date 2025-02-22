use napi_derive::napi;

#[napi(object)]
pub struct JsAssetInfoRelated {
  pub source_map: Option<String>,
}

impl From<JsAssetInfoRelated> for rspack_core::AssetInfoRelated {
  fn from(i: JsAssetInfoRelated) -> Self {
    Self {
      source_map: i.source_map,
    }
  }
}

#[napi(object)]
pub struct JsAssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  pub immutable: Option<bool>,
  /// whether the asset is minimized
  pub minimized: Option<bool>,
  /// the value(s) of the full hash used for this asset
  pub fullhash: Vec<String>,
  /// the value(s) of the chunk hash used for this asset
  pub chunkhash: Vec<String>,
  /// the value(s) of the module hash used for this asset
  // pub modulehash:
  /// the value(s) of the content hash used for this asset
  pub contenthash: Vec<String>,
  // when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  pub source_filename: Option<String>,
  /// when asset was created from a source file (potentially transformed), it should be flagged as copied
  pub copied: Option<bool>,
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: Option<bool>,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: Option<bool>,
  /// when asset is javascript and an ESM
  pub javascript_module: Option<bool>,
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: JsAssetInfoRelated,
  /// unused css local ident for the css chunk
  pub css_unused_idents: Option<Vec<String>>,
  /// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
  /// But Napi.rs does not support Intersectiont types. This is a hack to store the additional fields
  /// in the rust struct and have the Js side to reshape and align with webpack
  /// Related: packages/rspack/src/Compilation.ts
  pub extras: serde_json::Map<String, serde_json::Value>,
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,
}

impl From<JsAssetInfo> for rspack_core::AssetInfo {
  fn from(i: JsAssetInfo) -> Self {
    Self {
      immutable: i.immutable,
      minimized: i.minimized,
      development: i.development,
      hot_module_replacement: i.hot_module_replacement,
      chunk_hash: i.chunkhash.into_iter().collect(),
      related: i.related.into(),
      full_hash: i.fullhash.into_iter().collect(),
      content_hash: i.contenthash.into_iter().collect(),
      version: String::from(""),
      source_filename: i.source_filename,
      copied: i.copied,
      javascript_module: i.javascript_module,
      css_unused_idents: i.css_unused_idents.map(|i| i.into_iter().collect()),
      extras: i.extras,
      is_over_size_limit: i.is_over_size_limit,
    }
  }
}

#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub info: JsAssetInfo,
}

impl From<rspack_core::AssetInfoRelated> for JsAssetInfoRelated {
  fn from(related: rspack_core::AssetInfoRelated) -> Self {
    Self {
      source_map: related.source_map,
    }
  }
}

impl From<rspack_core::AssetInfo> for JsAssetInfo {
  fn from(info: rspack_core::AssetInfo) -> Self {
    Self {
      immutable: info.immutable,
      minimized: info.minimized,
      development: info.development,
      hot_module_replacement: info.hot_module_replacement,
      related: info.related.into(),
      chunkhash: info.chunk_hash.into_iter().collect(),
      fullhash: info.full_hash.into_iter().collect(),
      contenthash: info.content_hash.into_iter().collect(),
      source_filename: info.source_filename,
      copied: info.copied,
      javascript_module: info.javascript_module,
      css_unused_idents: info.css_unused_idents.map(|i| i.into_iter().collect()),
      extras: info.extras,
      is_over_size_limit: info.is_over_size_limit,
    }
  }
}

#[napi(object)]
pub struct JsAssetEmittedArgs {
  pub filename: String,
  pub output_path: String,
  pub target_path: String,
}
