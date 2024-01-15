use crate::config::{Config, ImportType, MapProperty};
use swc_core::{
    common::{
        DUMMY_SP,
        util::{
            take::Take
        },
    },
    ecma::{
        ast::*,
        atoms::{JsWord},
        visit::{VisitMut, VisitMutWith},
    },
};
use swc_ecma_utils::quote_str;

pub struct ModuleImportVisitor {
    pub options: Vec<Config>,
    new_stmts: Vec<ModuleItem>,
}

impl ModuleImportVisitor {
    pub fn new(options: Vec<Config>) -> Self {
        Self {
            options,
            new_stmts: vec![],
        }
    }
}

impl VisitMut for ModuleImportVisitor {
    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        import_decl.visit_mut_children_with(self);

        for option in &self.options {
            match option {
                Config::LiteralConfig(_) => {
                    if is_hit_rule(import_decl, option) {
                        // import { a } from "b";
                        if import_decl.specifiers.len() == 1 {
                            match &import_decl.specifiers[0] {
                                ImportSpecifier::Named(named_import_spec) => {
                                    if named_import_spec.is_type_only {
                                        continue;
                                    }
                                    let new_src = get_new_src(named_import_spec);
                                    import_decl.src = Box::new(quote_str!(named_import_spec.span, new_src));
                                    import_decl.specifiers[0] = ImportSpecifier::Default(ImportDefaultSpecifier {
                                        span: named_import_spec.span,
                                        local: named_import_spec.local.clone(),
                                    });
                                }
                                _ => ()
                            }
                            // import { a, b } from "c";
                        } else {
                            for specifier in &import_decl.specifiers {
                                match specifier {
                                    ImportSpecifier::Named(named_import_spec) => {
                                        if named_import_spec.is_type_only {
                                            continue;
                                        }
                                        let new_src: String = get_new_src(named_import_spec);
                                        let new_import_decl = create_default_import_decl(new_src, named_import_spec.local.clone());
                                        self.new_stmts.push(new_import_decl);
                                    }
                                    _ => ()
                                }
                            }
                            import_decl.take();
                        };
                        break;
                    }
                }
                Config::SpecificConfig(config) => {
                    if is_hit_rule(import_decl, option) {
                        if import_decl.specifiers.len() == 1 {
                            for (target, rules) in config.map.iter() {
                                let MapProperty { to, import_type, name } = rules;

                                match &mut import_decl.specifiers[0] {
                                    ImportSpecifier::Named(named_import_spec) => {
                                        if named_import_spec.is_type_only {
                                            continue;
                                        }

                                        if named_import_spec.imported.is_some() {
                                            match named_import_spec.imported.clone().unwrap() {
                                                ModuleExportName::Ident(ident) => {
                                                    if ident.sym.to_string() != *target {
                                                        continue;
                                                    }
                                                }
                                                ModuleExportName::Str(str) => {
                                                    if str.value.to_string() != *target {
                                                        continue;
                                                    }
                                                }
                                            }
                                        } else if named_import_spec.local.sym.to_string() != *target {
                                            continue;
                                        }

                                        import_decl.src = Box::new(quote_str!(named_import_spec.span, to.to_string()));
                                        if import_type.is_none() || match import_type.as_ref().unwrap() {
                                            ImportType::Default => true,
                                            _ => false
                                        } {
                                            import_decl.specifiers[0] = ImportSpecifier::Default(ImportDefaultSpecifier {
                                                span: named_import_spec.span,
                                                local: named_import_spec.local.clone(),
                                            })
                                        } else if name.is_some() {
                                            named_import_spec.imported = Some(ModuleExportName::Str(Str {
                                                span: named_import_spec.span,
                                                value: name.clone().unwrap().into(),
                                                raw: Some(name.clone().unwrap().clone().into()),
                                            }))
                                        }
                                    }
                                    _ => ()
                                }
                            }
                        } else {
                            let target_fields: Vec<&String> = config.map.keys().clone().collect();

                            // 当导入的字段数量与配置的字段数量不一致时，需保留未命中配置的导入字段
                            if target_fields.len() != import_decl.specifiers.len() {
                                let mut named_import_spec_copy = import_decl.clone();
                                named_import_spec_copy.specifiers = named_import_spec_copy.specifiers.into_iter().filter(|specifier| {
                                    match specifier {
                                        ImportSpecifier::Named(named_import_spec) => {
                                            !target_fields.contains(&&named_import_spec.local.sym.to_string())
                                        }
                                        _ => true
                                    }
                                }).collect::<Vec<_>>();

                                self.new_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(named_import_spec_copy)));
                            }

                            for (target, rules) in config.map.iter() {
                                for specifier in &import_decl.specifiers {
                                    match specifier {
                                        ImportSpecifier::Named(named_import_spec) => {
                                            if target == &named_import_spec.local.sym.to_string() {
                                                let new_import_decl: ModuleItem;
                                                if rules.import_type.is_none() || match rules.import_type.as_ref().unwrap() {
                                                    ImportType::Default => true,
                                                    _ => false
                                                } {
                                                    // Default import mode
                                                    new_import_decl = create_default_import_decl(rules.to.to_string(), named_import_spec.local.clone());
                                                } else {
                                                    // Named import mode
                                                    let mut named_import_spec_copy = named_import_spec.clone();

                                                    named_import_spec_copy.local.sym = target.clone().into();

                                                    if rules.name.is_some() {
                                                        named_import_spec_copy.imported = Some(ModuleExportName::Str(Str {
                                                            span: named_import_spec.span,
                                                            value: rules.name.clone().unwrap().into(),
                                                            raw: Some(rules.name.clone().unwrap().clone().into()),
                                                        }))
                                                    }

                                                    new_import_decl = create_named_import_decl(rules.to.to_string(), vec![swc_ecma_utils::swc_ecma_ast::ImportSpecifier::Named(named_import_spec_copy)]);
                                                }

                                                self.new_stmts.push(new_import_decl);
                                            }
                                        }
                                        _ => ()
                                    }
                                }
                            }
                            import_decl.take();
                        }
                        break;
                    }
                }
            }
        }

        if !is_empty_decl(import_decl) {
            self.new_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl.clone())));
        }
    }

    fn visit_mut_module_item(&mut self, n: &mut ModuleItem) {
        n.visit_mut_children_with(self);

        if !n.is_module_decl() {
            self.new_stmts.push(n.clone());
        }
    }

    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);

        *stmts = self.new_stmts.clone();
    }
}

fn is_hit_rule(cur_import: &ImportDecl, target: &Config) -> bool {
    match target {
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

fn is_empty_decl(decl: &ImportDecl) -> bool {
    decl.specifiers.len() == 0 && decl.src.value == JsWord::from("".to_string())
}

fn get_new_src(named_import_spec: &ImportNamedSpecifier) -> String {
    if named_import_spec.imported.is_none() {
        (&named_import_spec.local.sym).to_string()
    } else {
        match &named_import_spec.imported.clone().unwrap() {
            ModuleExportName::Ident(ident) => {
                (&ident.sym).to_string()
            }
            ModuleExportName::Str(str) => {
                (&str.value).to_string()
            }
        }
    }
}

fn create_default_import_decl(src: String, local: Ident) -> ModuleItem {
    let import_decl = ImportDecl {
        src: Box::new(quote_str!(src)),
        specifiers: vec![ImportSpecifier::Default(
            ImportDefaultSpecifier {
                span: DUMMY_SP,
                local,
            }
        )],
        span: DUMMY_SP,
        type_only: false,
        with: None,
    };
    ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl))
}

fn create_named_import_decl(src: String, specifiers: Vec<ImportSpecifier>) -> ModuleItem {
    let import_decl = ImportDecl {
        src: Box::new(quote_str!(src)),
        specifiers,
        span: DUMMY_SP,
        type_only: false,
        with: None,
    };
    ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl))
}
