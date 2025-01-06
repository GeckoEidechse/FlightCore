<p align="center">
	<img src="docs/assets/Square310x310Logo.png" width="200px">
	<br>
	<br>
	<a href="https://r2northstartools.github.io/FlightCore/index.html?win-setup"><img src="docs/assets/downloadbutton.png" width="300px"></a>
	<br>
</p>

# FlightCore

A [Northstar](https://northstar.tf/) installer, updater, and mod-manager

![FlightCore screenshot](docs/assets/main-window-screenshot.png)

## Install

Downloads are available on the [releases page](https://github.com/R2NorthstarTools/FlightCore/releases).

**Windows:** Download `FlightCore_X.Y.Z_x64_en-US.msi` and then run the installer by double-clicking the file.

**Linux:** Download `flight-core_X.Y.Z_amd64.AppImage`, put it in a preferred location and make it executable. A Flatpak version is planned for the future.

All versions of FlightCore feature an auto-updater that will ask to self-update on new releases.

<a href="https://github.com/R2NorthstarTools/FlightCore/releases"><img src="https://img.shields.io/github/v/release/R2NorthstarTools/FlightCore" alt="GitHub release (latest by date)"></a>
<img src="https://img.shields.io/github/downloads/R2NorthstarTools/FlightCore/latest/total" alt="GitHub release downloads (latest by date)">
<a href="https://translate.harmony.tf/engage/northstar/">
<img src="https://translate.harmony.tf/widgets/northstar/-/flightcore/svg-badge.svg" alt="Translation status" />
</a>

## Frequently Asked Questions (FAQ)

Answers to frequently asked questions can be found in [docs/FAQ.md](docs/FAQ.md)

## Development

If you'd like to contribute to FlightCore, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)

If you are a Northstar developer/contributor, you might want to look at [docs/DEV-TOOLS.md](docs/DEV-TOOLS.md)

### Translating

Translations can be submitted via [weblate](https://translate.harmony.tf/projects/northstar/flightcore/). \
If you want to add translations for a new language that does not exist in FlightCore yet, please [reach out via GitHub issues](https://github.com/R2NorthstarTools/FlightCore/issues/new) so that we can add support for it.

<a href="https://translate.harmony.tf/engage/northstar/">
<img src="https://translate.harmony.tf/widgets/northstar/-/flightcore/multi-auto.svg" alt="Translation status" />
</a>

## Roadmap

--> See https://github.com/R2NorthstarTools/FlightCore/issues/1

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Type Support For `.vue` Imports in TS

Since TypeScript cannot handle type information for `.vue` imports, they are shimmed to be a generic Vue component type by default. In most cases this is fine if you don't really care about component prop types outside of templates. However, if you wish to get actual prop types in `.vue` imports (for example to get props validation when using manual `h(...)` calls), you can enable Volar's Take Over mode by following these steps:

1. Run `Extensions: Show Built-in Extensions` from VS Code's command palette, look for `TypeScript and JavaScript Language Features`, then right click and select `Disable (Workspace)`. By default, Take Over mode will enable itself if the default TypeScript extension is disabled.
2. Reload the VS Code window by running `Developer: Reload Window` from the command palette.

You can learn more about Take Over mode [here](https://github.com/johnsoncodehk/volar/discussions/471).
