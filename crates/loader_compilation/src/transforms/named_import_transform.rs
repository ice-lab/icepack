use std::collections::HashSet;
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::*,
    visit::{Fold, fold_pass},
  },
};

pub struct TransformConfig {
  pub packages: Vec<String>,
}

pub struct NamedImportTransformImpl {
  pub packages: Vec<String>,
}

impl Fold for NamedImportTransformImpl {
  fn fold_import_decl(&mut self, decl: ImportDecl) -> ImportDecl {
    let src_value = decl.src.value.clone();
    if self.packages.iter().any(|p| src_value == *p) {
      let mut specifier_names = HashSet::new();
      let mut skip = false;
      for specifier in &decl.specifiers {
        match specifier {
          ImportSpecifier::Named(specifier) => {
            if let Some(imported) = &specifier.imported {
              match imported {
                ModuleExportName::Ident(ident) => {
                  specifier_names.insert(ident.sym.to_string());
                }
                ModuleExportName::Str(str) => {
                  specifier_names.insert(str.value.to_string());
                }
              }
            } else {
              specifier_names.insert(specifier.local.sym.to_string());
            }
          }
          ImportSpecifier::Default(_) => {
            skip = true;
          }
          ImportSpecifier::Namespace(_) => {
            skip = true;
          }
        }
      }
      if !skip {
        let mut names = specifier_names.into_iter().collect::<Vec<_>>();
        names.sort();

        let new_src = format!(
          "__barrel_optimize__?names={}!=!{}?{}",
          names.join(","),
          src_value,
          names.join(","),
        );

        let mut new_decl = decl.clone();
        new_decl.src = Box::new(Str {
          span: DUMMY_SP,
          value: new_src.into(),
          raw: None,
        });

        return new_decl;
      }
    }
    decl
  }
}

pub fn named_import_transform(config: TransformConfig) -> impl swc_core::ecma::ast::Pass {
  fold_pass(NamedImportTransformImpl {
    packages: config.packages,
  })
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
  fn test_named_import_transform_basic() {
    let code = r#"import { Button, Input } from 'antd';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["antd".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should transform to barrel optimization format
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert!(import_decl.src.value.contains("__barrel_optimize__"));
      assert!(import_decl.src.value.contains("Button,Input"));
    }
  }

  #[test]
  fn test_named_import_transform_with_default() {
    let code = r#"import React, { useState } from 'react';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["react".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should skip when default import is present
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert!(!import_decl.src.value.contains("__barrel_optimize__"));
    }
  }

  #[test]
  fn test_named_import_transform_with_namespace() {
    let code = r#"import * as React from 'react';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["react".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should skip when namespace import is present
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert!(!import_decl.src.value.contains("__barrel_optimize__"));
    }
  }

  #[test]
  fn test_named_import_transform_not_matching_package() {
    let code = r#"import { Button } from 'other-lib';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["antd".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should not transform non-matching packages
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert_eq!(import_decl.src.value.as_str(), "other-lib");
    }
  }

  #[test]
  fn test_named_import_transform_with_aliased_imports() {
    let code = r#"import { Button as AntdButton, Input as AntdInput } from 'antd';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["antd".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should handle aliased imports
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert!(import_decl.src.value.contains("__barrel_optimize__"));
      assert!(import_decl.src.value.contains("Button,Input"));
    }
  }

  #[test]
  fn test_named_import_transform_sorted_names() {
    let code = r#"import { Input, Button, Form } from 'antd';"#;
    
    let mut module = parse_js(code);
    let mut transform = NamedImportTransformImpl {
      packages: vec!["antd".to_string()],
    };
    module = module.fold_with(&mut transform);
    
    // Should sort the names alphabetically
    assert_eq!(module.body.len(), 1);
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
      assert!(import_decl.src.value.contains("Button,Form,Input"));
    }
  }
}