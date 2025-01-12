use std::path::Path;
use anyhow::{Context, Error};
use either::Either;
use serde::Deserialize;
use swc_core::atoms::Atom;
use swc_core::common::collections::AHashMap;
use swc_core::common::BytePos;
use swc_core::ecma::ast::{Ident, Pass, noop_pass};
use swc_core::ecma::visit::{noop_visit_type, Visit};
use swc_env_replacement::env_replacement;
use swc_keep_export::keep_export;
use swc_named_import_transform::{named_import_transform, TransformConfig};
use swc_remove_export::remove_export;
use swc_change_package_import::{change_package_import, Config as ImportConfig, SpecificConfigs};

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
      Either::Left($f(config))
    } else {
      Either::Right(noop_pass())
    }
  };
  ($config:expr, $f:expr, $enabled:expr) => {
    if $enabled() {
      either!($config, $f)
    } else {
      Either::Right(noop_pass())
    }
  };
}

// Only define the stuct which is used in the following function.
#[derive(Deserialize, Debug)]
struct NestedRoutesManifest {
  file: String,
  children: Option<Vec<NestedRoutesManifest>>,
}

fn get_routes_file(routes: Vec<NestedRoutesManifest>) -> Vec<String> {
  let mut result: Vec<String> = vec![];
  for route in routes {
    // Add default prefix of src/pages/ to the route file.
    let mut path_str = String::from("src/pages/");
    path_str.push_str(&route.file);

    result.push(path_str.to_string());

    if let Some(children) = route.children {
      result.append(&mut get_routes_file(children));
    }
  }
  result
}

fn parse_routes_config(c: String) -> Result<Vec<String>, Error> {
  let routes = serde_json::from_str(&c)?;
  Ok(get_routes_file(routes))
}

pub(crate) fn load_routes_config(path: &Path) -> Result<Vec<String>, Error> {
  let content = std::fs::read_to_string(path).context("failed to read routes config")?;
  parse_routes_config(content)
}

fn match_route_entry(resource_path: &str, routes: Option<&Vec<String>>) -> bool {
  if let Some(routes) = routes {
    for route in routes {
      if resource_path.ends_with(&route.to_string()) {
        return true;
      }
    }
  }
  false
}

fn match_app_entry(resource_path: &str) -> bool {
  // File path ends with src/app.(ts|tsx|js|jsx)
  let regex_for_app = regex::Regex::new(r"src/app\.(ts|tsx|js|jsx)$").unwrap();
  regex_for_app.is_match(resource_path)
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TransformFeatureOptions {
  pub keep_export: Option<Vec<String>>,
  pub remove_export: Option<Vec<String>>,
  pub optimize_import: Option<Vec<String>>,
  pub import_config: Option<Vec<SpecificConfigs>>,
}

pub(crate) fn transform<'a>(
  resource_path: &'a str,
  routes_config: Option<&Vec<String>>,
  feature_options: &TransformFeatureOptions,
) -> impl Pass + 'a {
  (
    either!(feature_options.optimize_import, |options: &Vec<String>| {
      named_import_transform(TransformConfig {
        packages: options.clone(),
      })
    }),
    either!(
      feature_options.import_config,
      |options: &Vec<SpecificConfigs>| {
        let import_config = options.to_vec();
        change_package_import(import_config.into_iter().map(ImportConfig::SpecificConfig).collect())
      }
    ),
    either!(
      Some(&vec!["@uni/env".to_string(), "universal-env".to_string()]),
      |options: &Vec<String>| { env_replacement(options.clone()) }
    ),
    either!(
      feature_options.keep_export,
      |options: &Vec<String>| {
        let mut exports_name = options.clone();
        // Special case for app entry.
        // When keep pageConfig, we should also keep the default export of app entry.
        if match_app_entry(resource_path) && exports_name.contains(&String::from("pageConfig")) {
          exports_name.push(String::from("default"));
        }
        keep_export(exports_name)
      },
      || { match_app_entry(resource_path) || match_route_entry(resource_path, routes_config) }
    ),
    either!(
      feature_options.remove_export,
      |options: &Vec<String>| { remove_export(options.clone()) },
      || {
        // Remove export only work for app entry and route entry.
        match_app_entry(resource_path) || match_route_entry(resource_path, routes_config)
      }
    ),
  )
}

pub struct IdentCollector {
  pub names: AHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
