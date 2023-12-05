use swc_core::{
  ecma::{
    ast::*,
    visit::{Fold, FoldWith},
  },
  common::DUMMY_SP,
};
struct EnvReplacement {
  sources: Vec<String>,
}

fn create_check_expr(meta_value: &str, renderer: &str) -> Expr {
  Expr::Bin(BinExpr {
    span: DUMMY_SP,
    op: BinaryOp::EqEqEq,
    left: Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::MetaProp(MetaPropExpr { span: DUMMY_SP, kind: MetaPropKind::ImportMeta })),
      prop: Ident::new(meta_value.into(), DUMMY_SP).into(),
    })),
    right: Box::new(Expr::Lit(Lit::Str(Str {
      value: renderer.into(),
      span: DUMMY_SP,
      raw: None,
    }))),
  })
}

fn create_typeof_check(expr: Expr, check_value: &str, op: BinaryOp) -> Expr {
  // Create `typeof pha` unary expression
  let typeof_expr = Expr::Unary(UnaryExpr {
    op: UnaryOp::TypeOf,
    arg: Box::new(expr),
    span: DUMMY_SP,
  });

  // Create `typeof pha === 'object'` binary expression
  Expr::Bin(BinExpr {
    left: Box::new(typeof_expr),
    op,
    right: Box::new(Expr::Lit(Lit::Str(Str {
        value: check_value.into(),
        span: DUMMY_SP,
        raw: None,
    }))),
    span: DUMMY_SP,
  })
}

fn combine_check_exprs(exprs: Vec<Expr>, op: BinaryOp) -> Expr {
  let mut result = exprs[0].clone();
  for expr in exprs[1..].iter() {
    result = Expr::Bin(BinExpr {
      span: DUMMY_SP,
      op,
      left: Box::new(result),
      right: Box::new(expr.clone()),
    });
  }
  result
}

fn build_regex_test_expression() -> Expr {
  // Create the regex literal
  let regex_pattern = Expr::Lit(Lit::Regex(Regex {
      exp: ".+AliApp\\((\\w+)\\/((?:\\d+\\.)+\\d+)\\).* .*(WindVane)(?:\\/((?:\\d+\\.)+\\d+))?.*".into(),
      flags: "".into(),
      span: DUMMY_SP,
  }));

  // Create the typeof expression for `navigator`
  let typeof_navigator = Expr::Unary(UnaryExpr {
      op: UnaryOp::TypeOf,
      arg: Box::new(Expr::Ident(Ident::new("navigator".into(), DUMMY_SP))),
      span: DUMMY_SP,
  });

  // Create the conditional expression
  let conditional = Expr::Cond(CondExpr {
      test: Box::new(typeof_navigator),
      cons: Box::new(Expr::Bin(BinExpr {
          left: Box::new(Expr::Member(MemberExpr {
              obj: Box::new(Expr::Ident(Ident::new("navigator".into(), DUMMY_SP))),
              prop: MemberProp::Ident(Ident::new("userAgent".into(), DUMMY_SP)),
              span: DUMMY_SP,
          })),
          op: BinaryOp::LogicalOr,
          right: Box::new(Expr::Member(MemberExpr {
              obj: Box::new(Expr::Ident(Ident::new("navigator".into(), DUMMY_SP))),
              prop: MemberProp::Ident(Ident::new("swuserAgent".into(), DUMMY_SP)),
              span: DUMMY_SP,
          })),
          span: DUMMY_SP,
      })),
      alt: Box::new(Expr::Lit(Lit::Str(Str {
          value: "".into(),
          span: DUMMY_SP,
          raw: None,
      }))),
      span: DUMMY_SP,
  });

  // Create the 'test' method call on the regex pattern
  let test_call = Expr::Call(CallExpr {
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
          obj: Box::new(regex_pattern),
          prop: MemberProp::Ident(Ident::new("test".into(), DUMMY_SP)),
          span: DUMMY_SP,
      }))),
      args: vec![ExprOrSpread {
          spread: None,
          expr: Box::new(conditional),
      }],
      span: DUMMY_SP,
      type_args: None,
  });

  test_call
}

fn get_env_expr(specifier: &Ident) -> Expr {
  match specifier.sym.as_ref() {
    "isClient" => {
      create_check_expr("renderer", "client")
    },
    "isServer" => {
      create_check_expr("renderer", "server")
    },
    "isWeb" => {
      combine_check_exprs(vec![
        create_check_expr("renderer", "client"),
        create_check_expr("target", "web"),
      ], BinaryOp::LogicalAnd)
    },
    "isNode" => {
      create_check_expr("renderer", "server")
    },
    "isWeex" => {
      combine_check_exprs(vec![
        create_check_expr("renderer", "client"),
        create_check_expr("target", "weex"),
      ], BinaryOp::LogicalAnd)
    },
    "isKraken" => {
      combine_check_exprs(vec![
        create_check_expr("renderer", "client"),
        create_check_expr("target", "kraken"),
      ], BinaryOp::LogicalAnd)
    },
    "isPHA" => {
      combine_check_exprs(vec![
        create_check_expr("renderer", "client"),
        create_check_expr("target", "web"),
        create_typeof_check(
          Expr::Ident(Ident::new("pha".into(), DUMMY_SP)),
          "object",
          BinaryOp::EqEqEq,
        ),
      ], BinaryOp::LogicalAnd)
    },
    "isWindVane" => {
      combine_check_exprs(vec![
        create_check_expr("renderer", "client"),
        build_regex_test_expression(),
        create_typeof_check(
          Expr::Ident(Ident::new("WindVane".into(), DUMMY_SP)),
          "undefined",
          BinaryOp::NotEqEq,
        ),
        create_typeof_check(
          Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new("WindVane".into(), DUMMY_SP))),
            prop: Ident::new("call".into(), DUMMY_SP).into(),
          }),
          "undefined",
          BinaryOp::NotEqEq,
        ),
      ], BinaryOp::LogicalAnd)
    },
    _ => {
      // Do not support miniapp env.
      Expr::Lit(Lit::Bool(Bool {
        span: DUMMY_SP,
        value: false,
      }))
    }
  }
}

fn create_env_declare(specifier: &Ident, imported: &Ident) -> Stmt {
  let expr = get_env_expr(&specifier);

  return Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: imported.clone(),
        type_ann: Default::default(),
      }),
      init: Some(Box::new(expr)),
      definite: false,
    }],
  })));
}

fn create_env_default_export(export_name: Ident) -> Stmt {
  Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: export_name.clone(),
        type_ann: Default::default(),
      }),
      init: Some(Box::new(Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props: vec![
          "isWeb",
          "isClient",
          "isNode",
          "isWeex",
          "isKraken",
          "isMiniApp",
          "isByteDanceMicroApp",
          "isBaiduSmartProgram",
          "isKuaiShouMiniProgram",
          "isWeChatMiniProgram",
          "isQuickApp",
          "isPHA",
          "isWindVane",
          "isFRM",
        ].into_iter().map(|target| PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Ident(Ident::new(target.into(), DUMMY_SP)),
          value: Box::new(get_env_expr(&Ident::new(target.into(), DUMMY_SP))),
        })))).collect(),
      }))),
      definite: false,
    }],
  })))
}

fn get_env_stmt(sources: &Vec<String>, decls: Vec<VarDeclarator>) -> Vec<Stmt> {
  let mut stmts = vec![];
  for decl in decls {
    if let Some(init) = decl.init {
      if let Expr::Call(CallExpr {
        args: ref call_args,
        callee: Callee::Expr(box Expr::Ident(Ident { ref sym, .. })),
        ..
      }) = *init {
        if sym == "require" && call_args.len() == 1 {
          // Case const env = require('env');
          if let ExprOrSpread { expr: box Expr::Lit(Lit::Str(Str { ref value, .. })), .. } = call_args[0] {
            if sources.iter().any(|s| value == s) {
              match &decl.name {
                Pat::Ident(BindingIdent { id, .. }) => {
                  stmts.push(create_env_default_export(id.clone()));
                }
                Pat::Object(ObjectPat { props, .. }) => {
                  props.iter().for_each(|prop| {
                    match prop {
                      ObjectPatProp::Assign(AssignPatProp { key, value, .. }) => {
                        if value.is_some() {
                          if let box Expr::Ident(ident) = &value.as_ref().unwrap() {
                            stmts.push(create_env_declare(key, &ident));
                          }
                        } else {
                          stmts.push(create_env_declare(key, key));
                        }
                      }
                      ObjectPatProp::KeyValue(KeyValuePatProp { key, value, .. }) => {
                        if let box Pat::Ident(BindingIdent { id , ..}) = &value {
                          if let PropName::Ident(i) = key {
                            stmts.push(create_env_declare(i, &id));
                          }
                        }
                      }
                      ObjectPatProp::Rest(RestPat {arg, ..}) => {
                        if let box Pat::Ident(BindingIdent { id , ..}) = arg {
                          stmts.push(create_env_default_export(id.clone()));
                        }
                      }
                    }
                  });
                }
                _ => {}
              }
              continue;
            }
          }
        }
      }
    }
  }
  stmts
}

impl Fold for EnvReplacement {
  fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut new_module_items: Vec<ModuleItem> = vec![];
    for item in items.iter() {
      match &item {
        // Import declaration.
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
          let src = &import_decl.src.value;
          if self.sources.iter().any(|s| src == s) {
            // Collect all specifiers.
            import_decl.specifiers.iter().for_each(|specifier| {
              match specifier {
                ImportSpecifier::Named(named_specifier) => {
                  let imported = match &named_specifier.imported {
                    Some(ModuleExportName::Ident(ident)) => Some(ident),
                    _ => None,
                  };
                  let s = if imported.is_some() {
                    imported.unwrap()
                  } else {
                    &named_specifier.local
                  };
                  new_module_items.push(ModuleItem::Stmt(create_env_declare(s, &named_specifier.local)));
                }
                ImportSpecifier::Default(default_specifier) => {
                  new_module_items.push(ModuleItem::Stmt(create_env_default_export(default_specifier.local.clone())));
                }
                ImportSpecifier::Namespace(namespace_specifier) => {
                  new_module_items.push(ModuleItem::Stmt(create_env_default_export(namespace_specifier.local.clone())));
                }
              }
            });
            
          } else {
            new_module_items.push(item.clone());
          }
        }
        ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) => {
          let stmt = get_env_stmt(&self.sources, var_decl.decls.clone());
          if stmt.len() > 0 {
            let module_stmts = stmt.into_iter().map(|s| ModuleItem::Stmt(s)).collect::<Vec<ModuleItem>>();
            new_module_items.extend_from_slice(&module_stmts);
          } else {
            new_module_items.push(item.clone());
          }
        }
        _ => {
          new_module_items.push(item.clone().fold_children_with(self));
        }
      }
    }
    new_module_items
  }

  fn fold_block_stmt(&mut self, block: BlockStmt) -> BlockStmt {
    let mut new_stmts: Vec<Stmt> = vec![];
    block.stmts.clone().into_iter().for_each(|stmt| {
      match &stmt {
        Stmt::Decl(Decl::Var(var_decl)) => {
          let env_stmts = get_env_stmt(&self.sources, var_decl.decls.clone());
          if env_stmts.len() > 0 {
            new_stmts.extend_from_slice(&env_stmts);
          } else {
            new_stmts.push(stmt);
          }
        }
        _ => {
          new_stmts.push(stmt.fold_children_with(self));
        }
      }
    });
    BlockStmt {
      stmts: new_stmts,
      ..block
    }
  }
}

pub fn env_replacement(sources: Vec<String>) -> impl Fold {
  EnvReplacement { sources }
}