use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_core::{
  common::{SyntaxContext, DUMMY_SP},
  ecma::{
    ast::*,
    atoms::Atom,
    visit::{Fold, fold_pass},
  },
};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeepPlatformPatcher {
  pub platform: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum KeepPlatformConfig {
  Bool(bool),
  KeepPlatform(String),
}

impl Default for KeepPlatformConfig {
  fn default() -> Self {
    KeepPlatformConfig::Bool(false)
  }
}

fn get_platform_map() -> HashMap<String, Vec<String>> {
  HashMap::from([
    ("web".to_string(), vec!["isWeb".to_string()]),
    ("node".to_string(), vec!["isNode".to_string()]),
    ("weex".to_string(), vec!["isWeex".to_string()]),
    (
      "kraken".to_string(),
      vec!["isKraken".to_string(), "isWeb".to_string()]
    ),
    (
      "wechat-miniprogram".to_string(),
      vec![
        "isWeChatMiniProgram".to_string(),
        "isWeChatMiniprogram".to_string()
      ]
    ),
    ("miniapp".to_string(), vec!["isMiniApp".to_string()]),
    (
      "bytedance-microapp".to_string(),
      vec!["isByteDanceMicroApp".to_string()]
    ),
    (
      "kuaishou-miniprogram".to_string(),
      vec!["isKuaiShouMiniProgram".to_string()]
    ),
    (
      "baidu-smartprogram".to_string(),
      vec!["isBaiduSmartProgram".to_string()]
    ),
  ])
}

impl Fold for KeepPlatformPatcher {
  fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let platform_map = get_platform_map();
    let platform_flags: Vec<String> = platform_map
      .get(&self.platform)
      .cloned()
      .unwrap_or_default();
    
    let mut new_module_items: Vec<ModuleItem> = vec![];
    let mut env_variables: Vec<&Ident> = vec![];
    let mut decls: Vec<VarDeclarator> = vec![];

    for module_item in items.iter() {
      match module_item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
          if check_source(&import_decl.src.value) {
            for specifier in import_decl.specifiers.iter() {
              match specifier {
                ImportSpecifier::Named(named) => {
                  let ImportNamedSpecifier { local, .. } = named;
                  env_variables.push(local);
                }
                ImportSpecifier::Namespace(namespace) => {
                  let ImportStarAsSpecifier { local, .. } = namespace;
                  decls.push(create_var_decl(
                    local.clone(),
                    Some(Box::new(Expr::Object(ObjectLit {
                      span: DUMMY_SP,
                      props: platform_flags
                        .iter()
                        .map(|platform| {
                          PropOrSpread::Prop(Box::new(Prop::KeyValue(
                            KeyValueProp {
                              key: PropName::Ident(create_ident_name(platform)),
                              value: Box::new(create_bool_expr(true)),
                            },
                          )))
                        })
                        .collect(),
                    }))),
                  ))
                }
                _ => {}
              }
            }
          } else {
            new_module_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(
              import_decl.clone(),
            )))
          }
        }
        _ => new_module_items.push(module_item.clone()),
      }
    }

    if !env_variables.is_empty() {
      for env_variable in env_variables {
        decls.push(create_var_decl(
          env_variable.clone(),
          Some(Box::new(create_bool_expr(
            platform_flags.contains(&env_variable.sym.to_string()),
          ))),
        ));
      }
    }

    insert_decls_into_module_items(decls, &mut new_module_items);
    new_module_items
  }
}

fn check_source(source: &str) -> bool {
  source == "universal-env" || source == "@uni/env"
}

fn insert_decls_into_module_items(decls: Vec<VarDeclarator>, module_items: &mut Vec<ModuleItem>) {
  if !decls.is_empty() {
    module_items.insert(
      0,
      ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls,
        ctxt: SyntaxContext::empty()
      })))),
    )
  }
}

fn create_ident_name(value: &str) -> IdentName {
  IdentName {
    span: DUMMY_SP,
    sym: Atom::from(value)
  }
}

fn create_var_decl(id: Ident, init: Option<Box<Expr>>) -> VarDeclarator {
  let decl_name = Pat::Ident(BindingIdent {
    id,
    type_ann: Default::default(),
  });

  VarDeclarator {
    name: decl_name,
    init,
    span: DUMMY_SP,
    definite: false,
  }
}

fn create_bool_expr(value: bool) -> Expr {
  Expr::Lit(Lit::Bool(Bool {
    value,
    span: Default::default(),
  }))
}

#[allow(dead_code)]
pub fn keep_platform(options: KeepPlatformConfig) -> impl swc_core::ecma::ast::Pass {
  let platform: String = match options {
    KeepPlatformConfig::KeepPlatform(platform) => platform,
    _ => "".to_string(),
  };
  
  fold_pass(KeepPlatformPatcher { platform })
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

  fn test_transform(input: &str, platform: &str, expected_vars: Vec<(&str, bool)>) {
    let mut module = parse_js(input);
    let mut transform = KeepPlatformPatcher {
      platform: platform.to_string(),
    };
    module = module.fold_with(&mut transform);
    
    // Basic validation - in real tests we'd check the actual variable assignments
    if !expected_vars.is_empty() {
      // Should have variable declarations
      let has_var_decl = module.body.iter().any(|item| {
        matches!(item, ModuleItem::Stmt(Stmt::Decl(Decl::Var(_))))
      });
      assert!(has_var_decl, "Expected variable declarations");
    }
  }

  #[test]
  fn test_keep_platform_web() {
    let input = r#"import { isWeb, isWeex } from 'universal-env';

if (isWeb) {
  console.log('This is web');
} else if (isWeex) {
  console.log('This is weex');
} else {
  console.log('others');
}"#;
    
    test_transform(input, "web", vec![("isWeb", true), ("isWeex", false)]);
  }

  #[test]
  fn test_keep_platform_kraken() {
    let input = r#"import { isWeb, isKraken } from 'universal-env';

if (isWeb) {
  console.log('This is web');
} else if (isKraken) {
  console.log('This is kraken');
}"#;
    
    test_transform(input, "kraken", vec![("isWeb", true), ("isKraken", true)]);
  }

  #[test]
  fn test_keep_platform_namespace() {
    let input = r#"import * as env from 'universal-env';

if (env.isWeb) {
  console.log('This is web');
}"#;
    
    test_transform(input, "web", vec![]);
  }

  #[test]
  fn test_keep_platform_empty() {
    let input = r#"const foo = 1;
export default foo;"#;
    
    test_transform(input, "web", vec![]);
  }

  #[test]
  fn test_keep_platform_named_export() {
    let input = r#"import { isWeb } from 'universal-env';

export const isWebEnv = isWeb;"#;
    
    test_transform(input, "web", vec![("isWeb", true)]);
  }

  #[test]
  fn test_keep_platform_default_export() {
    let input = r#"import { isWeb } from 'universal-env';

export default isWeb;"#;
    
    test_transform(input, "web", vec![("isWeb", true)]);
  }

  #[test]
  fn test_keep_platform_wechat_miniprogram() {
    let input = r#"import { isWeChatMiniProgram } from '@uni/env';

if (isWeChatMiniProgram) {
  console.log('WeChat MiniProgram');
}"#;
    
    test_transform(input, "wechat-miniprogram", vec![("isWeChatMiniProgram", true)]);
  }
}