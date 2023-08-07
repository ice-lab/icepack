#![deny(clippy::all)]


pub mod hash;
pub mod css_modules;
pub mod local_ident_name;
use std::path::Path;
use swc_atoms::JsWord;
use crate::css_modules::CSSModulesLocalIdent;
use crate::local_ident_name::LocalIdentName;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn get_css_modules_local_ident(filepath: String, local: String, template: String) -> String {
  let filename = Path::new(&filepath);
  let local_ident_name = LocalIdentName::from(template);
  let str = CSSModulesLocalIdent::new(filename, &local_ident_name);
  str.get_new_name(JsWord::from(local))
}
