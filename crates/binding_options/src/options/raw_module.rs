use std::sync::Arc;

use loader_barrel::BARREL_LOADER_IDENTIFIER;
use loader_compilation::COMPILATION_LOADER_IDENTIFIER;
use rspack_core::BoxLoader;
use rspack_loader_react_refresh::REACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_swc::SWC_LOADER_IDENTIFIER;

pub fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  if builtin.starts_with(SWC_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_swc::SwcLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:swc-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }
  if builtin.starts_with(REACT_REFRESH_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_react_refresh::ReactRefreshLoader::default().with_identifier(builtin.into()),
    );
  }

  if builtin.starts_with(COMPILATION_LOADER_IDENTIFIER) {
    return Arc::new(
      loader_compilation::CompilationLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:compilation-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }

  if builtin.starts_with(BARREL_LOADER_IDENTIFIER) {
    return Arc::new(
      loader_barrel::BarrelLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:barrel-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }

  unreachable!("Unexpected builtin loader: {builtin}")
}
