use std::{
  env, fs,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
  vec,
};

use loader_barrel::{BarrelLoader, LoaderOptions};
use rspack_core::{
  run_loaders, CompilerContext, CompilerOptions, Loader, LoaderRunnerContext, PluginDriver,
  ResolverFactory, ResourceData, SideEffectOption,
};
use rspack_util::source_map::SourceMapKind;
use swc_core::base::config::Config;
use swc_core::ecma::ast::EsVersion;

async fn loader_test(actual: impl AsRef<Path>, expected: impl AsRef<Path>) {
  let tests_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"))).join("tests");
  let expected_path = tests_path.join(expected);
  let actual_path = tests_path.join(actual);

  let mut options = Config::default();
  options.jsc.target = Some(EsVersion::Es2020);

  let compiler_options = CompilerOptions {
    bail: false,
    context: rspack_core::Context::default(),
    dev_server: rspack_core::DevServerOptions::default(),
    mode: rspack_core::Mode::None,
    output: rspack_core::OutputOptions {
      clean: false,
      path: Default::default(),
      public_path: Default::default(),
      filename: rspack_core::Filename::from_str("").expect("TODO:"),
      asset_module_filename: rspack_core::Filename::from_str("").expect("TODO:"),
      wasm_loading: rspack_core::WasmLoading::Disable,
      webassembly_module_filename: rspack_core::Filename::from_str("").expect("TODO:"),
      chunk_filename: rspack_core::Filename::from_str("").expect("TODO:"),
      cross_origin_loading: rspack_core::CrossOriginLoading::Disable,
      unique_name: Default::default(),
      chunk_loading: rspack_core::ChunkLoading::Enable(rspack_core::ChunkLoadingType::Jsonp),
      chunk_loading_global: "webpackChunkwebpack".to_string(),
      css_chunk_filename: rspack_core::Filename::from_str("").expect("TODO:"),
      css_filename: rspack_core::Filename::from_str("").expect("TODO:"),
      hot_update_chunk_filename: rspack_core::Filename::from_str("").expect("Should exist"),
      hot_update_main_filename: rspack_core::Filename::from_str("").expect("Should exist"),
      hot_update_global: "webpackHotUpdate".to_string(),
      library: None,
      enabled_library_types: None,
      strict_module_error_handling: false,
      global_object: "self".to_string(),
      import_function_name: "import".to_string(),
      iife: true,
      module: false,
      trusted_types: None,
      source_map_filename: rspack_core::Filename::from_str("./a.map.js").expect("TODO:"),
      hash_function: rspack_core::HashFunction::Xxhash64,
      hash_digest: rspack_core::HashDigest::Hex,
      hash_digest_length: 16,
      hash_salt: rspack_core::HashSalt::None,
      async_chunks: true,
      worker_chunk_loading: rspack_core::ChunkLoading::Enable(
        rspack_core::ChunkLoadingType::ImportScripts,
      ),
      worker_wasm_loading: rspack_core::WasmLoading::Disable,
      worker_public_path: String::new(),
      script_type: String::from("false"),
    },
    target: rspack_core::Target::new(&vec![String::from("web")]).expect("TODO:"),
    resolve: rspack_core::Resolve::default(),
    resolve_loader: rspack_core::Resolve::default(),
    builtins: Default::default(),
    module: Default::default(),
    stats: Default::default(),
    cache: Default::default(),
    snapshot: Default::default(),
    experiments: Default::default(),
    node: Default::default(),
    optimization: rspack_core::Optimization {
      remove_available_modules: false,
      side_effects: SideEffectOption::False,
      provided_exports: Default::default(),
      used_exports: Default::default(),
      inner_graph: Default::default(),
      mangle_exports: Default::default(),
      concatenate_modules: Default::default(),
    },
    profile: false,
  };

  let (plugin_driver, compiler_options) = PluginDriver::new(
    compiler_options,
    vec![],
    Arc::new(ResolverFactory::new(rspack_core::Resolve {
      extensions: Some(vec![".js".to_string()]),
      ..Default::default()
    })),
    &mut Default::default(),
    &mut Default::default(),
  );

  let (result, _) = run_loaders(
    &[Arc::new(BarrelLoader::new(LoaderOptions {
      cache_dir: None,
      names: vec!["a".to_string()],
    })) as Arc<dyn Loader<LoaderRunnerContext>>],
    &ResourceData::new(actual_path.to_string_lossy().to_string(), actual_path),
    &[],
    CompilerContext {
      options: Arc::clone(&compiler_options),
      resolver_factory: Arc::new(ResolverFactory::new(rspack_core::Resolve {
        extensions: Some(vec![".js".to_string()]),
        ..Default::default()
      })),
      module: "".into(),
      module_context: None,
      module_source_map_kind: SourceMapKind::SourceMap,
      factorize_queue: Default::default(),
      add_queue: Default::default(),
      build_queue: Default::default(),
      process_dependencies_queue: Default::default(),
      build_time_execution_queue: Default::default(),
      plugin_driver,
      cache: Arc::new(rspack_core::cache::Cache::new(compiler_options)),
    },
    Default::default(),
  )
  .await
  .expect("TODO:")
  .split_into_parts();
  let result = result.content.try_into_string().expect("TODO:");

  if env::var("UPDATE_BARREL_TEST").is_ok() {
    fs::write(expected_path, result).expect("TODO:");
  } else {
    let expected = fs::read_to_string(expected_path).expect("TODO:");
    assert_eq!(result, expected);
  }
}

#[tokio::test]
async fn basic() {
  loader_test("fixtures/basic/input.js", "fixtures/basic/output.js").await;
}

#[tokio::test]
async fn import() {
  loader_test(
    "fixtures/import-all/input.js",
    "fixtures/import-all/output.js",
  )
  .await;
}

#[tokio::test]
async fn named() {
  loader_test("fixtures/named/input.js", "fixtures/named/output.js").await;
}

#[tokio::test]
async fn client() {
  loader_test(
    "fixtures/use-client/input.js",
    "fixtures/use-client/output.js",
  )
  .await;
}
