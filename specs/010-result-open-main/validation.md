# Validation: Result Window Main Navigation

**Date**: 2026-08-08

**Baseline**: `origin/master` at `95c13d7` (`0.8.0` history resubmission configuration)

**Feature version**: `0.9.0`

## Automated checks

- Focused frontend red/green check: result/IPC tests failed before the new callback and command existed, then passed after implementation.
- `npm test`: 14 files, 55 tests passed.
- `npm run lint`: passed.
- `npm run format:check`: passed.
- `npm run build`: passed.
- `npm run test:e2e`: passed after installing the locked Playwright Chromium (`✓ See See primary desktop flow`).
- `npm run test:release-config`: passed.
- `npm run test:macos-signing`: 3 tests passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: all unit, integration, benchmark, and doc tests passed.
- Focused Rust checks verify that submitting/streaming remain active, completed/failed/cancelled are terminal, and the result navigation command checks terminal state before closing.

## Release build and installation

- Signed `npm run tauri build`: passed using the existing updater key and DPAPI-protected UTF-16LE password.
- Generated Windows x64 MSI (9,121,792 bytes) and NSIS (6,324,260 bytes) installers.
- Generated 416-byte updater signatures for both installers.
- Installed the NSIS bundle silently with exit code 0.
- Verified installed executable product and file versions are both `0.9.0` at `C:\Users\cekav\AppData\Local\See See\see-see.exe`.

## Result-window visual review

The completed-result state was rendered from the production React source with mocked local Tauri IPC, then inspected at the configured normal and minimum result-window sizes.

| Viewport | Result | Evidence |
|----------|--------|----------|
| 460×500 | Pass: "打开主窗口" and "复制全文" remain visible in the persistent footer, the new action has a clear focus outline, and the page has no horizontal overflow. | [result-460x500.png](./result-460x500.png) |
| 420×360 | Pass: both actions remain fully readable on one row, the footer remains within the viewport, and the page has no horizontal overflow. | [result-420x360.png](./result-420x360.png) |

The installed computer-control plugin could not enumerate native Windows windows and its documented runtime entry point was unavailable in the installed version. Native active-versus-terminal window closure was therefore not claimed as directly observed; it is covered by the focused frontend IPC tests and Rust lifecycle/state regressions. The installed 0.9.0 binary and signed installers were verified separately as described above.
