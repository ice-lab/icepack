/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/5dacaa174baaf6bf40594d79d14884c8c2fc0de2/crates/swc/src/lib.rs
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
use std::env;
use std::fs::File;
use std::{path::PathBuf, sync::Arc};

use anyhow::{bail, Context, Error};
use base64::prelude::*;
use rspack_ast::javascript::{Ast as JsAst, Context as JsAstContext, Program as JsProgram};
use rspack_util::swc::minify_file_comments;
use swc::config::JsMinifyCommentOption;
use swc::BoolOr;
use swc_core::base::config::{
  BuiltInput, Config, InputSourceMap, IsModule
};
use swc_core::base::{sourcemap, SwcComments};
use swc_core::common::comments::Comments;
use swc_core::common::errors::{Handler, HANDLER};
use swc_core::common::SourceFile;
use swc_core::common::{
  comments::SingleThreadedComments, FileName, FilePathMapping, Mark, SourceMap, GLOBALS,
};
use swc_core::ecma::ast::{EsVersion, Pass, Program};
use swc_core::ecma::parser::{
  parse_file_as_module, parse_file_as_program, parse_file_as_script, Syntax,
};
use swc_core::ecma::transforms::base::helpers::{self, Helpers};
use swc_core::{
  base::{config::Options, try_with_handler},
  common::Globals,
};
use url::Url;
pub struct SwcCompiler {
  cm: Arc<SourceMap>,
  fm: Arc<SourceFile>,
  comments: SingleThreadedComments,
  options: Options,
  globals: Globals,
  helpers: Helpers,
  config: Config,
}

impl SwcCompiler {
  fn parse_js(
    &self,
    fm: Arc<SourceFile>,
    handler: &Handler,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<&dyn Comments>,
  ) -> Result<Program, Error> {
    let mut error = false;

    let mut errors = vec![];
    let program_result = match is_module {
      IsModule::Bool(true) => {
        parse_file_as_module(&fm, syntax, target, comments, &mut errors).map(Program::Module)
      }
      IsModule::Bool(false) => {
        parse_file_as_script(&fm, syntax, target, comments, &mut errors).map(Program::Script)
      }
      IsModule::Unknown => parse_file_as_program(&fm, syntax, target, comments, &mut errors),
    };

    for e in errors {
      e.into_diagnostic(handler).emit();
      error = true;
    }

    let mut res = program_result.map_err(|e| {
      e.into_diagnostic(handler).emit();
      Error::msg("Syntax Error")
    });

    if error {
      return Err(anyhow::anyhow!("Syntax Error"));
    }

    if env::var("SWC_DEBUG").unwrap_or_default() == "1" {
      res = res.with_context(|| format!("Parser config: {:?}", syntax));
    }

    res
  }
}

impl SwcCompiler {
  pub fn new(resource_path: PathBuf, source: String, mut options: Options) -> Result<Self, Error> {
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let globals = Globals::default();
    GLOBALS.set(&globals, || {
      let top_level_mark = Mark::new();
      let unresolved_mark = Mark::new();
      options.top_level_mark = Some(top_level_mark);
      options.unresolved_mark = Some(unresolved_mark);
    });

    let fm = cm.new_source_file(Arc::new(FileName::Real(resource_path)), source);
    let comments = SingleThreadedComments::default();
    let config = options.config.clone();

    let helpers = GLOBALS.set(&globals, || {
      let external_helpers = options.config.jsc.external_helpers;
      Helpers::new(external_helpers.into())
    });

    Ok(Self {
      cm,
      fm,
      comments,
      options,
      globals,
      helpers,
      config,
    })
  }

  pub fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.globals, op)
  }

  pub fn parse<'a, P>(
    &'a self,
    program: Option<Program>,
    before_pass: impl FnOnce(&Program) -> P + 'a,
  ) -> Result<BuiltInput<impl Pass + 'a>, Error>
  where
    P: Pass + 'a,
  {
    let built = self.run(|| {
      try_with_handler(self.cm.clone(), Default::default(), |handler| {
        let built = self.options.build_as_input(
          &self.cm,
          &self.fm.name,
          move |syntax, target, is_module| match program {
            Some(v) => Ok(v),
            _ => self.parse_js(
              self.fm.clone(),
              handler,
              target,
              syntax,
              is_module,
              Some(&self.comments),
            ),
          },
          self.options.output_path.as_deref(),
          self.options.source_root.clone(),
          self.options.source_file_name.clone(),
          handler,
          Some(self.config.clone()),
          Some(&self.comments),
          before_pass,
        )?;

        Ok(Some(built))
      })
    })?;

    match built {
      Some(v) => Ok(v),
      None => {
        anyhow::bail!("cannot process file because it's ignored by .swcrc")
      }
    }
  }

  pub fn transform(&self, config: BuiltInput<impl Pass>) -> Result<Program, Error> {
    let program = config.program;
    let mut pass = config.pass;

    let program = self.run(|| {
      helpers::HELPERS.set(&self.helpers, || {
        try_with_handler(self.cm.clone(), Default::default(), |handler| {
          HANDLER.set(handler, || Ok(program.apply(&mut pass)))
        })
      })
    });

    if let Some(comments) = &config.comments {
      // TODO: Wait for https://github.com/swc-project/swc/blob/e6fc5327b1a309eae840fe1ec3a2367adab37430/crates/swc/src/config/mod.rs#L808 to land.
      let preserve_annotations = match &config.preserve_comments {
        BoolOr::Bool(true) | BoolOr::Data(JsMinifyCommentOption::PreserveAllComments) => true,
        BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments) => false,
        BoolOr::Bool(false) => false,
      };

      minify_file_comments(comments, config.preserve_comments, preserve_annotations);
    };

    program
  }

  pub fn input_source_map(
    &self,
    input_src_map: &InputSourceMap,
  ) -> Result<Option<sourcemap::SourceMap>, Error> {
    let fm = &self.fm;
    let name = &self.fm.name;

    let read_inline_sourcemap =
      |data_url: Option<&str>| -> Result<Option<sourcemap::SourceMap>, Error> {
        match data_url {
          Some(data_url) => {
            let url = Url::parse(data_url)
              .with_context(|| format!("failed to parse inline source map url\n{}", data_url))?;

            let idx = match url.path().find("base64,") {
              Some(v) => v,
              None => {
                bail!("failed to parse inline source map: not base64: {:?}", url)
              }
            };

            let content = url.path()[idx + "base64,".len()..].trim();

            let res = BASE64_STANDARD
              .decode(content.as_bytes())
              .context("failed to decode base64-encoded source map")?;

            Ok(Some(sourcemap::SourceMap::from_slice(&res).context(
              "failed to read input source map from inlined base64 encoded \
                                 string",
            )?))
          }
          None => {
            bail!("failed to parse inline source map: `sourceMappingURL` not found")
          }
        }
      };

    let read_file_sourcemap =
      |data_url: Option<&str>| -> Result<Option<sourcemap::SourceMap>, Error> {
        match name.as_ref() {
          FileName::Real(filename) => {
            let dir = match filename.parent() {
              Some(v) => v,
              None => {
                bail!("unexpected: root directory is given as a input file")
              }
            };

            let map_path = match data_url {
              Some(data_url) => {
                let mut map_path = dir.join(data_url);
                if !map_path.exists() {
                  // Old behavior. This check would prevent
                  // regressions.
                  // Perhaps it shouldn't be supported. Sometimes
                  // developers don't want to expose their source
                  // code.
                  // Map files are for internal troubleshooting
                  // convenience.
                  map_path = PathBuf::from(format!("{}.map", filename.display()));
                  if !map_path.exists() {
                    bail!(
                      "failed to find input source map file {:?} in \
                                                 {:?} file",
                      map_path.display(),
                      filename.display()
                    )
                  }
                }

                Some(map_path)
              }
              None => {
                // Old behavior.
                let map_path = PathBuf::from(format!("{}.map", filename.display()));
                if map_path.exists() {
                  Some(map_path)
                } else {
                  None
                }
              }
            };

            match map_path {
              Some(map_path) => {
                let path = map_path.display().to_string();
                let file = File::open(&path);

                // Old behavior.
                let file = file?;

                Ok(Some(sourcemap::SourceMap::from_reader(file).with_context(
                  || {
                    format!(
                      "failed to read input source map
                                from file at {}",
                      path
                    )
                  },
                )?))
              }
              None => Ok(None),
            }
          }
          _ => Ok(None),
        }
      };

    let read_sourcemap = || -> Option<sourcemap::SourceMap> {
      let s = "sourceMappingURL=";
      let idx = fm.src.rfind(s);

      let data_url = idx.map(|idx| {
        let data_idx = idx + s.len();
        if let Some(end) = fm.src[data_idx..].find('\n').map(|i| i + data_idx + 1) {
          &fm.src[data_idx..end]
        } else {
          &fm.src[data_idx..]
        }
      });

      match read_inline_sourcemap(data_url) {
        Ok(r) => r,
        Err(_err) => {
          // Load original source map if possible
          read_file_sourcemap(data_url).unwrap_or(None)
        }
      }
    };

    // Load original source map
    match input_src_map {
      InputSourceMap::Bool(false) => Ok(None),
      InputSourceMap::Bool(true) => Ok(read_sourcemap()),
      InputSourceMap::Str(ref s) => {
        if s == "inline" {
          Ok(read_sourcemap())
        } else {
          // Load source map passed by user
          Ok(Some(
            sourcemap::SourceMap::from_slice(s.as_bytes())
              .context("failed to read input source map from user-provided sourcemap")?,
          ))
        }
      }
    }
  }
}

pub trait IntoJsAst {
  fn into_js_ast(self, program: Program) -> JsAst;
}

impl IntoJsAst for SwcCompiler {
  fn into_js_ast(self, program: Program) -> JsAst {
    JsAst::default()
      .with_program(JsProgram::new(
        program,
        Some(self.comments.into_swc_comments()),
      ))
      .with_context(JsAstContext {
        globals: self.globals,
        helpers: self.helpers.data(),
        source_map: self.cm,
        top_level_mark: self
          .options
          .top_level_mark
          .expect("`top_level_mark` should be initialized"),
        unresolved_mark: self
          .options
          .unresolved_mark
          .expect("`unresolved_mark` should be initialized"),
      })
  }
}

trait IntoSwcComments {
  fn into_swc_comments(self) -> SwcComments;
}

impl IntoSwcComments for SingleThreadedComments {
  fn into_swc_comments(self) -> SwcComments {
    let (l, t) = {
      let (l, t) = self.take_all();
      (l.take(), t.take())
    };
    SwcComments {
      leading: Arc::new(FromIterator::<_>::from_iter(l)),
      trailing: Arc::new(FromIterator::<_>::from_iter(t)),
    }
  }
}