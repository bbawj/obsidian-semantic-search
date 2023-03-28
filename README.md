# Obsidian Sample Plugin (Rust)

This is a quick proof of concept combining the current [Obsidian Git template](https://github.com/obsidianmd/obsidian-sample-plugin) and an [earlier proof-of-concept](https://github.com/trashhalo/obsidian-rust-plugin). Thanks to @trashhalo who essentially did all the work!

How to use:

1. Create a new repo with this one as a template.
2. Make sure Rust & cargo are installed (I used `rustup`).
3. Install `wasm-pack` [link](https://rustwasm.github.io/wasm-pack/installer/).
4. Follow the steps here to set up the sample plugin: [Obsidian plugin setup](https://marcus.se.net/obsidian-plugin-docs/getting-started/create-your-first-plugin), ideally using the `yarn` instructions (I haven't tested the npm version).

Setting up yarn will install and run the `esbuild-plugin-wasm-pack` plugin, which compiles the Rust code (in `src/`) and puts the resulting packed WebAssembly into `pkg/` (NOTE: this generated folder will have a `.gitignore`, which you will need to remove to distribute your plugin). The `main.ts` file then loads that code.

To test: try running the "Sample Plugin: Example Command" in the command bar (i.e. Cmd+P); you should see a "hello from rust" message.

Note: currently there is a bug (probably with esbuild-plugin-wasm-pack) where esbuild doesn't wait for output to be generated (I think); the solution for now is just to rerun it:

```bash
(prod) test_repo> yarn run dev
yarn run v1.22.17
$ node esbuild.config.mjs

ℹ  Compiling your crate in <default> mode...

[INFO]: Checking for the Wasm target...
[INFO]: Compiling to Wasm...
   Compiling proc-macro2 v1.0.49
   Compiling quote v1.0.23
   Compiling unicode-ident v1.0.6
   Compiling syn v1.0.107
   Compiling log v0.4.17
   Compiling wasm-bindgen-shared v0.2.83
   Compiling cfg-if v1.0.0
   Compiling bumpalo v3.11.1
   Compiling once_cell v1.16.0
   Compiling wasm-bindgen v0.2.83
   Compiling wasm-bindgen-backend v0.2.83
   Compiling wasm-bindgen-macro-support v0.2.83
   Compiling wasm-bindgen-macro v0.2.83
   Compiling js-sys v0.3.60
   Compiling obsidian-rust-plugin v0.1.0 (<repo>)
    Finished release [optimized] target(s) in 13.54s
node:internal/process/promises:246
          triggerUncaughtException(err, true /* fromPromise */);
          ^

[Error: ENOENT: no such file or directory, open '<repo>/target/wasm32-unknown-unknown/release/obsidian_rust_plugin.d'] {
  errno: -2,
  code: 'ENOENT',
  syscall: 'open',
  path: '<repo>/target/wasm32-unknown-unknown/release/obsidian_rust_plugin.d'
}

Node.js v17.0.1
[INFO]: Installing wasm-bindgen...
error Command failed with exit code 1.
info Visit https://yarnpkg.com/en/docs/cli/run for documentation about this command.
[INFO]: Optimizing wasm binaries with `wasm-opt`...
(prod) test_repo> [INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
[INFO]: :-) Done in 13.94s
[INFO]: :-) Your wasm pkg is ready to publish at <repo>/pkg.

(prod) test_repo> yarn run dev
yarn run v1.22.17
$ node esbuild.config.mjs

ℹ  Compiling your crate in <default> mode...

[INFO]: Checking for the Wasm target...
[INFO]: Compiling to Wasm...
    Finished release [optimized] target(s) in 0.06s
[INFO]: Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
[INFO]: :-) Done in 0.41s
[INFO]: :-) Your wasm pkg is ready to publish at <repo>/pkg.

✅  Your crate was successfully compiled.

[watch] build finished, watching for changes...
```

I'm leaving the original Obsidian plugin docs below in case it's helpful:

# Obsidian Sample Plugin docs

This is a sample plugin for Obsidian (https://obsidian.md).

This project uses Typescript to provide type checking and documentation.
The repo depends on the latest plugin API (obsidian.d.ts) in Typescript Definition format, which contains TSDoc comments describing what it does.

**Note:** The Obsidian API is still in early alpha and is subject to change at any time!

This sample plugin demonstrates some of the basic functionality the plugin API can do.
- Changes the default font color to red using `styles.css`.
- Adds a ribbon icon, which shows a Notice when clicked.
- Adds a command "Open Sample Modal" which opens a Modal.
- Adds a plugin setting tab to the settings page.
- Registers a global click event and output 'click' to the console.
- Registers a global interval which logs 'setInterval' to the console.

## First time developing plugins?

Quick starting guide for new plugin devs:

- Check if [someone already developed a plugin for what you want](https://obsidian.md/plugins)! There might be an existing plugin similar enough that you can partner up with.
- Make a copy of this repo as a template with the "Use this template" button (login to GitHub if you don't see it).
- Clone your repo to a local development folder. For convenience, you can place this folder in your `.obsidian/plugins/your-plugin-name` folder.
- Install NodeJS, then run `npm i` in the command line under your repo folder.
- Run `npm run dev` to compile your plugin from `main.ts` to `main.js`.
- Make changes to `main.ts` (or create new `.ts` files). Those changes should be automatically compiled into `main.js`.
- Reload Obsidian to load the new version of your plugin.
- Enable plugin in settings window.
- For updates to the Obsidian API run `npm update` in the command line under your repo folder.

## Releasing new releases

- Update your `manifest.json` with your new version number, such as `1.0.1`, and the minimum Obsidian version required for your latest release.
- Update your `versions.json` file with `"new-plugin-version": "minimum-obsidian-version"` so older versions of Obsidian can download an older version of your plugin that's compatible.
- Create new GitHub release using your new version number as the "Tag version". Use the exact version number, don't include a prefix `v`. See here for an example: https://github.com/obsidianmd/obsidian-sample-plugin/releases
- Upload the files `manifest.json`, `main.js`, `styles.css` as binary attachments. Note: The manifest.json file must be in two places, first the root path of your repository and also in the release.
- Publish the release.

> You can simplify the version bump process by running `npm version patch`, `npm version minor` or `npm version major` after updating `minAppVersion` manually in `manifest.json`.
> The command will bump version in `manifest.json` and `package.json`, and add the entry for the new version to `versions.json`

## Adding your plugin to the community plugin list

- Check https://github.com/obsidianmd/obsidian-releases/blob/master/plugin-review.md
- Publish an initial version.
- Make sure you have a `README.md` file in the root of your repo.
- Make a pull request at https://github.com/obsidianmd/obsidian-releases to add your plugin.

## How to use

- Clone this repo.
- `npm i` or `yarn` to install dependencies
- `npm run dev` to start compilation in watch mode.

## Manually installing the plugin

- Copy over `main.js`, `styles.css`, `manifest.json` to your vault `VaultFolder/.obsidian/plugins/your-plugin-id/`.

## Improve code quality with eslint (optional)
- [ESLint](https://eslint.org/) is a tool that analyzes your code to quickly find problems. You can run ESLint against your plugin to find common bugs and ways to improve your code. 
- To use eslint with this project, make sure to install eslint from terminal:
  - `npm install -g eslint`
- To use eslint to analyze this project use this command:
  - `eslint main.ts`
  - eslint will then create a report with suggestions for code improvement by file and line number.
- If your source code is in a folder, such as `src`, you can use eslint with this command to analyze all files in that folder:
  - `eslint .\src\`

## Funding URL

You can include funding URLs where people who use your plugin can financially support it.

The simple way is to set the `fundingUrl` field to your link in your `manifest.json` file:

```json
{
    "fundingUrl": "https://buymeacoffee.com"
}
```

If you have multiple URLs, you can also do:

```json
{
    "fundingUrl": {
        "Buy Me a Coffee": "https://buymeacoffee.com",
        "GitHub Sponsor": "https://github.com/sponsors",
        "Patreon": "https://www.patreon.com/"
    }
}
```

## API Documentation

See https://github.com/obsidianmd/obsidian-api
