# Validation: History Configuration Resubmit

**Date**: 2026-08-08

**Baseline**: `origin/master` at `e5beafa` (`0.7.1` macOS release fixes)

**Feature version**: `0.8.0`

## Automated checks

- `npm test`: 14 files, 52 tests passed.
- `npm run lint`: passed.
- `npm run format:check`: passed.
- `npm run build`: passed.
- `npm run test:release-config`: passed; updater release configuration is complete.
- `npm run test:macos-signing`: 3 tests passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: all unit, integration, benchmark, and doc tests passed.
- Focused frontend red/green check: 7 history/IPC tests passed after first failing against the previous behavior.
- Focused migration check: `legacy_history_rows_backfill_configuration_ids` passed.

## Release build and installation

- Signed `npm run tauri build`: passed using the existing updater key path and DPAPI-protected password.
- Generated Windows x64 MSI and NSIS installers plus 416-byte updater signatures for both bundles.
- Installed the NSIS bundle silently over the existing application.
- Verified installed executable product and file versions are both `0.8.0` at `C:\Users\cekav\AppData\Local\See See\see-see.exe`.
- Confirmed the upstream `0.7.1` release workflow and verification script have no feature diff from `origin/master`.

## Installed-app visual review

The installed application was opened with existing history data. The original history model and prompt were selected by default using their current display values.

| Viewport | Result | Evidence |
|----------|--------|----------|
| 1094×768 | Pass: two full-width selectors align with existing content; both actions remain on one row; no horizontal page overflow. | [detail-1094x768-actions.png](./detail-1094x768-actions.png) |
| 780×800 | Pass: selectors remain readable; action labels remain complete; responsive navigation behavior is unchanged. | [detail-780x800-actions.png](./detail-780x800-actions.png) |
| 540×800 | Pass: selectors stack at full width; actions wrap vertically without clipping; "重新选择配置提交" remains fully readable. | [detail-540x800-actions.png](./detail-540x800-actions.png) |

Keyboard/accessibility behavior is covered by native labeled `select` elements and the existing shared button component. The computer-use plugin's documented `sky.documentation` entry point and window enumerator were unavailable in the installed plugin version, so the fallback used Windows child-window messages and `PrintWindow` capture against the actual installed Tauri WebView; no product data or global configuration was modified during visual review.
