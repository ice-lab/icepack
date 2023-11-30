use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CompilerOptions, Context, DevServerOptions, Devtool, Experiments, IncrementalRebuild,
  IncrementalRebuildMakeState, ModuleOptions, ModuleType, OutputOptions, PluginExt, Optimization,
};
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, SideEffectsFlagPlugin,
};
use serde::Deserialize;

use rspack_binding_options::{
  RawBuiltins, RawCacheOptions, RawContext, RawDevServer, RawDevtool, RawExperiments,
  RawMode, RawNodeOption, RawOutputOptions, RawResolveOptions, RawOptionsApply,
  RawSnapshotOptions, RawStatsOptions, RawTarget, RawModuleOptions,
};

mod raw_module;
mod raw_features;
mod js_loader;
mod raw_optimization;

pub use raw_module::*;
pub use raw_features::*;
pub use js_loader::*;
pub use raw_optimization::*;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RSPackRawOptions {
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  #[napi(ts_type = "Array<string>")]
  pub target: RawTarget,
  #[napi(ts_type = "string")]
  pub context: RawContext,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub resolve_loader: RawResolveOptions,
  pub module: RawModuleOptions,
  #[napi(ts_type = "string")]
  pub devtool: RawDevtool,
  pub optimization: RspackRawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub dev_server: RawDevServer,
  pub snapshot: RawSnapshotOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub builtins: RawBuiltins,
  pub features: RawFeatures,
}

impl RawOptionsApply for RSPackRawOptions {
  type Options = CompilerOptions;

  fn apply(self, plugins: &mut Vec<BoxPlugin>) -> Result<Self::Options, rspack_error::Error> {
    let context: Context = self.context.into();
    let output: OutputOptions = self.output.apply(plugins)?;
    let resolve = self.resolve.try_into()?;
    let resolve_loader = self.resolve_loader.try_into()?;
    let devtool: Devtool = self.devtool.into();
    let mode = self.mode.unwrap_or_default().into();
    let module: ModuleOptions = self.module.apply(plugins)?;
    let target = self.target.apply(plugins)?;
    let cache = self.cache.into();
    let experiments = Experiments {
      lazy_compilation: self.experiments.lazy_compilation,
      incremental_rebuild: IncrementalRebuild {
        make: self
          .experiments
          .incremental_rebuild
          .make
          .then(IncrementalRebuildMakeState::default),
        emit_asset: self.experiments.incremental_rebuild.emit_asset,
      },
      async_web_assembly: self.experiments.async_web_assembly,
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
        self.optimization.apply(plugins)
      })?;
    }

    let stats = self.stats.into();
    let snapshot = self.snapshot.into();
    let node = self.node.map(|n| n.into());
    let dev_server: DevServerOptions = self.dev_server.into();

    plugins.push(rspack_plugin_schemes::DataUriPlugin.boxed());
    plugins.push(rspack_plugin_schemes::FileUriPlugin.boxed());

    plugins.push(
      rspack_plugin_asset::AssetPlugin::new(rspack_plugin_asset::AssetConfig {
        parse_options: module
          .parser
          .as_ref()
          .and_then(|x| x.get(&ModuleType::Asset))
          .and_then(|x| x.get_asset(&ModuleType::Asset).cloned()),
      })
      .boxed(),
    );
    plugins.push(rspack_plugin_json::JsonPlugin {}.boxed());
    plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
    if experiments.lazy_compilation {
      plugins.push(rspack_plugin_runtime::LazyCompilationPlugin {}.boxed());
    }
    if experiments.async_web_assembly {
      plugins.push(rspack_plugin_wasm::AsyncWasmPlugin::new().boxed());
    }
    rspack_plugin_worker::worker_plugin(
      output.worker_chunk_loading.clone(),
      output.worker_wasm_loading.clone(),
      plugins,
    );
    plugins.push(rspack_plugin_javascript::JsPlugin::new().boxed());
    plugins.push(rspack_plugin_javascript::InferAsyncModulesPlugin {}.boxed());

    if devtool.source_map() {
      plugins.push(
        rspack_plugin_devtool::DevtoolPlugin::new(rspack_plugin_devtool::DevtoolPluginOptions {
          inline: devtool.inline(),
          append: !devtool.hidden(),
          namespace: output.unique_name.clone(),
          columns: !devtool.cheap(),
          no_sources: devtool.no_sources(),
          public_path: None,
        })
        .boxed(),
      );
    }

    plugins.push(rspack_ids::NamedChunkIdsPlugin::new(None, None).boxed());

    if experiments.rspack_future.new_treeshaking {
      if optimization.side_effects.is_enable() {
        plugins.push(SideEffectsFlagPlugin::default().boxed());
      }
      if optimization.provided_exports {
        plugins.push(FlagDependencyExportsPlugin::default().boxed());
      }
      if optimization.used_exports.is_enable() {
        plugins.push(FlagDependencyUsagePlugin::default().boxed());
      }
    }

    // Notice the plugin need to be placed after SplitChunksPlugin
    if optimization.remove_empty_chunks {
      plugins.push(rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin.boxed());
    }

    plugins.push(rspack_plugin_ensure_chunk_conditions::EnsureChunkConditionsPlugin.boxed());

    plugins.push(rspack_plugin_warn_sensitive_module::WarnCaseSensitiveModulesPlugin.boxed());

    // Add custom plugins.
    plugins.push(plugin_manifest::ManifestPlugin::new().boxed());

    Ok(Self::Options {
      context,
      mode,
      module,
      target,
      output,
      resolve,
      resolve_loader,
      devtool,
      experiments,
      stats,
      cache,
      snapshot,
      optimization,
      node,
      dev_server,
      profile: self.profile,
      builtins: self.builtins.apply(plugins)?,
    })
  }
}
