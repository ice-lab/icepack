use std::{
  str::FromStr,
  path::PathBuf,
  string::ParseError,
};
use regex::{Captures, Regex};
use once_cell::sync::Lazy;

#[derive(Default, Clone, Copy)]
pub struct PathData<'a> {
  pub filename: Option<&'a str>,
  pub hash: Option<&'a str>,
  pub local: Option<&'a str>,
}

impl<'a> PathData<'a> {
  pub fn filename(mut self, v: &'a str) -> Self {
    self.filename = Some(v);
    self
  }
  pub fn hash(mut self, v: &'a str) -> Self {
    self.hash = Some(v);
    self
  }
}

#[derive(Debug, Clone)]
pub struct LocalIdentName(Filename);

impl From<String> for LocalIdentName {
  fn from(value: String) -> Self {
    Self(Filename::from(value))
  }
}
pub struct LocalIdentNameRenderOptions<'a> {
  pub path_data: PathData<'a>,
  pub local: &'a str,
}

#[derive(Debug, Clone)]
pub struct Filename {
  template: String,
}

impl FromStr for Filename {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      template: s.to_string(),
    })
  }
}

impl From<String> for Filename {
  fn from(value: String) -> Self {
    Self { template: value }
  }
}

#[derive(Debug)]
pub struct ResourceParsedData {
  pub path: PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
}

pub const FILE_PLACEHOLDER: &str = "[file]";
pub const BASE_PLACEHOLDER: &str = "[base]";
pub const NAME_PLACEHOLDER: &str = "[name]";
pub const PATH_PLACEHOLDER: &str = "[path]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const QUERY_PLACEHOLDER: &str = "[query]";
pub const FRAGMENT_PLACEHOLDER: &str = "[fragment]";
pub const RUNTIME_PLACEHOLDER: &str = "[runtime]";
static PATH_QUERY_FRAGMENT_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^((?:\0.|[^?#\0])*)(\\?(?:\0.|[^#\0])*)?(#.*)?$")
    .expect("Failed to initialize `PATH_QUERY_FRAGMENT_REGEXP`")
});
pub static HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[hash(:(\d*))?]").expect("Invalid regex"));
pub static FULL_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[fullhash(:(\d*))?]").expect("Invalid regex"));
static ESCAPE_LOCAL_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\.]"#).expect("Invalid regex"));

pub fn parse_resource(resource: &str) -> Option<ResourceParsedData> {
  let groups = PATH_QUERY_FRAGMENT_REGEXP.captures(resource)?;

  Some(ResourceParsedData {
    path: groups.get(1)?.as_str().into(),
    query: groups.get(2).map(|q| q.as_str().to_owned()),
    fragment: groups.get(3).map(|q| q.as_str().to_owned()),
  })
}

impl LocalIdentName {
  pub fn render(&self, options: LocalIdentNameRenderOptions) -> String {
    self.0.render(options.path_data, options.local)
  }
}

impl Filename {
  pub fn template(&self) -> &str {
    &self.template
  }
  
  pub fn render(&self, options: PathData, local: &str) -> String {
    let mut template = self.template.clone();
    if let Some(filename) = options.filename {
      if let Some(ResourceParsedData {
        path: file,
        query,
        fragment,
      }) = parse_resource(filename)
      {
        template = template.replace(FILE_PLACEHOLDER, &file.to_string_lossy());
        template = template.replace(
          EXT_PLACEHOLDER,
          &file
            .extension()
            .map(|p| format!(".{}", p.to_string_lossy()))
            .unwrap_or_default(),
        );
        if let Some(base) = file.file_name().map(|p| p.to_string_lossy()) {
          template = template.replace(BASE_PLACEHOLDER, &base);
        }
        if let Some(name) = file.file_stem().map(|p| p.to_string_lossy()) {
          template = template.replace(NAME_PLACEHOLDER, &name);
        }
        template = template.replace(
          PATH_PLACEHOLDER,
          &file
            .parent()
            .map(|p| p.to_string_lossy())
            // "" -> "", "folder" -> "folder/"
            .filter(|p| !p.is_empty())
            .map(|p| p + "/")
            .unwrap_or_default(),
        );
        template = template.replace(QUERY_PLACEHOLDER, &query.unwrap_or_default());
        template = template.replace(FRAGMENT_PLACEHOLDER, &fragment.unwrap_or_default());
      }
    }
    if let Some(hash) = options.hash {
      for reg in [&HASH_PLACEHOLDER, &FULL_HASH_PLACEHOLDER] {
        template = reg
          .replace_all(&template, |caps: &Captures| {
            let hash = &hash[..hash_len(hash, caps)];
            hash
          })
          .into_owned();
      }
    }
    template = template.replace("[local]", local);
    template = ESCAPE_LOCAL_IDENT_REGEX.replace_all(&template, "-").into_owned();

    template
  }
}

fn hash_len(hash: &str, caps: &Captures) -> usize {
  let hash_len = hash.len();
  caps
    .get(2)
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(hash_len)
    .min(hash_len)
}

