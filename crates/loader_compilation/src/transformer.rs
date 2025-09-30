use swc_core::ecma::ast::Pass;
use crate::options::{TransformFeatures, ChangeConfig};
use crate::transforms::{
  env_replacement::env_replacement,
  keep_export::keep_export,
  remove_export::remove_export,
  named_import_transform::{named_import_transform, TransformConfig},
  change_package_import::{change_package_import, Config},
};

pub(crate) fn transform(transform_features: &TransformFeatures) -> impl Pass + '_ {
  // Chain transforms based on enabled features
  let mut passes: Vec<Box<dyn Pass>> = Vec::new();
  
  if let Some(sources) = &transform_features.env_replacement {
    passes.push(Box::new(env_replacement(sources.clone())));
  }
  
  if let Some(exports) = &transform_features.keep_export {
    passes.push(Box::new(keep_export(exports.clone())));
  }
  
  if let Some(exports) = &transform_features.remove_export {
    passes.push(Box::new(remove_export(exports.clone())));
  }
  
  if let Some(config) = &transform_features.named_import_transform {
    passes.push(Box::new(named_import_transform(TransformConfig {
      packages: config.packages.clone(),
    })));
  }
  
  if let Some(configs) = &transform_features.change_package_import {
    let change_configs: Vec<Config> = configs.iter().map(|c| match c {
      ChangeConfig::LiteralConfig(s) => Config::LiteralConfig(s.clone()),
    }).collect();
    passes.push(Box::new(change_package_import(change_configs)));
  }
  
  ChainedTransform { passes }
}

struct ChainedTransform {
  passes: Vec<Box<dyn Pass>>,
}

impl Pass for ChainedTransform {
  fn process(&mut self, program: &mut swc_core::ecma::ast::Program) {
    for pass in &mut self.passes {
      pass.process(program);
    }
  }
}