use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_error::{
  internal_error, Diagnostic, DiagnosticKind, Error, InternalError, Result, Severity,
  TraceableError,
};

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
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Err(internal_error!("No content found"));
    };
    let mut source = content.try_into_string()?;
    source += r#"
window.__custom_code__ = true;
"#;
    loader_context.content = Some(source.into());
    Ok(())
  }
}

pub const COMPILATION_LOADER_IDENTIFIER: &str = "builtin:compilation-loader";

impl Identifiable for CompilationLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}