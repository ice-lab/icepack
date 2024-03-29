use std::{collections::HashMap, path::Path, sync::Mutex};

use lazy_static::lazy_static;
use rspack_ast::RspackAst;
use rspack_core::{
  rspack_sources::SourceMap, LoaderRunnerContext, LoadersShouldAlwaysGiveContent, Mode,
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
  base::config::{Config, InputSourceMap, Options, OutputCharset, TransformConfig},
  ecma::parser::{Syntax, TsConfig},
};

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
}

lazy_static! {
  static ref GLOBAL_FILE_ACCESS: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
  static ref GLOBAL_ROUTES_CONFIG: Mutex<Option<Vec<String>>> = Mutex::new(None);
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for CompilationLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let resource_path = loader_context.resource_path.to_path_buf();

    if self.loader_options.compile_rules.exclude.is_some() {
      let exclude = self.loader_options.compile_rules.exclude.as_ref().unwrap();
      for pattern in exclude {
        let pattern = RspackRegex::new(pattern).unwrap();
        if pattern.test(&resource_path.to_string_lossy()) {
          return Ok(());
        }
      }
    }

    let content = std::mem::take(&mut loader_context.content).expect("content should be available");

    let swc_options = {
      let mut swc_options = self.loader_options.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        transform.react.development = Some(Mode::is_development(&loader_context.context.options.mode));
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }

      let file_extension = resource_path.extension().unwrap();
      let ts_extensions = vec!["tsx", "ts", "mts"];
      if ts_extensions.iter().any(|ext| ext == &file_extension) {
        swc_options.config.jsc.syntax = Some(Syntax::Typescript(TsConfig {
          tsx: true,
          decorators: true,
          ..Default::default()
        }));
      }

      if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          COMPILATION_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }
      swc_options
    };
    let source_map_kind = &loader_context.context.module_source_map_kind;
    let source = content.try_into_string()?;
    let compiler = SwcCompiler::new(resource_path.clone(), source.clone(), swc_options)
      .map_err(AnyhowError::from)?;

    let transform_options = &self.loader_options.transform_features;
    let compiler_context: &str = loader_context.context.options.context.as_ref();
    let mut file_access = GLOBAL_FILE_ACCESS.lock().unwrap();
    let mut routes_config = GLOBAL_ROUTES_CONFIG.lock().unwrap();
    let file_accessed = file_access.contains_key(&resource_path.to_string_lossy().to_string());

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
    file_access.insert(resource_path.to_string_lossy().to_string(), true);

    let built = compiler
      .parse(None, |_| {
        transform(&resource_path, routes_config.as_ref(), transform_options)
      })
      .map_err(AnyhowError::from)?;

    let codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: !matches!(source_map_kind, SourceMapKind::None),
        inline_sources_content: true,
        emit_columns: matches!(source_map_kind, SourceMapKind::SourceMap),
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };
    let program = compiler.transform(built).map_err(AnyhowError::from)?;
    let ast = compiler.into_js_ast(program);

    // If swc-loader is the latest loader available,
    // then loader produces AST, which could be used as an optimization.
    if loader_context.loader_index() == 0
      && (loader_context
        .current_loader()
        .composed_index_by_identifier(&self.identifier)
        .map(|idx| idx == 0)
        .unwrap_or(true))
      && !loader_context
        .additional_data
        .contains::<&LoadersShouldAlwaysGiveContent>()
    {
      loader_context
        .additional_data
        .insert(RspackAst::JavaScript(ast));
      loader_context.additional_data.insert(codegen_options);
      loader_context.content = Some("".to_owned().into())
    } else {
      let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;
      loader_context.content = Some(code.into());
      loader_context.source_map = map
        .map(|m| SourceMap::from_json(&m))
        .transpose()
        .map_err(|e| error!(e.to_string()))?;
    }

    Ok(())
  }
}

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
