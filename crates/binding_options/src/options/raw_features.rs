use std::{sync::Arc, hash::Hasher};
use futures_util::{FutureExt, future};
use napi_derive::napi;
use serde::Deserialize;
use rspack_core::{
  Optimization, PluginExt, SideEffectOption, UsedExportsOption, SourceType,
  BoxPlugin, Module, ModuleType,
};
use rspack_error::internal_error;
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin,
};
use rspack_binding_options::RawOptimizationOptions;
use rspack_plugin_split_chunks_new::{PluginOptions, CacheGroup};
use rspack_regex::RspackRegex;
use rspack_hash::{RspackHash, HashFunction, HashDigest};

pub struct SplitChunksStrategy {
  strategy: RawStrategyOptions,
  // Get the neccessary options from RawOptimizationOptions.
  chunk_ids: String,
  module_ids: String,
  side_effects: String,
  used_exports: String,
  provided_exports: bool,
  real_content_hash: bool,
  remove_empty_chunks: bool,
  remove_available_modules: bool,
}

fn get_modules_size(module: &dyn Module) -> f64 {
  let mut size = 0f64;
  for source_type in module.source_types() {
    size += module.size(source_type);
  }
  size
}

fn get_plugin_options(strategy: RawStrategyOptions, context: String) -> rspack_plugin_split_chunks_new::PluginOptions {
  use rspack_plugin_split_chunks_new::SplitChunkSizes;
  let default_size_types = [SourceType::JavaScript, SourceType::Unknown];
  let create_sizes = |size: Option<f64>| {
    size
      .map(|size| SplitChunkSizes::with_initial_value(&default_size_types, size))
      .unwrap_or_else(SplitChunkSizes::default)
  };

  let re_node_modules = RspackRegex::new("node_modules").unwrap();
  
  let cache_groups = vec![
    CacheGroup {
      key: String::from("framework"),
      name: rspack_plugin_split_chunks_new::create_chunk_name_getter_by_const_name("framework".to_string()),
      chunk_filter: rspack_plugin_split_chunks_new::create_all_chunk_filter(),
      priority: 40.0,
      test: Arc::new(move |module: &dyn Module| -> bool {
        module
          .name_for_condition()
          .map_or(false, |name| {
            strategy.top_level_frameworks.iter().any(|framework| name.starts_with(framework))
          })
      }),
      max_initial_requests: 25,
      max_async_requests: 25,
      reuse_existing_chunk: true,
      min_chunks: 1,
      // When enfoce is true, all size should be set to SplitChunkSizes::empty().
      min_size: SplitChunkSizes::empty(),
      max_async_size: SplitChunkSizes::empty(),
      max_initial_size: SplitChunkSizes::empty(),
      id_hint: String::from("framework"),
      r#type: rspack_plugin_split_chunks_new::create_default_module_type_filter(),
    },
    CacheGroup {
      key: String::from("lib"),
      name: Arc::new(move |module| {
        let mut hash = RspackHash::new(&HashFunction::Xxhash64);
        match module.module_type() {
          ModuleType::Css | ModuleType::CssModule | ModuleType::CssAuto => {
            module.update_hash(&mut hash);
          },
          _ => {
            let options = rspack_core::LibIdentOptions { context: &context };
            let lib_ident = module.lib_ident(options);
            hash.write(lib_ident.unwrap().as_bytes());
          },
        }
        future::ready(Some(hash.digest(&HashDigest::Hex).rendered(8).to_string())).boxed()
      }),
      chunk_filter: rspack_plugin_split_chunks_new::create_all_chunk_filter(),
      test: Arc::new(move |module: &dyn Module| -> bool {
        module
          .name_for_condition()
          .map_or(false, |name| re_node_modules.test(&name))
        && get_modules_size(module) > 160000.0
      }),
      priority: 30.0,
      min_chunks: 1,
      reuse_existing_chunk: true,
      max_initial_requests: 25,
      max_async_requests: 25,
      min_size: create_sizes(Some(20000.0)),
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
      id_hint: String::from("lib"),
      r#type: rspack_plugin_split_chunks_new::create_default_module_type_filter(),
    },
  ];

  PluginOptions {
    cache_groups,
    fallback_cache_group: rspack_plugin_split_chunks_new::FallbackCacheGroup {
      chunks_filter: rspack_plugin_split_chunks_new::create_all_chunk_filter(),
      min_size: SplitChunkSizes::default(),
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
    },
  }
}


pub trait FeatureApply {
  type Options;
  fn apply(self, plugins: &mut Vec<BoxPlugin>, context: String) -> Result<Self::Options, rspack_error::Error>;
}

impl SplitChunksStrategy {
  pub fn new(strategy: RawStrategyOptions, option: RawOptimizationOptions) -> Self {
    Self {
      strategy,
      chunk_ids: option.chunk_ids,
      module_ids: option.module_ids,
      remove_available_modules: option.remove_available_modules,
      remove_empty_chunks: option.remove_empty_chunks,
      side_effects: option.side_effects,
      used_exports: option.used_exports,
      provided_exports: option.provided_exports,
      real_content_hash: option.real_content_hash,
    }
  }
}

impl FeatureApply for SplitChunksStrategy {
  type Options = Optimization;

  fn apply(self, plugins: &mut Vec<Box<dyn rspack_core::Plugin>>, context: String) -> Result<Self::Options, rspack_error::Error> {
    let split_chunks_plugin = rspack_plugin_split_chunks_new::SplitChunksPlugin::new(
      get_plugin_options(self.strategy, context),
    ).boxed();
    plugins.push(split_chunks_plugin);

    let chunk_ids_plugin = match self.chunk_ids.as_ref() {
      "named" => NamedChunkIdsPlugin::new(None, None).boxed(),
      "deterministic" => DeterministicChunkIdsPlugin::default().boxed(),
      _ => {
        return Err(internal_error!(
          "'chunk_ids' should be 'named' or 'deterministic'."
        ))
      }
    };
    plugins.push(chunk_ids_plugin);
    let module_ids_plugin = match self.module_ids.as_ref() {
      "named" => NamedModuleIdsPlugin::default().boxed(),
      "deterministic" => DeterministicModuleIdsPlugin::default().boxed(),
      _ => {
        return Err(internal_error!(
          "'module_ids' should be 'named' or 'deterministic'."
        ))
      }
    };
    plugins.push(module_ids_plugin);
    if self.real_content_hash {
      plugins.push(rspack_plugin_real_content_hash::RealContentHashPlugin.boxed());
    }
    Ok(Self::Options {
      remove_available_modules: self.remove_available_modules,
      remove_empty_chunks: self.remove_empty_chunks,
      side_effects: SideEffectOption::from(self.side_effects.as_str()),
      provided_exports: self.provided_exports,
      used_exports: UsedExportsOption::from(self.used_exports.as_str()),
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawStrategyOptions {
  pub name: String,
  pub top_level_frameworks: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawFeatures {
  pub split_chunks_strategy: Option<RawStrategyOptions>,
}
