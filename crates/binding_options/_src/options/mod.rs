use napi_derive::napi;
use rspack_binding_options::{
  RawBuiltins, RawCacheOptions, RawExperiments, RawMode, RawModuleOptions, RawNodeOption,
  RawOutputOptions, RawResolveOptions, RawSnapshotOptions, RawStatsOptions,
};
use rspack_core::{
  CacheOptions, CompilerOptions, Context, Experiments, IncrementalRebuild,
  IncrementalRebuildMakeState, ModuleOptions, Optimization, OutputOptions, Target, TreeShaking,
};
use serde::Deserialize;

mod js_loader;
mod raw_features;
mod raw_module;
mod raw_optimization;

pub use js_loader::*;
pub use raw_features::*;
pub use raw_module::*;
pub use raw_optimization::*;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RSPackRawOptions {
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  pub target: Vec<String>,
  pub context: String,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub resolve_loader: RawResolveOptions,
  pub module: RawModuleOptions,
  pub devtool: String,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub bail: bool,
  #[napi(js_name = "__references", ts_type = "Record<string, any>")]
  pub __references: References,
  pub features: RawFeatures,
}

impl TryFrom<RawOptions> for CompilerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptions) -> Result<Self, rspack_error::Error> {
    let context: Context = value.context.into();
    let output: OutputOptions = value.output.try_into()?;
    let resolve = value.resolve.try_into()?;
    let resolve_loader = value.resolve_loader.try_into()?;
    let mode = value.mode.unwrap_or_default().into();
    let module: ModuleOptions = value.module.try_into()?;
    let target = Target::new(&value.target)?;
    let cache = value.cache.into();
    let experiments = Experiments {
      incremental_rebuild: IncrementalRebuild {
        make: if matches!(cache, CacheOptions::Disabled) {
          None
        } else {
          Some(IncrementalRebuildMakeState::default())
        },
        emit_asset: true,
      },
      top_level_await: value.experiments.top_level_await,
      rspack_future: value.experiments.rspack_future.into(),
    };
    let optimization = value.optimization.try_into()?;
    let optimization: Optimization;
    if self.features.split_chunks_strategy.is_some() {
      let split_chunk_strategy = SplitChunksStrategy::new(
        self.features.split_chunks_strategy.unwrap(),
        self.optimization,
      );
      optimization = split_chunk_strategy.apply(plugins, context.to_string())?;
    } else {
      optimization = value.optimization.try_into()?;;
    }
    let stats = value.stats.into();
    let snapshot = value.snapshot.into();
    let node = value.node.map(|n| n.into());

    // Add custom plugins.
    if self.features.assets_manifest.unwrap_or_default() {
      plugins.push(Box::new(plugin_manifest::ManifestPlugin::new()));
    }

    let mut builtins = self.builtins.apply(plugins)?;

    Ok(CompilerOptions {
      context,
      mode,
      module,
      target,
      output,
      resolve,
      resolve_loader,
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      dev_server: Default::default(),
      profile: value.profile,
      bail: value.bail,
      builtins,
      __references: value.__references,
    })
  }
}
