use napi::bindgen_prelude::*;
use rspack_binding_builder_macros::register_plugin;
use rspack_core::BoxPlugin;

#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

// Export the CompilationLoaderPlugin
//
// The plugin needs to be wrapped with `require('@rspack/core').experiments.createNativePlugin`
// to be used in the host.
//
// `register_plugin` is a macro that registers a plugin.
//
// The first argument to `register_plugin` is the name of the plugin.
// The second argument to `register_plugin` is a resolver function that is called with `napi::Env` and the options returned from the resolver function from JS side.
//
// The resolver function should return a `BoxPlugin` instance.
register_plugin!(
  "CompilationLoaderPlugin",
  |_env: Env, _options: Unknown<'_>| {
    Ok(Box::new(loader_compilation::CompilationLoaderPlugin::new()) as BoxPlugin)
  }
);

// Export the ManifestPlugin
register_plugin!(
  "ManifestPlugin", 
  |_env: Env, _options: Unknown<'_>| {
    Ok(Box::new(plugin_manifest::ManifestPlugin::new()) as BoxPlugin)
  }
);
