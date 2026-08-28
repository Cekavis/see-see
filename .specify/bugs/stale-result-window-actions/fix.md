# Bug Fix: Stale result-window actions report a missing analysis

- **Slug**: stale-result-window-actions
- **Fixed**: 2026-08-28
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Result-window navigation and cleanup now tolerate a run that has already been replaced by a newer analysis. Opening the main window from an older result focuses the main window and closes only that stale result window; stale cleanup is idempotent and never touches the current analysis.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | Treat missing run state as a stale terminal result for `open_main_window`; make `close_result` a no-op for stale runs. |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | Pins stale navigation and cleanup handling. |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | Synchronized patch version to 0.11.4. |

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::stale_result_navigation_does_not_report_missing_analysis` — verifies stale `runId` branches are handled without propagating `NotFound` and that result-window close is idempotent.

## Local Verification

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed (18 unit tests plus all integration suites).
- `C:\nvm4w\nodejs\npm.cmd test -- --run src/windowLabels.test.ts src/views/Result.test.tsx` → passed (11 tests).
- `C:\nvm4w\nodejs\npm.cmd run build` → passed (TypeScript and Vite build).
- `C:\nvm4w\nodejs\npm.cmd run lint` → passed.
- `C:\nvm4w\nodejs\npm.cmd run format:check` → passed.
- Manual multi-window reproduction → not run; it requires invoking the configured external model.

## Deviations from Assessment

None.

## Follow-ups

- Perform one manual double-capture check on Windows and macOS when a test model configuration is available.
