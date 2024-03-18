use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use napi::bindgen_prelude::*;
use rspack_binding_options::JsLoaderContext;
use rspack_core::{rspack_sources::SourceMap, Content, LoaderContext, ResourceData};
use rustc_hash::FxHashSet as HashSet;

use crate::get_builtin_loader;

pub async fn run_builtin_loader(
  builtin: String,
  options: Option<&str>,
  loader_context: JsLoaderContext,
) -> Result<JsLoaderContext> {
  use rspack_loader_runner::__private::loader::LoaderItemList;

  let loader = get_builtin_loader(&builtin, options);
  let loader_item = loader.clone().into();
  let list = &[loader_item];
  let additional_data = {
    let mut additional_data = loader_context.additional_data_external.clone();
    if let Some(data) = loader_context
      .additional_data
      .map(|b| String::from_utf8_lossy(b.as_ref()).to_string())
    {
      additional_data.insert(data);
    }
    additional_data
  };

  let mut cx = LoaderContext {
    hot: loader_context.hot,
    content: match loader_context.content {
      Either::A(_) => None,
      Either::B(c) => Some(Content::from(c.as_ref().to_owned())),
    },
    resource: &loader_context.resource,
    resource_path: Path::new(&loader_context.resource_path),
    resource_query: loader_context.resource_query.as_deref(),
    resource_fragment: loader_context.resource_fragment.as_deref(),
    context: loader_context.context_external.clone(),
    source_map: loader_context
      .source_map
      .map(|s| SourceMap::from_slice(s.as_ref()))
      .transpose()
      .map_err(|e| Error::from_reason(e.to_string()))?,
    additional_data,
    cacheable: loader_context.cacheable,
    file_dependencies: HashSet::from_iter(
      loader_context
        .file_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    context_dependencies: HashSet::from_iter(
      loader_context
        .context_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    missing_dependencies: HashSet::from_iter(
      loader_context
        .missing_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    build_dependencies: HashSet::from_iter(
      loader_context
        .build_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    asset_filenames: HashSet::from_iter(loader_context.asset_filenames.into_iter()),
    // Initialize with no diagnostic
    __diagnostics: vec![],
    __resource_data: &ResourceData::new(Default::default(), Default::default()),
    __loader_items: LoaderItemList(list),
    // This is used an hack to `builtin:swc-loader` in order to determine whether to return AST or source.
    __loader_index: loader_context.loader_index_from_js.unwrap_or(0) as usize,
    __plugins: &[],
  };
  if loader_context.is_pitching {
    // Builtin loaders dispatched using JS loader-runner does not support pitching.
    // This phase is ignored.
  } else {
    // Run normal loader
    loader
      .run(&mut cx)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    // restore the hack
    cx.__loader_index = 0;
  }

  JsLoaderContext::try_from(&mut cx).map_err(|e| Error::from_reason(e.to_string()))
}