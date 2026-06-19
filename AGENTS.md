# Repository Guidelines

## Project Structure & Module Organization

This is a Tauri 1 desktop app with a Vite/React frontend and Rust backend. Frontend code lives in `src/`: window entry points are under `src/window`, shared UI in `src/components`, integrations in `src/services`, hooks in `src/hooks`, utilities in `src/utils`, and translations in `src/i18n/locales`. Rust commands, tray/window logic, OCR, updater, and config code live in `src-tauri/src`. Tauri config and platform overrides are in `src-tauri/tauri*.conf.json`. Static assets are in `public`, documentation media in `asset`, updater scripts in `updater`, and distribution patches in `patches`.

## Fork-Specific Context

This repository is the **See See** fork of Pot. Treat See See as an independent product: keep `productName`, package metadata, bundle identifier, cache paths, About links, release docs, and user-facing copy aligned with `See See`, `see-see`, and `com.seesee.desktop`. Do not re-enable the upstream Pot updater or point release automation at Pot-owned repositories.

See See's primary fork feature is the OCR-free screenshot vision path. It must remain a provider inside the existing screenshot translation window, displayed alongside the current OCR -> text translation providers. Do not add a separate vision hotkey, tray item, HTTP route, or standalone window unless explicitly requested. Vision providers receive the screenshot image directly and must not depend on target language, OCR language, or language detection settings; prompt, model, and endpoint configuration control the output.

Pot plugin compatibility is still intentional. References to Pot plugin templates, `.potext`, `pot-app.com` service docs, or legacy provider endpoints can remain when they describe upstream plugin/service ecosystems rather than See See branding.

## Build, Test, and Development Commands

- `pnpm install`: install Node dependencies from `pnpm-lock.yaml`.
- `pnpm tauri dev`: run the full desktop app in development mode.
- `pnpm run build`: build the frontend with Vite.
- `pnpm tauri build`: create platform installers through Tauri.
- `pnpm prettier --write .`: format frontend, config, and Markdown files.
- `cd src-tauri && cargo check`: validate Rust backend compilation without packaging.
- `cd src-tauri && cargo fmt --check`: verify Rust formatting after backend changes.

Linux builds require the system libraries listed in `README.md`.
On Windows debug builds, the upstream single-instance plugin is intentionally skipped so `pnpm tauri dev` does not hit its null-pointer startup panic; release builds keep single-instance behavior.

## Coding Style & Naming Conventions

Use the existing JavaScript/JSX style: 4-space indentation, semicolons, single quotes, trailing commas where valid, and 120-character print width. Let Prettier enforce formatting. React and config components use `PascalCase`; hooks use `useName`; service folders use lowercase provider names such as `src/services/translate/openai`. Rust uses standard `rustfmt`, snake_case modules, and existing error/logging patterns.

## Testing Guidelines

There is no dedicated automated test suite or `pnpm test` script in this tree. For frontend-only changes, run `pnpm run build` and manually exercise the affected window with `pnpm tauri dev`. For Rust or Tauri command changes, run `cd src-tauri && cargo check`; use `cargo fmt --check` when touching Rust. Add targeted tests only with the needed harness and document how to run them.

For screenshot translation work, verify three paths: normal OCR -> translation still renders, enabled vision providers render after translation provider cards, and changing OCR/source/target language settings does not alter the vision request payload.

## Commit & Pull Request Guidelines

Recent history uses short imperative subjects, Weblate-generated translation commits, and occasional conventional prefixes such as `fix: remove cmd.exe wrapper in run_binary to fix paths with spaces (#1280)`. Keep commits scoped to one concern and include an issue/PR reference when relevant. Pull requests should describe the change, list verification commands, link related issues, and include screenshots or GIFs for visible UI changes.

## Security & Configuration Tips

Do not commit API keys, tokens, signing keys, local service credentials, or updater secrets. Avoid logging request headers or provider credentials in translation, OCR, backup, or updater code. Keep platform-specific changes isolated to the matching `tauri.*.conf.json` or Rust `cfg` block.
