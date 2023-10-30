use std::sync::Arc;
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_error::{
  internal_error, Diagnostic, DiagnosticKind, Error, InternalError, Result, Severity,
  TraceableError,
};
use swc_core::{
  base::{
    Compiler,
    config::{InputSourceMap, Options},
    try_with_handler
  },
  common::{FilePathMapping,GLOBALS, FileName, comments::SingleThreadedComments},
  ecma::transforms::base::pass::noop
};
mod transform;
use transform::*;
pub struct CompilationLoader {
  identifier: Identifier,
}

impl Default for CompilationLoader {
  fn default() -> Self {
    Self {
      identifier: COMPILATION_LOADER_IDENTIFIER.into(),
    }
  }
}

impl CompilationLoader {
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

    let compiler = Compiler::new(Arc::from(swc_core::common::SourceMap::new(FilePathMapping::empty())));
    // TODO: init loader with custom options.
    let mut swc_options = Options::default();

    // TODO: merge config with built-in config.

    if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
      if let Ok(source_map) = pre_source_map.to_json() {
        swc_options.config.input_source_map = Some(InputSourceMap:: Str(source_map))
      }
    }

    GLOBALS.set(&Default::default(), || {
      try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
        compiler.run(|| {
          let content = content.try_into_string()?;
          let fm = compiler.cm.new_source_file(FileName::Real(resource_path.clone()), content.clone());
          let comments = SingleThreadedComments::default();
          let transform_options = SwcPluginOptions {
            keep_export: None,
            remove_export: None,
          };
          let out = compiler.process_js_with_custom_pass(
            fm,
            None,
            handler,
            &swc_options,
            comments, |_| {
              transform(transform_options)
            },
            |_| noop(),
          )?;
          Ok(())
        })
      })
    })?;
    
    Ok(())
  }
}

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}