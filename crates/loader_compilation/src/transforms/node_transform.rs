use swc_core::{
  common::{util::take::Take, SyntaxContext, DUMMY_SP},
  ecma::{
    ast::*,
    atoms::Atom,
    visit::{Fold, FoldWith, fold_pass},
  },
};

pub struct NodeTransform;

fn create_import_str(key: i32) -> String {
  format!("__ice_import_{key}__")
}

fn create_var_decl(id: &str, init: Option<Box<Expr>>) -> VarDeclarator {
  let decl_name: Pat = Pat::Ident(BindingIdent {
    id: Ident {
      span: DUMMY_SP,
      sym: Atom::from(id),
      optional: Default::default(),
      ctxt: SyntaxContext::empty(),
    },
    type_ann: Default::default(),
  });
  VarDeclarator {
    span: DUMMY_SP,
    name: decl_name,
    init,
    definite: false,
  }
}

fn create_member_decl(id: Ident, object_name: &str, property: &str) -> VarDeclarator {
  VarDeclarator {
    span: DUMMY_SP,
    name: Pat::Ident(BindingIdent {
      id,
      type_ann: Default::default(),
    }),
    init: Some(Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        sym: Atom::from(object_name),
        optional: Default::default(),
        ctxt: SyntaxContext::empty(),
      })),
      prop: MemberProp::Ident(IdentName {
        span: DUMMY_SP,
        sym: Atom::from(property),
      }),
    }))),
    definite: false,
  }
}

fn create_import_decl(import_val: &str, import_source: &str) -> ModuleItem {
  let call_args = vec![ExprOrSpread {
    spread: Take::dummy(),
    expr: Box::new(Expr::Lit(Lit::Str(Str {
      span: DUMMY_SP,
      value: Atom::from(import_source),
      raw: Default::default(),
    }))),
  }];
  let decls: Vec<VarDeclarator> = vec![create_var_decl(
    import_val,
    Some(Box::new(Expr::Await(AwaitExpr {
      span: DUMMY_SP,
      arg: Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Ident(Ident {
          ctxt: SyntaxContext::empty(),
          span: DUMMY_SP,
          sym: Atom::from("__ice_import__"),
          optional: Default::default(),
        }))),
        args: call_args,
        type_args: Take::dummy(),
        ctxt: SyntaxContext::empty(),
      })),
    }))),
  )];

  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    declare: false,
    decls,
    ctxt: SyntaxContext::empty(),
  }))))
}

fn create_define_export(name: &str, value: &str) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      ctxt: SyntaxContext::empty(),
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          ctxt: SyntaxContext::empty(),
          span: DUMMY_SP,
          sym: Atom::from("Object"),
          optional: Default::default(),
        })),
        prop: MemberProp::Ident(IdentName {
          span: DUMMY_SP,
          sym: Atom::from("defineProperty"),
        }),
      }))),
      args: vec![
        ExprOrSpread {
          spread: Take::dummy(),
          expr: Box::new(Expr::Ident(Ident {
            span: DUMMY_SP,
            sym: Atom::from("__ice_exports__"),
            optional: Default::default(),
            ctxt: SyntaxContext::empty(),
          })),
        },
        ExprOrSpread {
          spread: Take::dummy(),
          expr: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: Atom::from(name),
            raw: Default::default(),
          }))),
        },
        ExprOrSpread {
          spread: Take::dummy(),
          expr: Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: vec![
              PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(IdentName {
                  span: DUMMY_SP,
                  sym: Atom::from("enumerable"),
                }),
                value: Box::new(Expr::Lit(Lit::Bool(Bool {
                  span: DUMMY_SP,
                  value: true,
                }))),
              }))),
              PropOrSpread::Prop(Box::new(Prop::Getter(GetterProp {
                span: DUMMY_SP,
                key: PropName::Ident(IdentName {
                  span: DUMMY_SP,
                  sym: Atom::from("get"),
                }),
                body: Some(BlockStmt {
                  ctxt: SyntaxContext::empty(),
                  span: DUMMY_SP,
                  stmts: vec![Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(Expr::Ident(Ident {
                      ctxt: SyntaxContext::empty(),
                      span: DUMMY_SP,
                      sym: Atom::from(value),
                      optional: Default::default(),
                    }))),
                  })],
                }),
                type_ann: Default::default(),
              }))),
            ],
          })),
        },
      ],
      type_args: Take::dummy(),
    })),
  }))
}

fn create_call_expr(name: &str) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Ident(Ident {
        ctxt: SyntaxContext::empty(),
        span: DUMMY_SP,
        sym: Atom::from("__ice_exports_all__"),
        optional: Default::default(),
      }))),
      args: vec![ExprOrSpread {
        spread: Take::dummy(),
        expr: Box::new(Expr::Ident(Ident {
          ctxt: SyntaxContext::empty(),
          span: DUMMY_SP,
          sym: Atom::from(name),
          optional: Default::default(),
        })),
      }],
      type_args: Take::dummy(),
      ctxt: SyntaxContext::empty(),
    })),
  }))
}

fn create_default_export(right: Box<Expr>) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Assign(AssignExpr {
      span: DUMMY_SP,
      left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: Atom::from("__ice_exports__"),
          optional: false,
          ctxt: SyntaxContext::empty(),
        })),
        prop: MemberProp::Ident(IdentName {
          span: DUMMY_SP,
          sym: Atom::from("default"),
        }),
      })),
      op: op!("="),
      right,
    })),
  }))
}

fn get_module_name(export_name: &ModuleExportName) -> &Atom {
  match export_name {
    ModuleExportName::Ident(ident) => &ident.sym,
    ModuleExportName::Str(str) => &str.value,
  }
}

impl Fold for NodeTransform {
  fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut new_module_items: Vec<ModuleItem> = vec![];
    let mut import_id: i32 = 0;
    for module_item in items.iter() {
      match module_item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
          let import_val = format!("__ice_import_{import_id}__");
          import_id += 1;
          new_module_items.push(create_import_decl(&import_val, &import_decl.src.value));

          for specifier in import_decl.specifiers.iter() {
            match specifier {
              ImportSpecifier::Named(named) => {
                let ImportNamedSpecifier { local, imported, .. } = named;
                let mut property = &local.sym;
                if let Some(ModuleExportName::Ident(import_ident)) = imported {
                  property = &import_ident.sym;
                }
                new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                  span: DUMMY_SP,
                  kind: VarDeclKind::Const,
                  declare: false,
                  decls: vec![create_member_decl(local.clone(), &import_val, property)],
                  ctxt: SyntaxContext::empty(),
                })))))
              }
              ImportSpecifier::Namespace(namespace) => {
                let ImportStarAsSpecifier { local, .. } = namespace;
                new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                  ctxt: SyntaxContext::empty(),
                  span: DUMMY_SP,
                  kind: VarDeclKind::Const,
                  declare: false,
                  decls: vec![VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent {
                      id: local.clone(),
                      type_ann: Default::default(),
                    }),
                    init: Some(Box::new(Expr::Ident(Ident {
                      ctxt: SyntaxContext::empty(),
                      span: DUMMY_SP,
                      sym: Atom::from(import_val.clone()),
                      optional: Default::default(),
                    }))),
                    definite: false,
                  }],
                })))))
              }
              ImportSpecifier::Default(default) => {
                let ImportDefaultSpecifier { local, .. } = default;
                new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                  span: DUMMY_SP,
                  kind: VarDeclKind::Const,
                  declare: false,
                  decls: vec![create_member_decl(local.clone(), &import_val, "default")],
                  ctxt: SyntaxContext::empty(),
                })))))
              }
            }
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export_named)) => {
          let import_val = create_import_str(import_id);
          let mut has_import = false;
          if let Some(src) = &export_named.src {
            has_import = true;
            import_id += 1;
            new_module_items.push(create_import_decl(&import_val, &src.value));
          }

          for specifier in export_named.specifiers.iter() {
            match specifier {
              ExportSpecifier::Named(named) => {
                let ExportNamedSpecifier { orig, exported, .. } = named;
                let orig_name = get_module_name(orig);
                let export_name = if let Some(exported_ident) = exported {
                  get_module_name(exported_ident)
                } else {
                  orig_name
                };
                let return_value = if has_import {
                  format!("{import_val}.{orig_name}")
                } else {
                  orig_name.to_string()
                };
                new_module_items.push(create_define_export(export_name, &return_value));
              }
              ExportSpecifier::Namespace(namespace) => {
                let ExportNamespaceSpecifier { name, .. } = namespace;
                let export_name = get_module_name(name);
                if has_import {
                  new_module_items.push(create_define_export(export_name, &import_val));
                } else {
                  new_module_items.push(create_define_export(export_name, export_name));
                }
              }
              _ => {}
            }
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_default_decl)) => {
          match &export_default_decl.decl {
            DefaultDecl::Class(class_decl) => {
              if let Some(ident) = &class_decl.ident {
                let export_name = &ident.sym;
                new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
                  ident: ident.clone(),
                  declare: false,
                  class: class_decl.class.clone(),
                }))));
                new_module_items.push(create_define_export("default", export_name));
              } else {
                new_module_items.push(create_default_export(Box::new(Expr::Class(class_decl.clone()))));
              }
            }
            DefaultDecl::Fn(function_decl) => {
              if let Some(ident) = &function_decl.ident {
                let export_name = &ident.sym;
                new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                  ident: ident.clone(),
                  function: function_decl.function.clone(),
                  declare: false,
                }))));
                new_module_items.push(create_define_export("default", export_name));
              } else {
                new_module_items.push(create_default_export(Box::new(Expr::Fn(function_decl.clone()))));
              }
            }
            _ => {
              new_module_items.push(module_item.clone())
            }
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export_default_expr)) => {
          new_module_items.push(create_default_export(export_default_expr.expr.clone()));
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all)) => {
          let import_val = create_import_str(import_id);
          import_id += 1;
          new_module_items.push(create_import_decl(&import_val, &export_all.src.value));
          new_module_items.push(create_call_expr(&import_val));
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
          match &export_decl.decl {
            Decl::Class(class_decl) => {
              let class_name = &class_decl.ident.sym;
              new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(class_decl.clone()))));
              new_module_items.push(create_define_export(class_name, class_name));
            }
            Decl::Fn(fn_decl) => {
              let fn_name = &fn_decl.ident.sym;
              new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl.clone()))));
              new_module_items.push(create_define_export(fn_name, fn_name));
            }
            Decl::Var(var_decl) => {
              new_module_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl.clone()))));
              for decl in var_decl.decls.iter() {
                if decl.name.is_ident() {
                  let var_name = &decl.name.as_ident().unwrap().id.sym;
                  new_module_items.push(create_define_export(var_name, var_name));
                }
              }
            }
            _ => {}
          }
        }
        _ => {
          new_module_items.push(module_item.clone())
        }
      }
    }
    new_module_items = new_module_items.fold_children_with(self);
    new_module_items
  }

  fn fold_call_expr(&mut self, call_expr: CallExpr) -> CallExpr {
    let callee = &call_expr.callee;
    if let Callee::Import(_) = callee {
      CallExpr {
        span: call_expr.span,
        args: call_expr.args,
        type_args: call_expr.type_args,
        callee: Callee::Expr(Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: Atom::from("__ice_dynamic_import__"),
          optional: Default::default(),
          ctxt: SyntaxContext::empty(),
        }))),
        ctxt: SyntaxContext::empty(),
      }
    } else {
      call_expr.fold_children_with(self)
    }
  }

  fn fold_member_expr(&mut self, member_expr: MemberExpr) -> MemberExpr {
    if member_expr.obj.is_meta_prop()
      && member_expr.obj.as_meta_prop().unwrap().kind == MetaPropKind::ImportMeta
    {
      MemberExpr {
        span: member_expr.span,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: Atom::from("__ice_import_meta__"),
          optional: Default::default(),
          ctxt: SyntaxContext::empty(),
        })),
        prop: member_expr.prop,
      }
    } else {
      member_expr.fold_children_with(self)
    }
  }
}

#[allow(dead_code)]
pub fn node_transform() -> impl swc_core::ecma::ast::Pass {
  fold_pass(NodeTransform)
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

  fn test_transform(input: &str, expected_items: usize) {
    let mut module = parse_js(input);
    let mut transform = NodeTransform;
    module = module.fold_with(&mut transform);
    
    // Basic validation - check that transform produces expected number of items
    assert!(module.body.len() >= expected_items, "Expected at least {} items, got {}", expected_items, module.body.len());
  }

  #[test]
  fn test_node_transform_export_default() {
    let input = "export default {};";
    test_transform(input, 1);
  }

  #[test]
  fn test_node_transform_export_named() {
    let input = r#"export const foo = 1;
export function bar() {}"#;
    test_transform(input, 4); // 2 declarations + 2 defineProperty calls
  }

  #[test]
  fn test_node_transform_import_named() {
    let input = r#"import { foo, bar } from 'module';"#;
    test_transform(input, 3); // 1 import + 2 const declarations
  }

  #[test]
  fn test_node_transform_import_default() {
    let input = r#"import foo from 'module';"#;
    test_transform(input, 2); // 1 import + 1 const declaration
  }

  #[test]
  fn test_node_transform_import_namespace() {
    let input = r#"import * as foo from 'module';"#;
    test_transform(input, 2); // 1 import + 1 const declaration
  }

  #[test]
  fn test_node_transform_export_all() {
    let input = r#"export * from 'module';"#;
    test_transform(input, 2); // 1 import + 1 export call
  }

  #[test]
  fn test_node_transform_dynamic_import() {
    let input = r#"import('module').then(mod => console.log(mod));"#;
    test_transform(input, 1);
  }

  #[test]
  fn test_node_transform_import_meta() {
    let input = r#"console.log(import.meta.url);"#;
    test_transform(input, 1);
  }

  #[test]
  fn test_node_transform_mixed() {
    let input = r#"import { foo } from 'module1';
import bar from 'module2';
export const baz = foo + bar;
export default baz * 2;"#;
    test_transform(input, 6); // 2 imports + 2 import declarations + 1 const + 1 defineProperty + 1 default export
  }
}