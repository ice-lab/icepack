use crate::config::{Config, ImportType};
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
    // 用户配置
    pub options: Vec<Config>,
    // 全新生成的导入声明
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
                Config::LiteralConfig(src) => {
                    if is_hit_rule(import_decl, option) {
                        for specifier in &import_decl.specifiers {
                            match specifier {
                                ImportSpecifier::Named(named_import_spec) => {
                                    let mut import_new_src = src.clone();
                                    import_new_src.push_str("/");
                                    import_new_src.push_str(&get_import_module_name(named_import_spec));

                                    self.new_stmts.push(create_default_import_decl(import_new_src, named_import_spec.local.clone()));
                                }
                                _ => ()
                            }
                        }
                        // 清空当前导入声明，否则会被作为有效的声明添加至 new_stmts 中
                        import_decl.take();
                        break;
                    }
                }
                Config::SpecificConfig(config) => {
                    if is_hit_rule(import_decl, option) {
                        let target_fields: Vec<&String> = config.map.keys().clone().collect();
                        let mut named_import_spec_copy = import_decl.clone();

                        // 获取未命中规则的导入声明
                        named_import_spec_copy.specifiers = named_import_spec_copy.specifiers.into_iter().filter(|specifier| {
                            match specifier {
                                ImportSpecifier::Named(named_import_spec) => {
                                    let import_object_name = get_import_module_name(named_import_spec);
                                    !target_fields.contains(&&import_object_name)
                                }
                                _ => true
                            }
                        }).collect::<Vec<_>>();

                        if named_import_spec_copy.specifiers.len() != 0 {
                            self.new_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(named_import_spec_copy)));
                        }

                        for (target, rules) in config.map.iter() {
                            for specifier in &import_decl.specifiers {
                                match specifier {
                                    ImportSpecifier::Named(named_import_spec) => {
                                        let import_object_name = get_import_module_name(named_import_spec);
                                        if target == &import_object_name {
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
                        break;
                    }
                }
            }
        }

        if !is_empty_decl(import_decl) {
            self.new_stmts.push(wrap_with_moudle_item(import_decl.clone()));
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

fn is_empty_decl(decl: &ImportDecl) -> bool {
    decl.specifiers.len() == 0 && decl.src.value == JsWord::from("".to_string())
}

fn get_import_module_name(named_import_spec: &ImportNamedSpecifier) -> String {
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
    wrap_with_moudle_item(
        ImportDecl {
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
        }
    )
}

fn create_named_import_decl(src: String, specifiers: Vec<ImportSpecifier>) -> ModuleItem {
    wrap_with_moudle_item(
        ImportDecl {
            src: Box::new(quote_str!(src)),
            specifiers,
            span: DUMMY_SP,
            type_only: false,
            with: None,
        }
    )
}

fn wrap_with_moudle_item(import_decl: ImportDecl) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl))
}