# esbuild-config

Config files for [esbuild](https://github.com/evanw/esbuild).

## Why?

esbuild is an incredible tool, that is [exclusively using command line parameters](https://github.com/evanw/esbuild/issues/39) as a configuration syntax. Some people prefer configuration files, so I thought it could be a good idea to provide a solution for this. It is also for me a pretext to use Rust while learning it :)

## Usage

The esbuild-config command outputs a list of parameters based on a `esbuild.config.json` file, that can get passed to esbuild directly:

```console
esbuild $(esbuild-config)
```

It detects the presence of `esbuild.config.json` in the current directory, or the project root (using the presence of a `package.json` file). Any file can also get provided as a parameter:

```console
esbuild $(esbuild-config ./my-conf.json)
```

## Install

You have different options to install esbuild-config.

### npm

Install globally with npm using the following command:

```console
npm install --global esbuild-config
```

You can also add it to your project:

```console
npm install --save-dev esbuild-config
```

### Cargo

Install it with [Cargo](https://github.com/rust-lang/cargo) using the following command:

```console
cargo install esbuild-config
```

### Binaries

You can download the precompiled binaries [from the release page](https://github.com/bpierre/esbuild-config/releases).

### From source

To clone the repository and build esbuild-config, run these commands ([after having installed Rust](https://www.rust-lang.org/tools/install)):

```console
git clone git@github.com:bpierre/esbuild-config.git
cd esbuild-config
cargo build --release
```

The compiled binary is at `target/release/esbuild-config`.

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

## Contribute

```console
# Run the app
cargo run

# Run the tests
cargo test

# Generate the code coverage report
cargo tarpaulin -o Html
```

## Special thanks

[esbuild](https://github.com/evanw/esbuild) and [its author](https://github.com/evanw) obviously, not only for esbuild itself but also for its approach to [install a platform-specific binary through npm](https://github.com/evanw/esbuild/blob/1336fbcf9bcca2f2708f5f575770f13a8440bde3/lib/install.ts), that esbuild-config is also using.

## License

[MIT](./LICENSE)
