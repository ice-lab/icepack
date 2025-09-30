mod options;
mod transformer;
mod transforms;

use std::{default::Default, path::Path};

use options::CompilationOptionsWithAdditional;
pub use options::CompilationLoaderJsOptions;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Mode, RunnerContext, Loader, LoaderContext};
use rspack_error::{Diagnostic, Result, Error};
use rspack_javascript_compiler::{JavaScriptCompiler, TransformOutput};
use rspack_collections::Identifier;
use sugar_path::SugarPath;
use swc_config::{merge::Merge, types::MergingOption};
use swc_core::{
  base::config::{InputSourceMap, TransformConfig},
  common::FileName,
};

#[cacheable]
#[derive(Debug)]
pub struct CompilationLoader {
  identifier: Identifier,
  options_with_additional: CompilationOptionsWithAdditional,
}

impl CompilationLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: COMPILATION_LOADER_IDENTIFIER.into(),
      options_with_additional: raw_options.try_into()?,
    })
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:compilation-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(COMPILATION_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }

  fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let resource_path = loader_context
      .resource_path()
      .map(|p| p.to_path_buf())
      .unwrap_or_default();
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    // Check compile rules for exclusion
    if let Some(exclude_patterns) = &self.options_with_additional.compile_rules.exclude {
      for pattern in exclude_patterns {
        let regex = regex::Regex::new(pattern).map_err(|e| {
          rspack_error::error!("Invalid regex pattern '{}': {}", pattern, e)
        })?;
        if regex.is_match(resource_path.as_str()) {
          // Skip compilation for excluded files, return content as-is
          let source = content.into_string_lossy();
          loader_context.finish_with((source, None));
          return Ok(());
        }
      }
    }

    let swc_options = {
      let mut swc_options = self.options_with_additional.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        transform.react.development =
          Some(Mode::is_development(&loader_context.context.options.mode));
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }
      if let Some(pre_source_map) = loader_context.source_map().cloned() {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }
      swc_options.filename = resource_path.as_str().to_string();
      swc_options.source_file_name = Some(resource_path.as_str().to_string());

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          COMPILATION_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }

      #[cfg(feature = "plugin")]
      {
        swc_options.runtime_options =
          swc_options
            .runtime_options
            .plugin_runtime(std::sync::Arc::new(
              rspack_util::swc::runtime::WasmtimeRuntime,
            ));
      }

      swc_options
    };

    let javascript_compiler = JavaScriptCompiler::new();
    let filename = FileName::Real(resource_path.clone().into_std_path_buf());

    let source = content.into_string_lossy();
    let _is_typescript =
      matches!(swc_options.config.jsc.syntax, Some(syntax) if syntax.typescript());

    let TransformOutput {
      code,
      mut map,
      diagnostics,
    } = javascript_compiler.transform(
      source,
      Some(filename),
      swc_options,
      Some(loader_context.context.module_source_map_kind),
      |_program| {
        // TypeScript info collection could be added here if needed
      },
      |_| transformer::transform(&self.options_with_additional.transform_features),
    )?;

    for diagnostic in diagnostics {
      loader_context.emit_diagnostic(Error::warning(diagnostic).into());
    }

    // When compiling target modules, SWC retrieves the source map via sourceMapUrl.
    // The sources paths in the source map are relative to the target module. We need to resolve these paths
    // to absolute paths using the resource path to avoid incorrect project path references.
    if let (Some(map), Some(resource_dir)) = (map.as_mut(), resource_path.parent()) {
      map.set_sources(
        map
          .sources()
          .iter()
          .map(|source| {
            let source_path = Path::new(source);
            if source_path.is_relative() {
              source_path
                .absolutize_with(resource_dir.as_std_path())
                .to_string_lossy()
                .into_owned()
            } else {
              source.to_string()
            }
          })
          .collect::<Vec<_>>(),
      );
    }

    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:builtin-compilation", skip_all, fields(
    perfetto.track_name = "loader:builtin-compilation",
    perfetto.process_name = "Loader Analysis",
    resource = loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(all(debug_assertions, not(target_family = "wasm")))]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(any(not(debug_assertions), target_family = "wasm"))]
    inner()
  }
}

use std::sync::Arc;
use rspack_core::{NormalModuleFactoryResolveLoader, Plugin, ApplyContext};
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug)]
pub struct CompilationLoaderPlugin;

impl Default for CompilationLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl CompilationLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for CompilationLoaderPlugin, tracing = false)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &rspack_core::Context,
  _resolver: &rspack_core::Resolver,
  loader: &rspack_core::ModuleRuleUseLoader,
) -> Result<Option<rspack_core::BoxLoader>> {
  if loader.loader.starts_with(COMPILATION_LOADER_IDENTIFIER) {
    let options = loader.options.clone().unwrap_or_default();
    let compilation_loader = CompilationLoader::new(&options)
      .map_err(|e| rspack_error::error!("Failed to create CompilationLoader: {}", e))?;
    return Ok(Some(Arc::new(compilation_loader)));
  }

  Ok(None)
}

impl Plugin for CompilationLoaderPlugin {
  fn name(&self) -> &'static str {
    "CompilationLoaderPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}