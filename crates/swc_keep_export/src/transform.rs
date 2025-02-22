// transform code is modified based on swc plugin of keep_export:
// https://github.com/ice-lab/swc-plugins/tree/main/packages/keep-export
use std::mem::take;

use fxhash::FxHashSet;
use swc_core::{
  common::{
    pass::{Repeat, Repeated},
    DUMMY_SP,
  },
  ecma::{
    ast::*,
    visit::{noop_fold_type, Fold, FoldWith, fold_pass},
  },
};
/// State of the transforms. Shared by the analyzer and the transform.
#[derive(Debug, Default)]
struct State {
  /// Identifiers referenced by other functions.
  ///
  /// Cleared before running each pass, because we drop ast nodes between the
  /// passes.
  refs_from_other: FxHashSet<Id>,

  /// Identifiers referenced by kept functions or derivatives.
  ///
  /// Preserved between runs, because we should remember derivatives of data
  /// functions as the data function itself is already removed.
  refs_used: FxHashSet<Id>,

  should_run_again: bool,
  keep_exports: Vec<String>,
}

impl State {
  fn should_keep_identifier(&mut self, i: &Ident) -> bool {
    self.keep_exports.contains(&String::from(&*i.sym))
  }

  fn should_keep_default(&mut self) -> bool {
    self.keep_exports.contains(&String::from("default"))
  }
}

struct KeepExport {
  pub state: State,
  in_lhs_of_var: bool,
}

impl KeepExport {
  fn should_remove(&self, id: Id) -> bool {
    !self.state.refs_used.contains(&id) && !self.state.refs_from_other.contains(&id)
  }

  /// Mark identifiers in `n` as a candidate for removal.
  fn mark_as_candidate<N>(&mut self, n: N) -> N
  where
    N: for<'a> FoldWith<Analyzer<'a>>,
  {
    // Analyzer never change `in_kept_fn` to false, so all identifiers in `n` will
    // be marked as referenced from a data function.
    let mut v = Analyzer {
      state: &mut self.state,
      in_lhs_of_var: false,
      in_kept_fn: false,
    };

    let n = n.fold_with(&mut v);
    self.state.should_run_again = true;
    n
  }
}

impl Repeated for KeepExport {
  fn changed(&self) -> bool {
    self.state.should_run_again
  }

  fn reset(&mut self) {
    self.state.refs_from_other.clear();
    self.state.should_run_again = false;
  }
}

impl Fold for KeepExport {
  // This is important for reducing binary sizes.
  noop_fold_type!();

  // Remove import expression
  fn fold_import_decl(&mut self, mut i: ImportDecl) -> ImportDecl {
    // Imports for side effects.
    if i.specifiers.is_empty() {
      return i;
    }

    i.specifiers.retain(|s| match s {
      ImportSpecifier::Named(ImportNamedSpecifier { local, .. })
      | ImportSpecifier::Default(ImportDefaultSpecifier { local, .. })
      | ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
        if self.should_remove(local.to_id()) {
          self.state.should_run_again = true;
          false
        } else {
          true
        }
      }
    });

    i
  }

  fn fold_module(&mut self, mut m: Module) -> Module {
    {
      // Fill the state.
      let mut v = Analyzer {
        state: &mut self.state,
        in_lhs_of_var: false,
        in_kept_fn: false,
      };
      m = m.fold_with(&mut v);
    }

    m.fold_children_with(self)
  }

  fn fold_module_items(&mut self, mut items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    items = items.fold_children_with(self);

    // Drop nodes.
    items.retain(|s| !matches!(s, ModuleItem::Stmt(Stmt::Empty(..))));

    // If all exports are deleted, return the empty named export.
    if items.len() == 0 {
      items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
        NamedExport {
          span: DUMMY_SP,
          specifiers: Vec::new(),
          src: None,
          type_only: false,
          with: Default::default(),
        },
      )));
    }

    items
  }

  fn fold_module_item(&mut self, i: ModuleItem) -> ModuleItem {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(i)) = i {
      let i = i.fold_with(self);

      if i.specifiers.is_empty() {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
      }

      return ModuleItem::ModuleDecl(ModuleDecl::Import(i));
    }

    let i = i.fold_children_with(self);

    match &i {
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) if e.specifiers.is_empty() => {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
      }
      _ => {}
    }

    i
  }

  fn fold_named_export(&mut self, mut n: NamedExport) -> NamedExport {
    n.specifiers = n.specifiers.fold_with(self);

    n.specifiers.retain(|s| {
      let preserve = match s {
        ExportSpecifier::Namespace(ExportNamespaceSpecifier {
          name: ModuleExportName::Ident(exported),
          ..
        })
        | ExportSpecifier::Default(ExportDefaultSpecifier { exported, .. })
        | ExportSpecifier::Named(ExportNamedSpecifier {
          exported: Some(ModuleExportName::Ident(exported)),
          ..
        }) => self.state.should_keep_identifier(exported),
        ExportSpecifier::Named(ExportNamedSpecifier {
          orig: ModuleExportName::Ident(orig),
          ..
        }) => self.state.should_keep_identifier(orig),
        _ => false,
      };

      match preserve {
        false => {
          if let ExportSpecifier::Named(ExportNamedSpecifier {
            orig: ModuleExportName::Ident(_orig),
            ..
          }) = s
          {
            self.state.should_run_again = true;
          }

          false
        }
        true => true,
      }
    });

    n
  }

  /// This methods returns [Pat::Invalid] if the pattern should be removed.
  fn fold_pat(&mut self, mut p: Pat) -> Pat {
    p = p.fold_children_with(self);

    if self.in_lhs_of_var {
      match &mut p {
        Pat::Ident(name) => {
          if self.should_remove(name.id.to_id()) {
            self.state.should_run_again = true;
            return Pat::Invalid(Invalid { span: DUMMY_SP });
          }
        }
        Pat::Array(arr) => {
          if !arr.elems.is_empty() {
            arr.elems.retain(|e| !matches!(e, Some(Pat::Invalid(..))));

            if arr.elems.is_empty() {
              return Pat::Invalid(Invalid { span: DUMMY_SP });
            }
          }
        }
        Pat::Object(obj) => {
          if !obj.props.is_empty() {
            obj.props = take(&mut obj.props)
              .into_iter()
              .filter_map(|prop| match prop {
                ObjectPatProp::KeyValue(prop) => {
                  if prop.value.is_invalid() {
                    None
                  } else {
                    Some(ObjectPatProp::KeyValue(prop))
                  }
                }
                ObjectPatProp::Assign(prop) => {
                  if self.should_remove(prop.key.to_id()) {
                    self.mark_as_candidate(prop.value);

                    None
                  } else {
                    Some(ObjectPatProp::Assign(prop))
                  }
                }
                ObjectPatProp::Rest(prop) => {
                  if prop.arg.is_invalid() {
                    None
                  } else {
                    Some(ObjectPatProp::Rest(prop))
                  }
                }
              })
              .collect();

            if obj.props.is_empty() {
              return Pat::Invalid(Invalid { span: DUMMY_SP });
            }
          }
        }
        Pat::Rest(rest) => {
          if rest.arg.is_invalid() {
            return Pat::Invalid(Invalid { span: DUMMY_SP });
          }
        }
        _ => {}
      }
    }

    p
  }

  #[allow(clippy::single_match)]
  fn fold_stmt(&mut self, mut s: Stmt) -> Stmt {
    match s {
      Stmt::Decl(Decl::Fn(f)) => {
        if self.should_remove(f.ident.to_id()) {
          self.mark_as_candidate(f.function);
          return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
        }

        s = Stmt::Decl(Decl::Fn(f));
      }
      Stmt::Decl(Decl::Class(c)) => {
        if self.should_remove(c.ident.to_id()) {
          self.mark_as_candidate(c.class);
          return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
        }

        s = Stmt::Decl(Decl::Class(c));
      }
      _ => {}
    }

    let s = s.fold_children_with(self);
    match s {
      Stmt::Decl(Decl::Var(v)) if v.decls.is_empty() => {
        return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
      }
      _ => {}
    }

    s
  }

  /// This method make `name` of [VarDeclarator] to [Pat::Invalid] if it
  /// should be removed.
  fn fold_var_declarator(&mut self, mut d: VarDeclarator) -> VarDeclarator {
    let old = self.in_lhs_of_var;
    self.in_lhs_of_var = true;
    let name = d.name.fold_with(self);

    self.in_lhs_of_var = false;
    if name.is_invalid() {
      d.init = self.mark_as_candidate(d.init);
    }
    let init = d.init.fold_with(self);
    self.in_lhs_of_var = old;

    VarDeclarator { name, init, ..d }
  }

  fn fold_var_declarators(&mut self, mut decls: Vec<VarDeclarator>) -> Vec<VarDeclarator> {
    decls = decls.fold_children_with(self);
    decls.retain(|d| !d.name.is_invalid());

    decls
  }
}

struct Analyzer<'a> {
  state: &'a mut State,
  in_lhs_of_var: bool,
  in_kept_fn: bool,
}

impl Analyzer<'_> {
  fn add_ref(&mut self, id: Id) {
    if self.in_kept_fn {
      self.state.refs_used.insert(id);
    } else {
      self.state.refs_from_other.insert(id);
    }
  }

  fn check_default<T: FoldWith<Self>>(&mut self, e: T) -> T {
    if self.state.should_keep_default() {
      let old_in_kept = self.in_kept_fn;
      self.in_kept_fn = true;
      let e = e.fold_children_with(self);
      self.in_kept_fn = old_in_kept;
      return e;
    }

    return e;
  }
}

impl Fold for Analyzer<'_> {
  // This is important for reducing binary sizes.
  noop_fold_type!();

  fn fold_binding_ident(&mut self, i: BindingIdent) -> BindingIdent {
    if !self.in_lhs_of_var || self.in_kept_fn {
      self.add_ref(i.id.to_id());
    }
    i
  }

  fn fold_export_named_specifier(&mut self, s: ExportNamedSpecifier) -> ExportNamedSpecifier {
    if let ModuleExportName::Ident(i) = &s.orig {
      match &s.exported {
        Some(exported) => {
          if let ModuleExportName::Ident(e) = exported {
            if self.state.should_keep_identifier(e) {
              self.add_ref(i.to_id());
            }
          }
        }
        None => {
          if self.state.should_keep_identifier(i) {
            self.add_ref(i.to_id());
          }
        }
      }
    }
    s
  }

  fn fold_export_decl(&mut self, s: ExportDecl) -> ExportDecl {
    let old_in_kept = self.in_kept_fn;

    match &s.decl {
      Decl::Fn(f) => {
        if self.state.should_keep_identifier(&f.ident) {
          self.in_kept_fn = true;
          self.add_ref(f.ident.to_id());
        }
      }

      Decl::Var(d) => {
        if d.decls.is_empty() {
          return s;
        }
        if let Pat::Ident(id) = &d.decls[0].name {
          if self.state.should_keep_identifier(&id.id) {
            self.in_kept_fn = true;
            self.add_ref(id.to_id());
          }
        }
      }
      _ => {}
    }
    let e = s.fold_children_with(self);
    self.in_kept_fn = old_in_kept;
    e
  }

  fn fold_expr(&mut self, e: Expr) -> Expr {
    let e = e.fold_children_with(self);

    if let Expr::Ident(i) = &e {
      self.add_ref(i.to_id());
    }
    e
  }

  fn fold_jsx_element(&mut self, jsx: JSXElement) -> JSXElement {
    fn get_leftmost_id_member_expr(e: &JSXMemberExpr) -> Id {
      match &e.obj {
        JSXObject::Ident(i) => i.to_id(),
        JSXObject::JSXMemberExpr(e) => get_leftmost_id_member_expr(e),
      }
    }

    match &jsx.opening.name {
      JSXElementName::Ident(i) => {
        self.add_ref(i.to_id());
      }
      JSXElementName::JSXMemberExpr(e) => {
        self.add_ref(get_leftmost_id_member_expr(e));
      }
      _ => {}
    }

    jsx.fold_children_with(self)
  }

  fn fold_fn_decl(&mut self, f: FnDecl) -> FnDecl {
    let f = f.fold_children_with(self);
    if self.in_kept_fn {
      self.add_ref(f.ident.to_id());
    }
    f
  }

  fn fold_fn_expr(&mut self, f: FnExpr) -> FnExpr {
    let f = f.fold_children_with(self);
    if let Some(id) = &f.ident {
      self.add_ref(id.to_id());
    }
    f
  }

  /// Drops [ExportDecl] if all specifiers are removed.
  fn fold_module_item(&mut self, s: ModuleItem) -> ModuleItem {
    match s {
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) if !e.specifiers.is_empty() => {
        let e = e.fold_with(self);

        if e.specifiers.is_empty() {
          return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
        }

        return ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e));
      }

      ModuleItem::Stmt(Stmt::Expr(_e)) => {
        // remove top expression
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
      }

      ModuleItem::Stmt(Stmt::If(_e)) => {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
      }

      ModuleItem::Stmt(Stmt::DoWhile(_e)) => {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
      }

      ModuleItem::Stmt(Stmt::Try(_e)) => {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
      }
      _ => {}
    };

    if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(e)) = &s {
      match &e.decl {
        Decl::Fn(f) => {
          if self.state.should_keep_identifier(&f.ident) {
            let s = s.fold_children_with(self);
            return s;
          } else {
            return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
          }
        }

        Decl::Var(d) => {
          if d.decls.is_empty() {
            return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
          }

          if let Pat::Ident(id) = &d.decls[0].name {
            if self.state.should_keep_identifier(&id.id) {
              let s = s.fold_children_with(self);
              return s;
            } else {
              return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
            }
          }
        }
        _ => {}
      }
    }

    if let ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(_e)) = &s {
      if !self.state.should_keep_default() {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
      }
    }

    if let ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(_e)) = &s {
      if !self.state.should_keep_default() {
        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
      }
    }

    // Visit children to ensure that all references is added to the scope.
    let s = s.fold_children_with(self);
    s
  }

  fn fold_default_decl(&mut self, d: DefaultDecl) -> DefaultDecl {
    return self.check_default(d);
  }

  fn fold_export_default_expr(&mut self, e: ExportDefaultExpr) -> ExportDefaultExpr {
    return self.check_default(e);
  }

  fn fold_prop(&mut self, p: Prop) -> Prop {
    let p = p.fold_children_with(self);

    if let Prop::Shorthand(i) = &p {
      self.add_ref(i.to_id());
    }

    p
  }

  fn fold_var_declarator(&mut self, mut v: VarDeclarator) -> VarDeclarator {
    let old_in_lhs_of_var = self.in_lhs_of_var;

    self.in_lhs_of_var = true;
    v.name = v.name.fold_with(self);

    self.in_lhs_of_var = false;
    v.init = v.init.fold_with(self);

    self.in_lhs_of_var = old_in_lhs_of_var;
    v
  }
}

pub fn keep_export(exports: Vec<String>) -> impl Pass {
  fold_pass(
    Repeat::new(KeepExport {
      state: State {
        keep_exports: exports,
        ..Default::default()
      },
      in_lhs_of_var: false,
    })
  )
}
