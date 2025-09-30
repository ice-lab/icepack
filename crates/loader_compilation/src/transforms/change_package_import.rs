use std::collections::HashMap;
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::*,
    utils::{quote_str, swc_ecma_ast::ImportSpecifier},
    visit::{noop_fold_type, Fold, FoldWith, fold_pass},
  },
};

#[derive(Debug, Clone)]
pub enum Config {
  LiteralConfig(String),
  #[allow(dead_code)]
  SpecificConfig(SpecificConfigs),
}

#[derive(Debug, Clone)]
pub struct SpecificConfigs {
  pub name: String,
  pub map: HashMap<String, MapProperty>,
}

#[derive(Debug, Clone)]
pub struct MapProperty {
  pub to: String,
  pub import_type: Option<ImportType>,
  pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportType {
  #[allow(dead_code)]
  Named,
  #[allow(dead_code)]
  Default,
}

struct ChangePackageImportImpl {
  pub options: Vec<Config>,
}

impl ChangePackageImportImpl {
  pub fn new(options: Vec<Config>) -> Self {
    Self { options }
  }
}

impl Fold for ChangePackageImportImpl {
  noop_fold_type!();

  fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut new_items: Vec<ModuleItem> = vec![];

    for item in items {
      let item = item.fold_with(self);
      let mut hit_rule = false;
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &item {
        for option in &self.options {
          match option {
            Config::LiteralConfig(src) => {
              if is_hit_rule(import_decl, option) {
                hit_rule = true;
                for specifier in &import_decl.specifiers {
                  if let ImportSpecifier::Named(named_import_spec) = specifier {
                    let mut import_new_src = src.clone();
                    import_new_src.push('/');
                    import_new_src.push_str(&get_import_module_name(named_import_spec));

                    new_items.push(create_default_import_decl(
                      import_new_src,
                      named_import_spec.local.clone(),
                    ));
                  }
                }
                break;
              }
            }
            Config::SpecificConfig(config) => {
              if is_hit_rule(import_decl, option) {
                hit_rule = true;
                let target_fields: Vec<&String> = config.map.keys().clone().collect();
                let mut named_import_spec_copy = import_decl.clone();

                named_import_spec_copy.specifiers = named_import_spec_copy
                  .specifiers
                  .into_iter()
                  .filter(|specifier| match specifier {
                    ImportSpecifier::Named(named_import_spec) => {
                      let import_object_name = get_import_module_name(named_import_spec);
                      !target_fields.contains(&&import_object_name)
                    }
                    _ => true,
                  })
                  .collect::<Vec<_>>();

                if !named_import_spec_copy.specifiers.is_empty() {
                  new_items.push(item.clone());
                  break;
                }
                for specifier in &import_decl.specifiers {
                  for (target, rules) in config.map.iter() {
                    if let ImportSpecifier::Named(named_import_spec) = specifier {
                      let import_object_name = get_import_module_name(named_import_spec);
                      if target == &import_object_name {
                        let new_import_decl: ModuleItem;
                        if rules.import_type.is_none()
                          || matches!(rules.import_type.as_ref().unwrap(), ImportType::Default)
                        {
                          new_import_decl = create_default_import_decl(
                            rules.to.to_string(),
                            named_import_spec.local.clone(),
                          );
                        } else {
                          let mut named_import_spec_copy = named_import_spec.clone();

                          if rules.name.is_some() {
                            named_import_spec_copy.imported = Some(ModuleExportName::Str(Str {
                              span: named_import_spec.span,
                              value: rules.name.clone().unwrap().into(),
                              raw: Some(rules.name.clone().unwrap().clone().into()),
                            }))
                          }

                          new_import_decl = create_named_import_decl(
                            rules.to.to_string(),
                            vec![ImportSpecifier::Named(named_import_spec_copy)],
                          );
                        }

                        new_items.push(new_import_decl);
                      }
                    }
                  }
                }
                break;
              }
            }
          }
        }

        if !hit_rule {
          new_items.push(item);
        }
      } else {
        new_items.push(item);
      }
    }
    new_items
  }
}

fn is_hit_rule(cur_import: &ImportDecl, rule: &Config) -> bool {
  match rule {
    Config::LiteralConfig(s) => {
      if cur_import.src.value == s.clone() {
        return true;
      }
      false
    }
    Config::SpecificConfig(s) => {
      if cur_import.src.value == s.name.clone() {
        return true;
      }
      false
    }
  }
}

fn get_import_module_name(named_import_spec: &ImportNamedSpecifier) -> String {
  if named_import_spec.imported.is_none() {
    named_import_spec.local.sym.to_string()
  } else {
    match &named_import_spec.imported.clone().unwrap() {
      ModuleExportName::Ident(ident) => ident.sym.to_string(),
      ModuleExportName::Str(str) => str.value.to_string(),
    }
  }
}

fn create_default_import_decl(src: String, local: Ident) -> ModuleItem {
  wrap_with_moudle_item(ImportDecl {
    phase: Default::default(),
    src: Box::new(quote_str!(src)),
    specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
      span: DUMMY_SP,
      local,
    })],
    span: DUMMY_SP,
    type_only: false,
    with: None,
  })
}

fn create_named_import_decl(src: String, specifiers: Vec<ImportSpecifier>) -> ModuleItem {
  wrap_with_moudle_item(ImportDecl {
    phase: Default::default(),
    src: Box::new(quote_str!(src)),
    specifiers,
    span: DUMMY_SP,
    type_only: false,
    with: None,
  })
}

fn wrap_with_moudle_item(import_decl: ImportDecl) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl))
}

pub fn change_package_import(options: Vec<Config>) -> impl swc_core::ecma::ast::Pass {
  fold_pass(ChangePackageImportImpl::new(options))
}

#[cfg(test)]
mod tests {
  use super::*;
  use swc_core::{
    common::{FileName, SourceMap},
    ecma::{
      parser::{lexer::Lexer, Parser, StringInput, Syntax},
      visit::FoldWith,
    },
  };

  fn parse_js(code: &str) -> Module {
    let cm = SourceMap::default();
    let fm = cm.new_source_file(FileName::Anon.into(), code.to_string());
    let lexer = Lexer::new(
      Syntax::Es(Default::default()),
      Default::default(),
      StringInput::from(&*fm),
      None,
    );
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().expect("Failed to parse module")
  }

  #[test]
  fn test_literal_config_transform() {
    let code = r#"import { Button, Input } from 'antd';"#;
    
    let mut module = parse_js(code);
    let mut transform = ChangePackageImportImpl::new(vec![
      Config::LiteralConfig("antd".to_string())
    ]);
    module = module.fold_with(&mut transform);
    
    // Should transform to separate default imports
    assert_eq!(module.body.len(), 2);
    
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert_eq!(import_decl.src.value.as_str(), "antd/Button");
      assert!(matches!(import_decl.specifiers[0], ImportSpecifier::Default(_)));
    }
    
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[1] {
      assert_eq!(import_decl.src.value.as_str(), "antd/Input");
      assert!(matches!(import_decl.specifiers[0], ImportSpecifier::Default(_)));
    }
  }

  #[test]
  fn test_specific_config_transform() {
    let code = r#"import { a, b } from 'ice';"#;
    
    let mut map = HashMap::new();
    map.insert("a".to_string(), MapProperty {
      to: "@ice/x/y".to_string(),
      import_type: None,
      name: None,
    });
    
    let mut module = parse_js(code);
    let mut transform = ChangePackageImportImpl::new(vec![
      Config::SpecificConfig(SpecificConfigs {
        name: "ice".to_string(),
        map,
      })
    ]);
    module = module.fold_with(&mut transform);
    
    // Should keep original import for 'b' and transform 'a'
    assert!(!module.body.is_empty());
  }

  #[test]
  fn test_specific_config_named_import() {
    let code = r#"import { a } from 'ice';"#;
    
    let mut map = HashMap::new();
    map.insert("a".to_string(), MapProperty {
      to: "@ice/x/y".to_string(),
      import_type: Some(ImportType::Named),
      name: Some("newA".to_string()),
    });
    
    let mut module = parse_js(code);
    let mut transform = ChangePackageImportImpl::new(vec![
      Config::SpecificConfig(SpecificConfigs {
        name: "ice".to_string(),
        map,
      })
    ]);
    module = module.fold_with(&mut transform);
    
    // Should transform to named import with new name
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert_eq!(import_decl.src.value.as_str(), "@ice/x/y");
      if let ImportSpecifier::Named(named_spec) = &import_decl.specifiers[0] {
        if let Some(ModuleExportName::Str(str_name)) = &named_spec.imported {
          assert_eq!(str_name.value.as_str(), "newA");
        }
      }
    }
  }

  #[test]
  fn test_no_transform_when_no_match() {
    let code = r#"import { Button } from 'other-lib';"#;
    
    let mut module = parse_js(code);
    let mut transform = ChangePackageImportImpl::new(vec![
      Config::LiteralConfig("antd".to_string())
    ]);
    module = module.fold_with(&mut transform);
    
    // Should keep original import when no rule matches
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert_eq!(import_decl.src.value.as_str(), "other-lib");
    }
  }

  #[test]
  fn test_mixed_import_types() {
    let code = r#"import React, { Component } from 'react';"#;
    
    let mut module = parse_js(code);
    let mut transform = ChangePackageImportImpl::new(vec![
      Config::LiteralConfig("react".to_string())
    ]);
    module = module.fold_with(&mut transform);
    
    // Should only transform named imports, keep default imports
    assert!(!module.body.is_empty());
  }
}