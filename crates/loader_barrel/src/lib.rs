use std::{sync::Mutex, collections::{HashMap, HashSet}, path::PathBuf};
use lazy_static::lazy_static;
use rspack_core::{
  DependencyCategory, ResolveOptionsWithDependencyType, LoaderRunnerContext
};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext, Content};
use rspack_error::{internal_error, AnyhowError, Result};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
use regex::Regex;
use swc_core::{
  base::config::{Options, OutputCharset},
  ecma::{
    parser::{Syntax, TsConfig},
    ast::EsVersion,
  }
};
use swc_compiler::{SwcCompiler, IntoJsAst};
use swc_optimize_barrel::optimize_barrel;

pub const BARREL_LOADER_IDENTIFIER: &str = "builtin:barrel-loader";

pub struct LoaderOptions {
  pub names: Vec<String>,
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

struct TransfromMapping {
  pub export_list: Vec<Vec<String>>,
  pub wildcard_exports: Vec<String>,
  pub is_client_entry: bool,
}

lazy_static! {
  static ref TRANSFORM_MAPPING: Mutex<HashMap<String, TransfromMapping>> = Mutex::new(HashMap::new());
}

async fn get_matches(file: PathBuf, is_wildcard: bool) -> Result<Option<TransfromMapping>> {
  // if visited.contains(&file) {
  //   return Ok(None);
  // }
  // visited.insert(file.clone());
  let result = tokio::fs::read(file.clone())
      .await
      .map_err(|e| {
        internal_error!(e.to_string())
      })?;
  let content = Content::from(result).try_into_string()?;
  let mut swc_options = Options {
    ..Default::default()
  };
  swc_options.config.jsc.target = Some(EsVersion::EsNext);
  let file_extension = file.extension().unwrap();
  let ts_extensions = vec!["tsx", "ts", "mts"];
  if ts_extensions.iter().any(|ext| ext == &file_extension) {
    swc_options.config.jsc.syntax = Some(Syntax::Typescript(TsConfig { tsx: true, decorators: true, ..Default::default() }));
  }
  let c = SwcCompiler::new(file.clone(), content, swc_options).map_err(AnyhowError::from)?;
  let built = c.parse(None, |_| {
    optimize_barrel(swc_optimize_barrel::Config { wildcard: is_wildcard })
  }).map_err(AnyhowError::from)?;
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
  let TransformOutput { code, map:_ } = ast::stringify(&ast, codegen_options)?;
  let regex = Regex::new(r#"^(.|\n)*export (const|var) __next_private_export_map__ = ('[^']+'|\"[^\"]+\")"#).unwrap();
  if let Some(captures) = regex.captures(&code) {
    let directive_regex = Regex::new(r#"^(.|\n)*export (const|var) __next_private_directive_list__ = '([^']+)'"#).unwrap();
    let directive_list: Vec<String> = if let Some(directive_captures) = directive_regex.captures(&code) {
      let matched_str = directive_captures.get(3).unwrap().as_str().to_string();
      serde_json::from_str::<Vec<String>>(&matched_str).unwrap()
    } else {
      vec![]
    };
    let is_client_entry = directive_list.iter().any(|directive| directive.contains("use client"));
    let export_str = captures.get(3).unwrap().as_str().to_string();
    let slice_str = &export_str[1..export_str.len() - 1];
    let format_list = serde_json::from_str::<Vec<Vec<String>>>(slice_str).unwrap();
    let wildcard_regex = Regex::new(r#"export \* from "([^"]+)""#).unwrap();
    let wildcard_exports = wildcard_regex.captures_iter(&code).filter_map(|cap| cap.get(1)).map(|m| m.as_str().to_owned()).collect::<Vec<String>>();
    
    let export_list = format_list.iter().map(|list| {
      if is_wildcard {
        vec![list[0].clone(), file.clone().to_string_lossy().to_string(), list[0].clone()]
      } else {
        list.clone()
      }
    }).collect::<Vec<Vec<String>>>();

    if wildcard_exports.len() > 0 {
      println!("wildcard_exports is not support yet: {:?}", wildcard_exports);
    }

    return Ok(Some(TransfromMapping {
      export_list,
      wildcard_exports,
      is_client_entry,
    }));
  }
  Ok(None)
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for BarrelLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    
    let resource_path: std::path::PathBuf = loader_context.resource_path.to_path_buf();

    let resolver = loader_context.context.resolver_factory.get(ResolveOptionsWithDependencyType {
      resolve_options: None,
      resolve_to_context: false,
      dependency_category: DependencyCategory::Esm,
    });
    let parent_path = resource_path.parent().unwrap();
    // let test_path = resolver.resolve(parent_path, "./output")
    //   .map_err(|err| {
    //     internal_error!("Failed to resolve {err:?}")
    //   })?;
    // Get barrel mapping
    // let mut tranfrom_mapping = TRANSFORM_MAPPING.lock().unwrap();
    // let mut mapping = tranfrom_mapping.get(&resource_path.to_string_lossy().to_string());

    // if mapping.is_none() {
    let result = get_matches(resource_path.clone(), false).await?;
    let mut export_map = HashMap::new();
    if let Some(TransfromMapping { export_list, wildcard_exports , is_client_entry  }) = result {
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
            output.push_str(&format!("\nexport {{ default as {} }} from '{}';", n, file_path));
          } else if orig == n {
            output.push_str(&format!("\nexport {{ {} }} from '{}';", n, file_path));
          } else {
            output.push_str(&format!("\nexport {{ {} as {} }} from '{}';", orig, n, file_path));
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
      let reexport_str = format!("export * from '{}';", resource_path.to_string_lossy().to_string());
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