use rspack_ast::RspackAst;
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_error::{
  internal_error, Result, Diagnostic,
};
use swc_core::{base::config::{InputSourceMap, Options, OutputCharset, Config, TransformConfig}, ecma::transforms::react::Runtime, common::source_map};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
mod compiler;
mod transform;

use transform::*;
use compiler::{SwcCompiler, IntoJsAst};

pub struct LoaderOptions {
  pub swc_options: Config,
  pub transform_features: TransformFeatureOptions,
}

pub struct CompilationOptions {
  swc_options: Options,
  transform_features: TransformFeatureOptions,
}
pub struct CompilationLoader {
  identifier: Identifier,
  loader_options: CompilationOptions,
}

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

impl From<LoaderOptions> for CompilationOptions {
  fn from(value: LoaderOptions) -> Self {
    let transform_features = TransformFeatureOptions::default();
    CompilationOptions {
      swc_options: Options {
        config: value.swc_options,
        ..Default::default()
      },
      transform_features,
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

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for CompilationLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let resource_path = loader_context.resource_path.to_path_buf();
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Err(internal_error!("No content found"));
    };

    let swc_options = {
      let mut swc_options = self.loader_options.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        let default_development = matches!(loader_context.context.options.mode, Mode::Development);
        transform.react.development = Some(default_development);
        transform.react.runtime = Some(Runtime::Automatic);
      }

      if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap:: Str(source_map))
        }
      }

      if swc_options.config.jsc.experimental.plugins.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          COMPILATION_LOADER_IDENTIFIER.to_string(),
          "Experimental plugins are not currently supported.".to_string(),
          0,
          0,
        ));
      }

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          COMPILATION_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
          0,
          0,
        ));
      }
      swc_options
    };
    let devtool = &loader_context.context.options.devtool;
    let source = content.try_into_string()?;
    let compiler = SwcCompiler::new(resource_path.clone(), source.clone(), swc_options)?;

    let transform_options = &self.loader_options.transform_features;
    let built = compiler.parse(None, |_| {
      transform(transform_options)
    })?;

    let codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: devtool.source_map(),
        inline_sources_content: true,
        emit_columns: !devtool.cheap(),
        names: Default::default(),
      },
      keep_comments: Some(true),
    };
    let program = compiler.transform(built)?;
    let ast = compiler.into_js_ast(program);

    // If swc-loader is the latest loader available,
    // then loader produces AST, which could be used as an optimization.
    if loader_context.loader_index() == 1
      && (loader_context
        .current_loader()
        .composed_index_by_identifier(&self.identifier)
        .map(|idx| idx == 0)
        .unwrap_or(true))
    {
      loader_context
        .additional_data
        .insert(RspackAst::JavaScript(ast));
      loader_context.additional_data.insert(codegen_options);
      loader_context.content = Some("".to_owned().into())
    } else {
      let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;
      loader_context.content = Some(code.into());
      loader_context.source_map = map.map(|m| SourceMap::from_json(&m)).transpose()?;
    }

    Ok(())
  }
}

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}