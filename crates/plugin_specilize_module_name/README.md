# plugin_specilize_module_name

## Introduction

This is a plugin that specializes the module name of the generated code. It use the rspack `before_resolve` plugin hook to do this.

## Usage

1. Add the following configuration to your `Cargo.toml` file

```toml
[plugins]
plugin_specilize_module_name = { version = "0.1.0", default-features = false }
```

2. Add the plugin to rspack-core and pass the argument(the package names you want to specialize)

```rs
Box::new(SpecilizeModuleNamePlugin::new(Some(vec![<your package name>])))
```

## Example

You put the `lodash` as point package name in rspack-core

```rs
Box::new(SpecilizeModuleNamePlugin::new(Some(vec!["lodash"])))
```

Then you can use it in your code

```js
const { add } = require("lodash");
```

The package name will be transformed to `lodash.js?id=1`. The `id` is unique for each package.

## License
MIT