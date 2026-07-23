# Bug Fix: Windows launch opens an extra console

- **Slug**: windows-console-logs
- **Fixed**: 2026-07-23
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Release builds now use the Windows GUI subsystem, and the Tauri log plugin writes only to the application log directory instead of duplicating output to stdout.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/main.rs` | modified | Added the release-only Windows GUI subsystem attribute. |
| `src-tauri/src/lib.rs` | modified | Replaced default log targets with `LogDir` only. |
| `package.json`, `package-lock.json` | modified | Bumped application version to 0.1.1. |
| `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` | modified | Bumped Rust package version to 0.1.1. |
| `src-tauri/tauri.conf.json` | modified | Bumped bundle version to 0.1.1. |

## Tests Added or Updated

No source-level test was added because the fix is linker and plugin configuration. The release bundle and resulting PE header provide the direct regression check.

## Local Verification

- `npm run format:check` → passed.
- `npm run lint` → passed.
- `npm test` → 8 files and 15 tests passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed.
- `cargo test --manifest-path src-tauri/Cargo.toml -j 1` → passed.
- `npm run test:e2e` → 1 WebdriverIO smoke test passed.
- `npm run tauri build` → produced 0.1.1 MSI and NSIS bundles.
- PE subsystem inspection → `2` (`IMAGE_SUBSYSTEM_WINDOWS_GUI`).
- NSIS silent installation → exit code 0.
- Installed application launch → main window opened as `See See`; `%LOCALAPPDATA%\app.seesee.desktop\logs\See See.log` was updated.

## Deviations from Assessment

None.

## Follow-ups

- The missing favicon diagnostic remains available in the log file and can be addressed separately if desired.
