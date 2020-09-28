# esbuild-config

Config files for [esbuild](https://github.com/evanw/esbuild).

## Why?

esbuild is an incredible tool, that is [using command line parameters](https://github.com/evanw/esbuild/issues/39) as a configuration syntax. This is fine, but some people might prefer using a configuration file.

esbuild-config can transform a `esbuild.config.json` configuration file like this one:

```json
{
  "entry": "./index.js",
  "outfile": "./bundle.js",
  "external": ["react", "react-dom"],
  "loader": { ".js": "jsx", ".png": "base64" },
  "minify": true
}
```

Into a set of parameters for `esbuild`:

```console
--outfile=./bundle.js --minify --external:react --external:react-dom --loader:.js=jsx --loader:.png=base64 ./index.js
```

Which means that `esbuild` can read a static configuration by running it this way:

```console
esbuild $(esbuild-config)
```

## Usage

The esbuild-config command outputs a list of parameters based on a `esbuild.config.json` file, that can get passed to esbuild directly:

```console
esbuild $(esbuild-config)
```

It detects the presence of `esbuild.config.json` in the current directory or the project root (using the presence of a `package.json` file). The same configuration format can also get defined in the `package.json` file, using the `esbuild` field.json` file.

A specific file path can also get passed as a parameter:

```console
esbuild $(esbuild-config ./my-conf.json)
```

## Syntax

esbuild-config doesn’t do any validation on the configuration values: it only converts JSON types into arguments that are compatible with the format esbuild uses for its arguments. This makes it independent from esbuild versions, assuming the format doesn’t change.

The only exception to this is the `entry` field, which gets converted into a list of file names (when an array is provided) or a single file name (when a string is provided).

This is how JSON types get converted:

```json
{
  "entry": "./index.js",
  "outfile": "./bundle.js",
  "external": ["react", "react-dom"],
  "loader": { ".js": "jsx", ".png": "base64" },
  "minify": true
}
```

Output:

```console
--outfile=./bundle.js --minify --external:react --external:react-dom --loader:.js=jsx --loader:.png=base64 ./index.js
```

Notice how the entry, `./index.js`, has been moved to the end. esbuild-config also takes care of escaping the parameters as needed (e.g. by adding quotes).

## Install

### npm

The easiest way to install esbuild-config is through npm.

Install it globally using the following command:

```console
npm install --global esbuild-config
```

Or add it to your project:

```console
npm install --save-dev esbuild-config
```

See below for [alternative installation methods](#other-installation-methods).

### Binaries

You can download the precompiled binaries [from the release page](https://github.com/bpierre/esbuild-config/releases).

### Cargo

Install it with [Cargo](https://github.com/rust-lang/cargo) using the following command:

```console
cargo install esbuild-config
```

### From source

To clone the repository and build esbuild-config, run these commands ([after having installed Rust](https://www.rust-lang.org/tools/install)):

```console
git clone git@github.com:bpierre/esbuild-config.git
cd esbuild-config
cargo build --release
```

The compiled binary is at `target/release/esbuild-config`.

## Contribute

```console
# Run the app
cargo run

# Run the tests
cargo test

# Generate the code coverage report (install cargo-tarpaulin first)
cargo tarpaulin -o Html
```

## FAQ

### Doesn’t esbuild already support config files?

The recommended way to use a configuration file with esbuild is [through its Node.js API](https://github.com/evanw/esbuild/blob/1336fbcf9bcca2f2708f5f575770f13a8440bde3/docs/js-api.md), using a Node program as a configuration file:

```js
const { build } = require('esbuild')

build({
  entryPoints: ['./index.js'],
  outfile: './bundle.js',
  external: ['react', 'react-dom'],
  loader: { '.js': 'jsx', '.png': 'base64' },
  minify: true,
}).catch((error) => {
  console.error(error)
  process.exit(1)
})
```

If it works for you, you don’t need esbuild-config: the esbuild module already comes bundled with this JS API. esbuild-config provides an alternative way to configure esbuild. Instead of using the esbuild API through Node.js, it converts a [configuration file](#syntax) into command line parameters, that can be passed directly to the esbuild binary.

There are several reasons why you might want to use esbuild-config:

- You prefer using JSON as a configuration language.
- You prefer to have as much configuration as possible in the package.json.
- You prefer to not launch Node at all in the process.

## Special thanks

[esbuild](https://github.com/evanw/esbuild) and [its author](https://github.com/evanw) obviously, not only for esbuild itself but also for its approach to [install a platform-specific binary through npm](https://github.com/evanw/esbuild/blob/1336fbcf9bcca2f2708f5f575770f13a8440bde3/lib/install.ts), that esbuild-config is also using.

## License

[MIT](./LICENSE)
