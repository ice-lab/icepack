use std::{collections::HashMap, path::Path, sync::Mutex};

use lazy_static::lazy_static;
use rspack_core::{
  rspack_sources::SourceMap, Mode, RunnerContext,
};
use rspack_error::{error, AnyhowError, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
use rspack_regex::RspackRegex;
use rspack_util::source_map::SourceMapKind;
use serde::Deserialize;
use swc_config::{config_types::MergingOption, merge::Merge};
use swc_core::{
  base::config::{Config, InputSourceMap, Options, OutputCharset, TransformConfig, SourceMapsConfig},
  ecma::parser::{Syntax, TsSyntax},
};
use swc_core::ecma::visit::VisitWith;

mod transform;

use swc_compiler::{IntoJsAst, SwcCompiler};
use transform::*;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompileRules {
  // Built-in rules to exclude files from compilation, such as react, react-dom, etc.
  exclude: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LoaderOptions {
  pub swc_options: Config,
  pub transform_features: TransformFeatureOptions,
  pub compile_rules: CompileRules,
}

pub struct CompilationOptions {
  swc_options: Options,
  transform_features: TransformFeatureOptions,
  compile_rules: CompileRules,
}

pub struct CompilationLoader {
  identifier: Identifier,
  loader_options: CompilationOptions,
}

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

impl From<LoaderOptions> for CompilationOptions {
  fn from(value: LoaderOptions) -> Self {
    let transform_features = value.transform_features;
    let compile_rules = value.compile_rules;
    CompilationOptions {
      swc_options: Options {
        config: value.swc_options,
        ..Default::default()
      },
      transform_features,
      compile_rules,
    }
  }
}

lazy_static! {
  static ref GLOBAL_FILE_ACCESS: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
  static ref GLOBAL_ROUTES_CONFIG: Mutex<Option<Vec<String>>> = Mutex::new(None);
}

impl CompilationLoader {
  pub fn new(options: LoaderOptions) -> Self {
    Self {
      identifier: COMPILATION_LOADER_IDENTIFIER.into(),
      loader_options: options.into(),
    }
  }

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
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Ok(());
    };

    if self.loader_options.compile_rules.exclude.is_some() {
      let exclude = self.loader_options.compile_rules.exclude.as_ref().unwrap();
      for pattern in exclude {
        let pattern = RspackRegex::new(pattern).unwrap();
        if pattern.test(&resource_path.as_str()) {
          return Ok(());
        }
      }
    }

    let resource_str = resource_path.as_str().to_string();

    let swc_options = {
      let mut swc_options = self.loader_options.swc_options.clone();
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
      if let Some(pre_source_map) = loader_context.source_map.clone() {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }
      swc_options.filename = resource_path.as_str().to_string();
      swc_options.source_file_name = Some(resource_path.as_str().to_string());

      let file_extension = resource_path.extension().unwrap();
      let ts_extensions = vec!["tsx", "ts", "mts"];
      if ts_extensions.iter().any(|ext| ext == &file_extension) {
        swc_options.config.jsc.syntax = Some(Syntax::Typescript(TsSyntax {
          tsx: true,
          decorators: true,
          ..Default::default()
        }));
      }
      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          COMPILATION_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }
      swc_options
    };

    let source_map_kind: SourceMapKind = match swc_options.config.source_maps {
      Some(SourceMapsConfig::Bool(false)) => SourceMapKind::empty(),
      _ => loader_context.context.module_source_map_kind,
    };

    let source = content.try_into_string()?;
    let c = SwcCompiler::new(
      resource_path.clone().into_std_path_buf(),
      source.clone(),
      swc_options,
    )
    .map_err(AnyhowError::from)?;

    let compiler_context: &str = loader_context.context.options.context.as_ref();
    let mut file_access = GLOBAL_FILE_ACCESS.lock().unwrap();
    let mut routes_config = GLOBAL_ROUTES_CONFIG.lock().unwrap();
    let file_accessed = file_access.contains_key(&resource_str);

    if routes_config.is_none() || file_accessed {
      // Load routes config for transform.
      let routes_config_path: std::path::PathBuf =
        Path::new(compiler_context).join(".ice/route-manifest.json");
      let routes_content = load_routes_config(&routes_config_path);
      if routes_content.is_ok() {
        *routes_config = Some(routes_content.map_err(AnyhowError::from)?);
      }
      if file_accessed {
        // If file accessed, then we need to clear the map for the current compilation.
        file_access.clear();
      }
    }
    file_access.insert(resource_str, true);

    let transform_options = &self.loader_options.transform_features;
    let built = c
      .parse(None, |_| {
        transform(&resource_path.as_str(), routes_config.as_ref(), transform_options)
      })
      .map_err(AnyhowError::from)?;

    let input_source_map = c
      .input_source_map(&built.input_source_map)
      .map_err(|e| error!(e.to_string()))?;
    let mut codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      input_source_map: input_source_map.as_ref(),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: source_map_kind.source_map(),
        inline_sources_content: source_map_kind.source_map(),
        emit_columns: !source_map_kind.cheap(),
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };

    let program = tokio::task::block_in_place(|| c.transform(built).map_err(AnyhowError::from))?;
    if source_map_kind.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };
      program.visit_with(&mut v);
      codegen_options.source_map_config.names = v.names;
    }
    let ast = c.into_js_ast(program);
    let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;

    loader_context.content = Some(code.into());
    let map = map
      .map(|m| SourceMap::from_json(&m))
      .transpose()
      .map_err(|e| error!(e.to_string()))?;
    loader_context.source_map = map;

    Ok(())
  }
}

#[async_trait::async_trait]
impl Loader<RunnerContext> for CompilationLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(debug_assertions)]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(not(debug_assertions))]
    inner()
  }
}

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
