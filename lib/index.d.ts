import * as RspackCore from '@rspack/core';

// Re-export types from @rspack/core for convenience
export type { Plugin, Compiler, Compilation, AssetInfo } from '@rspack/core';

// Import SWC types for better type definitions
import type {
  Config as SwcConfig,
  JscConfig,
  ModuleConfig,
  EnvConfig,
} from '@swc/types';

/**
 * Configuration options for the compilation loader.
 * These options are passed to the builtin:compilation-loader.
 */
export interface CompilationLoaderOptions {
  /**
   * Source map configuration for the compiled code.
   */
  sourceMaps?: SwcConfig['sourceMaps'];

  /**
   * Environment preset configuration for SWC.
   */
  env?: EnvConfig;

  /**
   * Test pattern to match files for compilation.
   */
  test?: SwcConfig['test'];

  /**
   * Exclude pattern to skip files from compilation.
   */
  exclude?: SwcConfig['exclude'];

  /**
   * JSC configuration for SWC.
   */
  jsc?: JscConfig;

  /**
   * Module configuration for SWC.
   */
  module?: ModuleConfig;

  /**
   * Minification configuration.
   */
  minify?: boolean;

  /**
   * Custom compilation rules for excluding files.
   */
  compileRules?: {
    /**
     * Patterns to exclude from compilation.
     */
    exclude?: string[];
  };

  /**
   * Transform features configuration.
   */
  transformFeatures?: {
    /**
     * Environment variable replacements.
     */
    envReplacement?: string[];

    /**
     * Exports to keep during transformation.
     */
    keepExport?: string[];

    /**
     * Exports to remove during transformation.
     */
    removeExport?: string[];

    /**
     * Named import transformation configuration.
     */
    namedImportTransform?: {
      /**
       * Package names to apply named import transformation.
       */
      packages: string[];
    };

    /**
     * Package import changes.
     */
    changePackageImport?: Array<string | { [key: string]: string }>;

    /**
     * Platform-specific transformations.
     */
    keepPlatform?: string[];
  };
}

/**
 * CompilationLoaderPlugin class that provides a builtin:compilation-loader for Rspack.
 * This plugin enables advanced JavaScript/TypeScript compilation with custom transformations.
 */
declare class CompilationLoaderPlugin {
  constructor();
}

/**
 * ManifestPlugin class that generates an assets manifest for the compilation.
 * This plugin creates an assets-manifest.json file containing information about
 * pages, entries, assets, and public paths.
 */
declare class ManifestPlugin {
  constructor();
}

declare const core: typeof RspackCore & {
  CompilationLoaderPlugin: typeof CompilationLoaderPlugin;
  ManifestPlugin: typeof ManifestPlugin;
};

export = core;
