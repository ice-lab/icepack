use std::collections::HashSet;

use swc_core::{
  common::DUMMY_SP,
  ecma::{ast::*, visit::Fold},
};

pub struct TransformConfig {
  pub packages: Vec<String>,
}

pub struct NamedImportTransform {
  pub packages: Vec<String>,
}

pub fn named_import_transform(config: TransformConfig) -> impl Fold {
  NamedImportTransform {
    packages: config.packages,
  }
}

impl Fold for NamedImportTransform {
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
          // Add unique query string to avoid loader cache.
          "__barrel_optimize__?names={}!=!{}?{}",
          names.join(","),
          src_value,
          names.join(","),
        );

        // Create a new import declaration, keep everything the same except the source
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
