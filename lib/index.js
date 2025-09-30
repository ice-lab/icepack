process.env.RSPACK_BINDING = require('node:path').dirname(
  require.resolve('@ice/pack-binding')
);

const binding = require('@ice/pack-binding');

// Register the plugin `CompilationLoaderPlugin` exported by `crates/binding/src/lib.rs`.
binding.registerCompilationLoaderPlugin();

// Register the plugin `ManifestPlugin` exported by `crates/binding/src/lib.rs`.
binding.registerManifestPlugin();

const core = require('@rspack/core');

/**
 * Creates a wrapper for the plugin `ManifestPlugin` exported by `crates/binding/src/lib.rs`.
 *
 * Check out `crates/binding/src/lib.rs` for the original plugin definition.
 * This plugin is used in `examples/use-plugin/build.js`.
 *
 * @example
 * ```js
 * const ManifestPlugin = require('@ice/pack-core').ManifestPlugin;
 * ```
 *
 * `createNativePlugin` is a function that creates a wrapper for the plugin.
 *
 * The first argument to `createNativePlugin` is the name of the plugin.
 * The second argument to `createNativePlugin` is a resolver function.
 *
 * Options used to call `new ManifestPlugin` will be passed as the arguments to the resolver function.
 * The return value of the resolver function will be used to initialize the plugin in `ManifestPlugin` on the Rust side.
 *
 */
const ManifestPlugin = core.experiments.createNativePlugin(
  'ManifestPlugin',
  function (options) {
    return options;
  }
);

Object.defineProperty(core, 'ManifestPlugin', {
  value: ManifestPlugin,
});

/**
 * Creates a wrapper for the plugin `CompilationLoaderPlugin` exported by `crates/binding/src/lib.rs`.
 *
 * Check out `crates/binding/src/lib.rs` for the original plugin definition.
 * This plugin is used in `examples/use-loader/build.js`.
 *
 * @example
 * ```js
 * const CompilationLoaderPlugin = require('@ice/pack-core').CompilationLoaderPlugin;
 * ```
 *
 * `createNativePlugin` is a function that creates a wrapper for the plugin.
 *
 * The first argument to `createNativePlugin` is the name of the plugin.
 * The second argument to `createNativePlugin` is a resolver function.
 *
 * Options used to call `new CompilationLoaderPlugin` will be passed as the arguments to the resolver function.
 * The return value of the resolver function will be used to initialize the plugin in `CompilationLoaderPlugin` on the Rust side.
 *
 */
const CompilationLoaderPlugin = core.experiments.createNativePlugin(
  'CompilationLoaderPlugin',
  function (options) {
    return options;
  }
);

Object.defineProperty(core, 'CompilationLoaderPlugin', {
  value: CompilationLoaderPlugin,
});

module.exports = core;
