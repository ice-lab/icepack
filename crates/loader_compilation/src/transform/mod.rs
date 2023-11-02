use either::Either;
use swc_core::common::chain;
use swc_core::ecma::{
  transforms::base::pass::noop,
  visit::{as_folder, Fold, VisitMut, Visit},
  ast::Module,
};

mod keep_export;
mod remove_export;

use keep_export::keep_export;
use remove_export::remove_export;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = $config {
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
  ($config:expr, $f:expr, $enabled:expr) => {
    if $enabled {
      either!($config, $f)
    } else {
      Either::Right(noop())
    }
  };
}

pub struct KeepExportOptions {
  pub export_names: Vec<String>,
}

pub struct RemoveExportOptions {
  pub remove_names: Vec<String>,
}

pub struct SwcPluginOptions {
  pub keep_export: Option<KeepExportOptions>,
  pub remove_export: Option<RemoveExportOptions>,
}

pub(crate) fn transform<'a>(plugin_options: SwcPluginOptions) -> impl Fold + 'a {
  chain!(
    either!(plugin_options.keep_export, |options: KeepExportOptions| {
      keep_export(options.export_names)
    }),
    either!(plugin_options.remove_export, |options: RemoveExportOptions| {
      remove_export(options.remove_names)
    }),
  )
}