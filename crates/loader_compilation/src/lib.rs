use rspack_ast::RspackAst;
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_error::{
  internal_error, Result,
};
use swc_core::base::config::{InputSourceMap, Options, OutputCharset};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
mod compiler;
mod transform;

use transform::*;
use compiler::{SwcCompiler, IntoJsAst};
pub struct CompilationLoader {
  identifier: Identifier,
}

impl CompilationLoader {
  pub fn new() -> Self {
    Self {
      identifier: COMPILATION_LOADER_IDENTIFIER.into(),
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
    // TODO: init loader with custom options.
    let mut swc_options = Options::default();

    // TODO: merge config with built-in config.

    if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
      if let Ok(source_map) = pre_source_map.to_json() {
        swc_options.config.input_source_map = Some(InputSourceMap:: Str(source_map))
      }
    }

    let devtool = &loader_context.context.options.devtool;
    let source = content.try_into_string()?;
    let compiler = SwcCompiler::new(resource_path.clone(), source.clone(), swc_options)?;

    let transform_options = SwcPluginOptions {
      keep_export: Some(KeepExportOptions {
        export_names: vec!["default".to_string()],
      }),
      remove_export: Some(RemoveExportOptions {
        remove_names: vec!["default".to_string()],
      }),
    };
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
    if loader_context.loader_index() == 0
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

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}