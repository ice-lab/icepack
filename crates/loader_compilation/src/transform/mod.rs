use either::Either;
use swc_core::common::chain;
use swc_core::ecma::{
  transforms::base::pass::noop,
  visit::{as_folder, Fold, VisitMut},
  ast::Module,
};

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
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
  export_names: Vec<String>,
}

pub struct RemoveExport {
  remove_names: Vec<String>,
}

pub struct SwcPluginOptions {
  pub keep_export: Option<KeepExportOptions>,
  pub remove_export: Option<RemoveExport>,
}

pub(crate) fn transform<'a>(plugin_options: SwcPluginOptions) -> impl Fold + 'a {
  chain!(
    either!(plugin_options.keep_export, |_| {
      keep_export()
    }),
    either!(plugin_options.remove_export, |_| {
      keep_export()
    }),
  )
}

struct KeepExport;

impl VisitMut for KeepExport {
  fn visit_mut_module(&mut self, module: &mut Module) {
    
  }
}

fn keep_export() -> impl Fold {
  as_folder(KeepExport)
}