/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

/* auto-generated by NAPI-RS */

const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'pack-binding.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.android-arm64.node')
          } else {
            nativeBinding = require('@ice/pack-binding-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'pack-binding.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.android-arm-eabi.node')
          } else {
            nativeBinding = require('@ice/pack-binding-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'pack-binding.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.win32-x64-msvc.node')
          } else {
            nativeBinding = require('@ice/pack-binding-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'pack-binding.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('@ice/pack-binding-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'pack-binding.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('@ice/pack-binding-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(join(__dirname, 'pack-binding.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./pack-binding.darwin-universal.node')
      } else {
        nativeBinding = require('@ice/pack-binding-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'pack-binding.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.darwin-x64.node')
          } else {
            nativeBinding = require('@ice/pack-binding-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'pack-binding.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.darwin-arm64.node')
          } else {
            nativeBinding = require('@ice/pack-binding-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(join(__dirname, 'pack-binding.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./pack-binding.freebsd-x64.node')
      } else {
        nativeBinding = require('@ice/pack-binding-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'pack-binding.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./pack-binding.linux-x64-musl.node')
            } else {
              nativeBinding = require('@ice/pack-binding-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'pack-binding.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./pack-binding.linux-x64-gnu.node')
            } else {
              nativeBinding = require('@ice/pack-binding-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'pack-binding.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./pack-binding.linux-arm64-musl.node')
            } else {
              nativeBinding = require('@ice/pack-binding-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'pack-binding.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./pack-binding.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('@ice/pack-binding-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        localFileExisted = existsSync(
          join(__dirname, 'pack-binding.linux-arm-gnueabihf.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./pack-binding.linux-arm-gnueabihf.node')
          } else {
            nativeBinding = require('@ice/pack-binding-linux-arm-gnueabihf')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { __chunk_inner_is_only_initial, __chunk_inner_can_be_initial, __chunk_inner_has_runtime, __chunk_inner_get_all_async_chunks, __chunk_inner_get_all_initial_chunks, __chunk_inner_get_all_referenced_chunks, __chunk_graph_inner_get_chunk_modules, __chunk_graph_inner_get_chunk_entry_modules, __chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable, __chunk_graph_inner_get_chunk_modules_iterable_by_source_type, __chunk_group_inner_get_chunk_group, __entrypoint_inner_get_runtime_chunk, DependenciesDto, EntryOptionsDto, EntryDataDto, JsEntries, JsCompilation, DependencyDto, DependenciesBlockDto, ModuleDto, JsResolver, JsRspackSeverity, JsStats, BuiltinPluginName, RawRuleSetConditionType, JsLoaderState, RegisterJsTapKind, JsResolverFactory, Rspack, registerGlobalTrace, cleanupGlobalTrace } = nativeBinding

module.exports.__chunk_inner_is_only_initial = __chunk_inner_is_only_initial
module.exports.__chunk_inner_can_be_initial = __chunk_inner_can_be_initial
module.exports.__chunk_inner_has_runtime = __chunk_inner_has_runtime
module.exports.__chunk_inner_get_all_async_chunks = __chunk_inner_get_all_async_chunks
module.exports.__chunk_inner_get_all_initial_chunks = __chunk_inner_get_all_initial_chunks
module.exports.__chunk_inner_get_all_referenced_chunks = __chunk_inner_get_all_referenced_chunks
module.exports.__chunk_graph_inner_get_chunk_modules = __chunk_graph_inner_get_chunk_modules
module.exports.__chunk_graph_inner_get_chunk_entry_modules = __chunk_graph_inner_get_chunk_entry_modules
module.exports.__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable = __chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable
module.exports.__chunk_graph_inner_get_chunk_modules_iterable_by_source_type = __chunk_graph_inner_get_chunk_modules_iterable_by_source_type
module.exports.__chunk_group_inner_get_chunk_group = __chunk_group_inner_get_chunk_group
module.exports.__entrypoint_inner_get_runtime_chunk = __entrypoint_inner_get_runtime_chunk
module.exports.DependenciesDto = DependenciesDto
module.exports.EntryOptionsDto = EntryOptionsDto
module.exports.EntryDataDto = EntryDataDto
module.exports.JsEntries = JsEntries
module.exports.JsCompilation = JsCompilation
module.exports.DependencyDto = DependencyDto
module.exports.DependenciesBlockDto = DependenciesBlockDto
module.exports.ModuleDto = ModuleDto
module.exports.JsResolver = JsResolver
module.exports.JsRspackSeverity = JsRspackSeverity
module.exports.JsStats = JsStats
module.exports.BuiltinPluginName = BuiltinPluginName
module.exports.RawRuleSetConditionType = RawRuleSetConditionType
module.exports.JsLoaderState = JsLoaderState
module.exports.RegisterJsTapKind = RegisterJsTapKind
module.exports.JsResolverFactory = JsResolverFactory
module.exports.Rspack = Rspack
module.exports.registerGlobalTrace = registerGlobalTrace
module.exports.cleanupGlobalTrace = cleanupGlobalTrace
