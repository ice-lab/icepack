use std::{
  path::{Path, PathBuf},
  str::FromStr,
};
use napi::bindgen_prelude::*;
use rspack_core::{
  rspack_sources::SourceMap,
  LoaderContext, Content, ResourceData,
};
use rustc_hash::FxHashSet as HashSet;
use rspack_binding_options::JsLoaderContext;
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

  let mut cx = LoaderContext {
    content: loader_context
      .content
      .map(|c| Content::from(c.as_ref().to_owned())),
    resource: &loader_context.resource,
    resource_path: Path::new(&loader_context.resource_path),
    resource_query: loader_context.resource_query.as_deref(),
    resource_fragment: loader_context.resource_fragment.as_deref(),
    context: loader_context.context.clone(),
    source_map: loader_context
      .source_map
      .map(|s| SourceMap::from_slice(s.as_ref()))
      .transpose()
      .map_err(|e| Error::from_reason(e.to_string()))?,
    additional_data: loader_context
      .additional_data
      .map(|b| String::from_utf8_lossy(b.as_ref()).to_string()),
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
    __loader_index: 0,
    __plugins: &[],
  };
  if loader_context.is_pitching {
    // Run pitching loader
    loader
      .pitch(&mut cx)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
  } else {
    // Run normal loader
    loader
      .run(&mut cx)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
  }

  JsLoaderContext::try_from(&cx).map_err(|e| Error::from_reason(e.to_string()))
}