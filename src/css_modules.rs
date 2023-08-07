use std::{
  path::Path,
  hash::Hash,
};
use swc_atoms::JsWord;
use crate::hash::{HashFunction, HashSalt, CSSHash, HashDigest};
use crate::local_ident_name::{LocalIdentName, LocalIdentNameRenderOptions, PathData};

pub struct CSSModulesLocalIdent<'a> {
  filename: &'a Path,
  local_ident_name: &'a LocalIdentName,
}

impl<'a> CSSModulesLocalIdent<'a> {
  pub fn new(filename: &'a Path, local_ident_name: &'a LocalIdentName) -> Self {
    Self {
      filename,
      local_ident_name,
    }
  }

  pub fn get_new_name(&self, local: JsWord) -> String {
    let hash = {
      let mut hasher = CSSHash::with_salt(&HashFunction::MD4, &HashSalt::None);
      self.filename.hash(&mut hasher);
      local.hash(&mut hasher);
      let hash = hasher.digest(&HashDigest::Hex);
      let hash = hash.rendered(20);
      if hash.as_bytes()[0].is_ascii_digit() {
        format!("_{hash}")
      } else {
        hash.into()
      }
    };
    self
      .local_ident_name
      .render(LocalIdentNameRenderOptions {
        path_data: PathData::default()
          .filename(&self.filename.to_string_lossy())
          .hash(&hash),
        local: &local,
      })
      .into()
  }
}

