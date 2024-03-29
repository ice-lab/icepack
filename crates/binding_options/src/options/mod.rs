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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
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
  pub optimization: RspackRawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub bail: bool,
  pub builtins: RawBuiltins,
  pub features: RawFeatures,
}

impl RSPackRawOptions {
  pub fn apply(
    self,
    plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> rspack_error::Result<CompilerOptions> {
    let context: Context = self.context.into();
    let output: OutputOptions = self.output.try_into()?;
    let resolve = self.resolve.try_into()?;
    let resolve_loader = self.resolve_loader.try_into()?;
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.try_into()?;
    let target = Target::new(&self.target)?;
    let cache = self.cache.into();
    let experiments = Experiments {
      incremental_rebuild: IncrementalRebuild {
        make: if matches!(cache, CacheOptions::Disabled)
          || self.experiments.rspack_future.new_treeshaking
        {
          None
        } else {
          Some(IncrementalRebuildMakeState::default())
        },
        emit_asset: true,
      },
      new_split_chunks: self.experiments.new_split_chunks,
      top_level_await: self.experiments.top_level_await,
      rspack_future: self.experiments.rspack_future.into(),
    };
    let optimization: Optimization;
    if self.features.split_chunks_strategy.is_some() {
      let split_chunk_strategy = SplitChunksStrategy::new(
        self.features.split_chunks_strategy.unwrap(),
        self.optimization,
      );
      optimization = split_chunk_strategy.apply(plugins, context.to_string())?;
    } else {
      optimization = IS_ENABLE_NEW_SPLIT_CHUNKS.set(&experiments.new_split_chunks, || {
        self.optimization.try_into()
      })?;
    }
    let stats = self.stats.into();
    let snapshot = self.snapshot.into();
    let node = self.node.map(|n| n.into());

    // Add custom plugins.
    if self.features.assets_manifest.unwrap_or_default() {
      plugins.push(Box::new(plugin_manifest::ManifestPlugin::new()));
    }

    let mut builtins = self.builtins.apply(plugins)?;
    if experiments.rspack_future.new_treeshaking {
      builtins.tree_shaking = TreeShaking::False;
    }

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
      profile: self.profile,
      bail: self.bail,
      builtins,
    })
  }
}
