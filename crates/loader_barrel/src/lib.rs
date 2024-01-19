use std::future::Future;
use std::pin::Pin;
use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use lazy_static::lazy_static;
use regex::Regex;
use rspack_core::{
  DependencyCategory, LoaderRunnerContext, ResolveOptionsWithDependencyType, ResolveResult,
  Resolver,
};
use serde::Deserialize;
use rspack_error::{error, AnyhowError, Result};
use rspack_loader_runner::{Content, Identifiable, Identifier, Loader, LoaderContext};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
use swc_compiler::{IntoJsAst, SwcCompiler};
use swc_core::{
  base::config::{Options, OutputCharset},
  ecma::{
    ast::EsVersion,
    parser::{Syntax, TsConfig},
  },
};
use swc_optimize_barrel::optimize_barrel;
use tokio::sync::Mutex;

pub const BARREL_LOADER_IDENTIFIER: &str = "builtin:barrel-loader";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LoaderOptions {
  pub names: Vec<String>,
  pub cache_dir: Option<String>,
}

pub struct BarrelLoader {
  identifier: Identifier,
  loader_options: LoaderOptions,
}

impl BarrelLoader {
  pub fn new(options: LoaderOptions) -> Self {
    Self {
      identifier: BARREL_LOADER_IDENTIFIER.into(),
      loader_options: options.into(),
    }
  }

  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(BARREL_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

#[derive(Debug, Clone)]
struct TransformMapping {
  pub export_list: Vec<Vec<String>>,
  pub wildcard_exports: Vec<String>,
  pub is_client_entry: bool,
}

lazy_static! {
  static ref GLOBAL_TRANSFORM_MAPPING: Arc<Mutex<HashMap<String, TransformMapping>>> =
    Arc::new(Mutex::new(HashMap::new()));
}

async fn get_barrel_map(
  mut visited: HashSet<PathBuf>,
  resolver: Arc<Resolver>,
  file: PathBuf,
  cache_dir: Option<String>,
  is_wildcard: bool,
  source: Option<String>,
) -> Result<Option<TransformMapping>> {
  if visited.contains(&file) {
    return Ok(None);
  }
  visited.insert(file.clone());
  let content = if source.is_none() {
    let result = tokio::fs::read(file.clone())
      .await
      .map_err(|e| error!(e.to_string()))?;
    Content::from(result).try_into_string()?
  } else {
    source.unwrap()
  };
  let mut swc_options = Options {
    ..Default::default()
  };
  swc_options.config.jsc.target = Some(EsVersion::EsNext);
  let file_extension = file.extension().unwrap();
  let ts_extensions = vec!["tsx", "ts", "mts"];
  if ts_extensions.iter().any(|ext| ext == &file_extension) {
    swc_options.config.jsc.syntax = Some(Syntax::Typescript(TsConfig {
      tsx: true,
      decorators: true,
      ..Default::default()
    }));
  }
  swc_options.config.jsc.experimental.cache_root = cache_dir.clone();

  let code = {
    // Drop the block for SwcCompiler will create Rc.
    let c = SwcCompiler::new(file.clone(), content, swc_options).map_err(AnyhowError::from)?;
    let built = c
      .parse(None, |_| {
        optimize_barrel(swc_optimize_barrel::Config {
          wildcard: is_wildcard,
        })
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
        enable: false,
        inline_sources_content: false,
        emit_columns: false,
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };
    let program = c.transform(built).map_err(AnyhowError::from)?;
    let ast = c.into_js_ast(program);
    let TransformOutput { code, map: _ } = ast::stringify(&ast, codegen_options)?;
    code
  };
  let regex =
    Regex::new(r#"^(.|\n)*export (const|var) __next_private_export_map__ = ('[^']+'|\"[^\"]+\")"#)
      .unwrap();

  if let Some(captures) = regex.captures(&code) {
    let directive_regex =
      Regex::new(r#"^(.|\n)*export (const|var) __next_private_directive_list__ = '([^']+)'"#)
        .unwrap();
    let directive_list: Vec<String> =
      if let Some(directive_captures) = directive_regex.captures(&code) {
        let matched_str = directive_captures.get(3).unwrap().as_str().to_string();
        serde_json::from_str::<Vec<String>>(&matched_str).unwrap()
      } else {
        vec![]
      };
    let is_client_entry = directive_list
      .iter()
      .any(|directive| directive.contains("use client"));
    let export_str = captures.get(3).unwrap().as_str().to_string();
    let slice_str = &export_str[1..export_str.len() - 1];
    let format_list = serde_json::from_str::<Vec<Vec<String>>>(slice_str).unwrap();
    let wildcard_regex = Regex::new(r#"export \* from "([^"]+)""#).unwrap();
    let wildcard_exports = wildcard_regex
      .captures_iter(&code)
      .filter_map(|cap| cap.get(1))
      .map(|m| m.as_str().to_owned())
      .collect::<Vec<String>>();

    let mut export_list = format_list
      .iter()
      .map(|list| {
        if is_wildcard {
          vec![
            list[0].clone(),
            file.clone().to_string_lossy().to_string(),
            list[0].clone(),
          ]
        } else {
          list.clone()
        }
      })
      .collect::<Vec<Vec<String>>>();

    if wildcard_exports.len() > 0 {
      for req in &wildcard_exports {
        let real_req = req.replace("__barrel_optimize__?names=__PLACEHOLDER__!=!", "");
        let wildcard_resolve = resolver
          .resolve(&file.parent().unwrap().to_path_buf(), &real_req)
          .map_err(|err| error!("Failed to resolve {err:?}"))?;
        if let ResolveResult::Resource(resource) = wildcard_resolve {
          let res =
            get_barrel_map_boxed(visited.clone(), resolver.clone(), resource.path, cache_dir.clone(), true, None)
              .await?;
          if let Some(TransformMapping {
            export_list: sub_export_list,
            wildcard_exports: _,
            is_client_entry: _,
          }) = res
          {
            export_list.extend(sub_export_list);
          }
        }
      }
    }
    let ret = TransformMapping {
      export_list,
      wildcard_exports,
      is_client_entry,
    };
    return Ok(Some(ret));
  }
  return Ok(None);
}

// A boxed function that can be sent across threads
fn get_barrel_map_boxed(
  visited: HashSet<PathBuf>,
  resolver: Arc<Resolver>,
  file: PathBuf,
  cache_dir: Option<String>,
  is_wildcard: bool,
  source: Option<String>,
) -> Pin<Box<dyn Future<Output = Result<Option<TransformMapping>>> + Send>> {
  Box::pin(get_barrel_map(visited, resolver, file, cache_dir, is_wildcard, source))
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for BarrelLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let resource_path: std::path::PathBuf = loader_context.resource_path.to_path_buf();

    let resolver = loader_context
      .context
      .resolver_factory
      .get(ResolveOptionsWithDependencyType {
        resolve_options: None,
        resolve_to_context: false,
        dependency_category: DependencyCategory::Esm,
      });

    let content = std::mem::take(&mut loader_context.content).expect("content should be available");
    let source = content.try_into_string()?;

    let result = {
      // Get barrel mapping
      let mut mapping_result = GLOBAL_TRANSFORM_MAPPING.lock().await;
      let resource_key = &resource_path.to_string_lossy().to_string();
      if mapping_result.contains_key(resource_key) {
        mapping_result.get(resource_key).cloned()
      } else {
        let visited = HashSet::new();
        let ret = get_barrel_map(
          visited,
          resolver,
          resource_path.clone(),
          self.loader_options.cache_dir.clone(),
          false,
          Some(source),
        )
        .await?;
        if let Some(mapping) = ret.clone() {
          mapping_result.insert(resource_key.to_string(), mapping);
        }
        ret
      }
    };
    let mut export_map = HashMap::new();
    if let Some(TransformMapping {
      export_list,
      wildcard_exports,
      is_client_entry,
    }) = result
    {
      export_list.iter().for_each(|list| {
        let key = list[0].clone();
        let value = (list[1].clone(), list[2].clone());
        export_map.insert(key, value);
      });

      let names = &self.loader_options.names;
      let mut missed_names = vec![];
      let mut output = if is_client_entry {
        String::from("\"use client;\"\n")
      } else {
        String::from("")
      };
      names.iter().for_each(|n| {
        if export_map.contains_key(n) {
          let (file_path, orig) = export_map.get(n).unwrap();
          if orig == "*" {
            output.push_str(&format!("\nexport * as {} from '{}';", n, file_path));
          } else if orig == "default" {
            output.push_str(&format!(
              "\nexport {{ default as {} }} from '{}';",
              n, file_path
            ));
          } else if orig == n {
            output.push_str(&format!("\nexport {{ {} }} from '{}';", n, file_path));
          } else {
            output.push_str(&format!(
              "\nexport {{ {} as {} }} from '{}';",
              orig, n, file_path
            ));
          }
        } else {
          missed_names.push(n.as_str());
        }
      });

      if missed_names.len() > 0 {
        wildcard_exports.iter().for_each(|n| {
          let mut missed_str = String::from(&missed_names.join(" ,"));
          missed_str.push_str("&wildcard");
          let req = n.replace("__PLACEHOLDER__", &missed_str);
          output.push_str(&format!("\nexport * from '{}';", req));
        });
      }
      loader_context.content = Some(Content::from(output));
    } else {
      let reexport_str = format!(
        "export * from '{}';",
        resource_path.to_string_lossy().to_string()
      );
      loader_context.content = Some(Content::from(reexport_str));
    }
    Ok(())
  }
}

impl Identifiable for BarrelLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
