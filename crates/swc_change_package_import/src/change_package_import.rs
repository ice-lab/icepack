use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::*,
    atoms::JsWord,
    utils::{quote_str, swc_ecma_ast::ImportSpecifier},
    visit::{noop_fold_type, Fold, FoldWith, fold_pass},
  },
};

use crate::config::{Config, ImportType};

pub struct ModuleImportVisitor {
  // 用户配置
  pub options: Vec<Config>,
}

pub fn change_package_import(options: Vec<Config>) -> impl Pass {
  fold_pass(ModuleImportVisitor::new(options))
}

impl ModuleImportVisitor {
  pub fn new(options: Vec<Config>) -> Self {
    Self { options }
  }
}

impl Fold for ModuleImportVisitor {
  noop_fold_type!();

  fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut new_items: Vec<ModuleItem> = vec![];

    for item in items {
      let item = item.fold_children_with(self);
      let mut hit_rule = false;
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &item {
        for option in &self.options {
          match option {
            Config::LiteralConfig(src) => {
              if is_hit_rule(import_decl, option) {
                hit_rule = true;
                for specifier in &import_decl.specifiers {
                  match specifier {
                    ImportSpecifier::Named(named_import_spec) => {
                      let mut import_new_src = src.clone();
                      import_new_src.push_str("/");
                      import_new_src.push_str(&get_import_module_name(named_import_spec));

                      new_items.push(create_default_import_decl(
                        import_new_src,
                        named_import_spec.local.clone(),
                      ));
                    }
                    _ => (),
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

                if named_import_spec_copy.specifiers.len() != 0 {
                  // It no need to redirect import source, if some specifiers are not configured.
                  new_items.push(item.clone());
                  break;
                }
                for specifier in &import_decl.specifiers {
                  for (target, rules) in config.map.iter() {
                    match specifier {
                      ImportSpecifier::Named(named_import_spec) => {
                        let import_object_name = get_import_module_name(named_import_spec);
                        if target == &import_object_name {
                          let new_import_decl: ModuleItem;
                          if rules.import_type.is_none()
                            || match rules.import_type.as_ref().unwrap() {
                              ImportType::Default => true,
                              _ => false,
                            }
                          {
                            // Default import mode
                            new_import_decl = create_default_import_decl(
                              rules.to.to_string(),
                              named_import_spec.local.clone(),
                            );
                          } else {
                            // Named import mode
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
                      _ => (),
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
      if cur_import.src.value == JsWord::from(s.clone()) {
        return true;
      }
      false
    }
    Config::SpecificConfig(s) => {
      if cur_import.src.value == JsWord::from(s.name.clone()) {
        return true;
      }
      false
    }
  }
}

fn get_import_module_name(named_import_spec: &ImportNamedSpecifier) -> String {
  if named_import_spec.imported.is_none() {
    (&named_import_spec.local.sym).to_string()
  } else {
    match &named_import_spec.imported.clone().unwrap() {
      ModuleExportName::Ident(ident) => (&ident.sym).to_string(),
      ModuleExportName::Str(str) => (&str.value).to_string(),
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
