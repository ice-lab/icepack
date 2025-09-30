# Rspack Binding - icepack-based Implementation

This project contains Rust crates that implement loaders and plugins for Rspack, based on the icepack implementation but adapted for the latest Rspack APIs.

## Crates

### loader_compilation

Location: `crates/loader_compilation/`

A JavaScript/TypeScript compilation loader that provides code transformation capabilities.

**Features:**
- Transform features configuration (remove_export, keep_export, optimize_barrel, etc.)
- Compile rules configuration (exclude patterns)
- Compatible with Rspack 0.5.0 API
- Implements the standard Rspack Loader trait

**Usage:**
```rust
use loader_compilation::{CompilationLoader, COMPILATION_LOADER_IDENTIFIER};

let loader = CompilationLoader::new(r#"{
  "transformFeatures": {
    "removeExport": true,
    "optimizeBarrel": true
  },
  "compileRules": {
    "exclude": ["node_modules"]
  }
}"#)?;
```

### plugin_manifest

Location: `crates/plugin_manifest/`

A webpack-style assets manifest plugin that generates a JSON manifest of build assets.

**Features:**
- Generates `assets-manifest.json` containing pages, entries, and assets information
- Supports public path configuration
- Detects data-loader files
- Compatible with Rspack 0.5.0 API
- Implements the standard Rspack Plugin trait

**Usage:**
```rust
use plugin_manifest::ManifestPlugin;

let plugin = ManifestPlugin::new();
```

## Key Changes from Original icepack Implementation

1. **Updated Imports**: 
   - `LoaderContext` and `Loader` now imported from `rspack_core`
   - `Identifier` imported from `rspack_collections`
   - `rspack_sources` is now a separate crate

2. **API Compatibility**:
   - Loader trait now requires an `identifier()` method
   - Plugin `apply` method signature updated for new ApplyContext
   - Asset handling APIs updated for new rspack_sources structure

3. **Simplified Structure**:
   - Removed complex SWC integration for initial implementation
   - Focus on basic transformation pipeline
   - Plugin functionality can be re-added when needed

## Dependencies

The crates use workspace dependencies defined in the root `Cargo.toml`:
- rspack_cacheable = "0.5.0"
- rspack_collections = "0.5.0"  
- rspack_core = "0.5.0"
- rspack_error = "0.5.0"
- rspack_sources = "0.4.8"
- async-trait = "0.1"

## Building

```bash
cargo check  # Check compilation
cargo build  # Build all crates
```

Both crates compile successfully with only minor warnings about unused fields.

## Future Enhancements

- Add full SWC integration for advanced JavaScript transformations
- Implement CompilationLoaderPlugin for loader registration
- Add more transform features (env replacement, named import transforms, etc.)
- Add comprehensive tests
- Add documentation and examples

---

## Original Rspack Binding Template Information

**🚀 Unlock native Rust speed for Rspack — supercharge your builds, keep every JS feature, zero compromise, no limits.**

### Features

- 🦀 Write your own Rspack plugins and loaders in Rust
- 🧩 Inherit all Rspack features and JavaScript API
- 🛡️ Secure supply chain with npm provenance
- 📦 Effortless publishing: just set your `NPM_TOKEN`

### Quick Start

📖 **[Create custom binding](https://rspack-contrib.github.io/rspack-rust-book/custom-binding/getting-started/index.html)**

### Why?

Rspack achieves high performance by being written in Rust, but using its JavaScript API introduces overhead due to cross-language calls. This can limit performance and access to native Rust features.

_Rspack Custom Binding_ allows you to extend Rspack directly with native Rust code, avoiding the JavaScript layer and unlocking full performance and flexibility.

With custom binding, you can still use the familiar JavaScript API (`@rspack/core`), but your custom logic runs natively, combining the best of both worlds.

Check out [rationale](https://rspack-contrib.github.io/rspack-rust-book/custom-binding/getting-started/rationale.html) for more details.

## Supported Platforms

| Target                        | Host Runner    | Notes               |
| ----------------------------- | -------------- | ------------------- |
| x86_64-apple-darwin           | macos-latest   | macOS Intel         |
| aarch64-apple-darwin          | macos-latest   | macOS Apple Silicon |
| x86_64-pc-windows-msvc        | windows-latest | Windows 64-bit      |
| i686-pc-windows-msvc          | windows-latest | Windows 32-bit      |
| aarch64-pc-windows-msvc       | windows-latest | Windows ARM64       |
| x86_64-unknown-linux-gnu      | ubuntu-22.04   | Linux x64 (GNU)     |
| x86_64-unknown-linux-musl     | ubuntu-22.04   | Linux x64 (musl)    |
| aarch64-unknown-linux-gnu     | ubuntu-22.04   | Linux ARM64 (GNU)   |
| aarch64-unknown-linux-musl    | ubuntu-22.04   | Linux ARM64 (musl)  |
| armv7-unknown-linux-gnueabihf | ubuntu-22.04   | Linux ARMv7         |
| aarch64-linux-android         | ubuntu-22.04   | Android ARM64       |
| armv7-linux-androideabi       | ubuntu-22.04   | Android ARMv7       |

> **Note:** Node.js support requires >= 18.
>
> Multi-platform publishing and CI support is powered by [rspack-toolchain](https://github.com/rspack-contrib/rspack-toolchain). For the latest supported platforms, see the [official supported targets list](https://github.com/rspack-contrib/rspack-toolchain/tree/main?tab=readme-ov-file#supported-targets).
