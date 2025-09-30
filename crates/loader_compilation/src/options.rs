use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsRefStrConverter},
};
use serde::Deserialize;
use swc_config::{file_pattern::FilePattern, types::BoolConfig};
use swc_core::base::config::{
  Config, ErrorConfig, FileMatcher, InputSourceMap, IsModule, JscConfig, ModuleConfig, Options,
  SourceMapsConfig,
};

// Compile rules for excluding files from compilation
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompileRules {
  // Built-in rules to exclude files from compilation, such as react, react-dom, etc.
  pub exclude: Option<Vec<String>>,
}

// Transform feature options for custom transformations
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TransformFeatures {
  pub env_replacement: Option<Vec<String>>,
  pub keep_export: Option<Vec<String>>,
  pub remove_export: Option<Vec<String>>,
  pub named_import_transform: Option<NamedImportTransformConfig>,
  pub change_package_import: Option<Vec<ChangeConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct NamedImportTransformConfig {
  pub packages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum ChangeConfig {
  LiteralConfig(String),
}

// Raw options from JavaScript side
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompilationLoaderJsOptions {
  // Standard SWC options
  #[serde(default)]
  pub source_maps: Option<SourceMapsConfig>,

  pub source_map: Option<SourceMapsConfig>,
  
  #[serde(default)]
  pub env: Option<swc_core::ecma::preset_env::Config>,

  #[serde(default)]
  pub test: Option<FileMatcher>,

  #[serde(default)]
  pub exclude: Option<FileMatcher>,

  #[serde(default)]
  pub jsc: JscConfig,

  #[serde(default)]
  pub module: Option<ModuleConfig>,

  #[serde(default)]
  pub minify: BoolConfig<false>,

  #[serde(default)]
  pub input_source_map: Option<InputSourceMap>,

  #[serde(default)]
  pub inline_sources_content: BoolConfig<true>,

  #[serde(default)]
  pub emit_source_map_columns: BoolConfig<true>,

  #[serde(default)]
  pub error: ErrorConfig,

  #[serde(default)]
  pub is_module: Option<IsModule>,

  #[serde(rename = "$schema")]
  pub schema: Option<String>,

  #[serde(default)]
  pub source_map_ignore_list: Option<FilePattern>,

  // Our custom extensions
  #[serde(default)]
  pub compile_rules: Option<CompileRules>,

  #[serde(default)]
  pub transform_features: Option<TransformFeatures>,
}

#[cacheable(with=AsRefStr)]
#[derive(Debug)]
pub(crate) struct CompilationOptionsWithAdditional {
  raw_options: String,
  pub(crate) swc_options: Options,
  pub(crate) compile_rules: CompileRules,
  pub(crate) transform_features: TransformFeatures,
}

impl AsRefStrConverter for CompilationOptionsWithAdditional {
  fn as_str(&self) -> &str {
    &self.raw_options
  }
  fn from_str(s: &str) -> Self {
    s.try_into()
      .expect("failed to generate CompilationOptionsWithAdditional")
  }
}

const SOURCE_MAP_INLINE: &str = "inline";

impl TryFrom<&str> for CompilationOptionsWithAdditional {
  type Error = serde_json::Error;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let option: CompilationLoaderJsOptions = serde_json::from_str(value)?;
    let CompilationLoaderJsOptions {
      source_maps,
      source_map,
      env,
      test,
      exclude,
      jsc,
      module,
      minify,
      input_source_map,
      inline_sources_content,
      emit_source_map_columns,
      error,
      is_module,
      schema,
      source_map_ignore_list,
      compile_rules,
      transform_features,
    } = option;
    
    let mut source_maps: Option<SourceMapsConfig> = source_maps;
    if source_maps.is_none() && source_map.is_some() {
      source_maps = source_map
    }
    if let Some(SourceMapsConfig::Str(str)) = &source_maps {
      if str == SOURCE_MAP_INLINE {
        source_maps = Some(SourceMapsConfig::Bool(true))
      }
    }
    
    Ok(CompilationOptionsWithAdditional {
      raw_options: value.into(),
      swc_options: Options {
        config: Config {
          env,
          test,
          exclude,
          jsc,
          module,
          minify,
          input_source_map,
          source_maps,
          inline_sources_content,
          emit_source_map_columns,
          error,
          is_module,
          schema,
          source_map_ignore_list,
        },
        ..Default::default()
      },
      compile_rules: compile_rules.unwrap_or_default(),
      transform_features: transform_features.unwrap_or_default(),
    })
  }
}