use crate::config::{Config, ImportType, MapProperty, SpecificConfigs};
use std::collections::HashMap;
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
        transforms::testing::test,
        visit::{as_folder, VisitMut, VisitMutWith},
    },
};

pub struct ModuleImportVisitor {
    pub options: Vec<Config>,
    // cache the new import declarations
    new_nodes: Vec<Option<ModuleItem>>,
}

impl ModuleImportVisitor {
    pub fn new(options: Vec<Config>) -> Self {
        Self {
            options,
            new_nodes: vec![],
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
                        if import_decl.specifiers.len() == 1 {
                            match &import_decl.specifiers[0] {
                                ImportSpecifier::Named(named_import_spec) => {
                                    if named_import_spec.is_type_only {
                                        return;
                                    }

                                    let new_src = get_new_src(named_import_spec, &import_decl);
                                    import_decl.src = Box::new(Str {
                                        span: named_import_spec.span,
                                        value: new_src.clone().into(),
                                        raw: Some(gen_raw_string(&new_src).into()),
                                    });
                                    import_decl.specifiers[0] = ImportSpecifier::Default(ImportDefaultSpecifier {
                                        span: named_import_spec.span,
                                        local: named_import_spec.local.clone(),
                                    });
                                }
                                _ => ()
                            }
                        } else {
                            for specifier in &import_decl.specifiers {
                                println!("[specifier] · · · -------------------> · · · {:?}", &specifier);
                                match specifier {
                                    ImportSpecifier::Named(named_import_spec) => {
                                        if named_import_spec.is_type_only {
                                            return;
                                        }
                                        let new_src: String = get_new_src(named_import_spec, &import_decl);
                                        let new_import_decl = create_default_import_decl(new_src, named_import_spec.local.clone());
                                        self.new_nodes.push(Some(new_import_decl));
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
                        println!("· · · -------------------> · · · Hit the specific rule!");
                        if import_decl.specifiers.len() == 1 {
                            for (_pkg_name, rules) in config.map.iter() {
                                let MapProperty { to, import_type, name } = rules;
                                let new_src = gen_path_string(config.name.clone(), to);

                                match &mut import_decl.specifiers[0] {
                                    ImportSpecifier::Named(named_import_spec) => {
                                        import_decl.src = Box::new(Str {
                                            span: named_import_spec.span,
                                            value: new_src.clone().into(),
                                            raw: Some(gen_raw_string(&new_src).into()),
                                        });
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
                            // Not support multiple case now 
                        }
                        break;
                    }
                }
            }
        }
    }

    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);
        // Remove the empty module
        stmts.retain(|stmt| {
            match stmt {
                ModuleItem::ModuleDecl(mod_decl) => {
                    match mod_decl.clone().import() {
                        Some(import) => {
                            if import.src.value.is_empty() {
                                return false;
                            }
                            true
                        },
                        None => false,
                    };
                    true
                }
                _ => true
            }
        });
        // Add the new import declarations
        self.new_nodes.clone().into_iter().for_each(|stmt| {
            if let Some(stmt) = stmt {
                stmts.push(stmt);
            }
        });
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

fn get_new_src(named_import_spec: &ImportNamedSpecifier, import_decl: &ImportDecl) -> String {
    if named_import_spec.imported.is_none() {
        gen_path_string((&import_decl.src.value).to_string(), &named_import_spec.local.sym)
    } else {
        match &named_import_spec.imported.clone().unwrap() {
            ModuleExportName::Ident(ident) => {
                gen_path_string((&import_decl.src.value).to_string(), &ident.sym)
            }
            ModuleExportName::Str(str) => {
                gen_path_string((&import_decl.src.value).to_string(), &str.value)
            }
        }
    }
}

fn gen_raw_string(content: &str) -> String {
    let mut s = String::new();
    s.push_str("\"");
    s.push_str(content);
    s.push_str("\"");
    s
}

fn gen_path_string(mut p1: String, p2: &str) -> String {
    p1.push_str("/");
    p1.push_str(p2);
    p1
}

fn create_default_import_decl(src: String, local: Ident) -> ModuleItem {
    let import_decl = ImportDecl {
        src: Box::new(Str {
            span: DUMMY_SP,
            value: src.clone().into(),
            raw: Some(gen_raw_string(&src).into()),
        }),
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

#[cfg(test)]
test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(vec![Config::LiteralConfig(String::from("y"))])),
    single_literal_transform,
    r#"import { x } from "y";"#
);

test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(
        vec![
            Config::LiteralConfig(String::from("y")),
            Config::LiteralConfig(String::from("c"))
        ]
    )),
    multiple_literal_transform,
    r#"
        import { x } from "y";
        import { a as b } from "c";
        import { n } from "m";

        console.log("hello world");
    "#
);

test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(
        vec![
            Config::LiteralConfig(String::from("y")),
            Config::LiteralConfig(String::from("i")),
        ]
    )),
    multiple_literal_transform_2,
    r#"
        import { x as a, z as b } from "y";
        // import { a, c, d } from "i";
    "#
);

test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(
        vec![Config::SpecificConfig(
            SpecificConfigs {
                name: String::from("y"),
                map: HashMap::from([
                    (
                        "x".to_string(), MapProperty {
                            to: String::from("m/n"),
                            import_type: Some(ImportType::Named),
                            name: Some(String::from("a")),
                        }
                    ),
                ])
            }
        )]
    )),
    single_specific_transform,
    r#"import { x } from "y";"#
);

test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(
        vec![Config::SpecificConfig(
            SpecificConfigs {
                name: String::from("y"),
                map: HashMap::from([
                    (
                        "x".to_string(), MapProperty {
                            to: String::from("m/n"),
                            import_type: Some(ImportType::Named),
                            name: Some(String::from("a")),
                        }
                    ),
                ])
            }
        )]
    )),
    single_specific_transform_import,
    r#"import { x as k } from "y";"#
);

test!(
    Default::default(),
    |_| as_folder(ModuleImportVisitor::new(
        vec![
            Config::LiteralConfig(String::from("antd")),
            Config::SpecificConfig(
                SpecificConfigs {
                    name: String::from("ice"),
                    map: HashMap::from([
                        (
                            "a".to_string(), MapProperty {
                                to: String::from("x/y"),
                                import_type: None,
                                name: None,
                            }
                        ),
                    ])
                }
            )
        ]
    )),
    mix_specific_transform,
    r#"
        import { Button, Spin } from "antd";
        import { a } from "ice";
        import { isArray } from "lodash";
    "#
);
