# Bug Fix: Re-capture fails while a result window is open

- **Slug**: result-window-recapture
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Result windows now use run-specific labels, so a completed translation can remain open while another capture and translation create a separate result window. Result routing, focus, close cleanup, and always-on-top behavior now understand multiple result windows.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | Added centralized result label construction and parsing. |
| `src-tauri/src/commands.rs` | modified | Creates unique result windows, focuses the active run, and updates all result windows' always-on-top state. |
| `src-tauri/src/state.rs` | modified | Added run-aware analysis removal so stale windows cannot clear a newer run. |
| `src-tauri/src/lib.rs` | modified | Result close events only cancel the analysis matching that window's run ID. |
| `src/App.tsx` | modified | Routes run-specific result labels to the result view. |
| `src/windowLabels.ts` | added | Centralizes frontend result-label recognition. |
| `src/windowLabels.test.ts` | added test | Pins frontend result routing behavior. |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | Pins unique labels and strict label parsing. |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | Synchronized patch version to 0.4.1. |

## Diff Highlights

- Replaced the fixed Tauri label `result` with `result-<run-id>`.
- Removed destroy-then-immediately-rebuild behavior for the old fixed result label.
- Added `RuntimeState::take_analysis(run_id)` to prevent an older result window from taking a newer active analysis.

## Tests Added or Updated

- `src-tauri/src/state.rs::stale_result_window_cannot_take_current_analysis` — verifies closing an old result cannot clear the current run.
- `src-tauri/tests/desktop_lifecycle.rs::result_windows_use_unique_run_labels` — verifies unique construction and strict parsing.
- `src/windowLabels.test.ts` — verifies frontend routing accepts run-specific results and rejects empty/unrelated labels.

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml state::tests::stale_result_window_cannot_take_current_analysis` → passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle result_windows_use_unique_run_labels` → passed.
- Commands run: `npm test -- --run src/windowLabels.test.ts` → passed.
- Manual checks: code-path inspection confirmed that a terminal analysis is released before a new capture while its uniquely labeled result window remains independent.

## Deviations from Assessment

- Added `src/windowLabels.ts` and `src/windowLabels.test.ts` beyond the initial file list so frontend routing behavior is directly testable without importing Tauri window state into a unit test.

## Follow-ups

- Verify multiple completed result windows and a subsequent capture manually on both macOS and Windows when those environments are available.
