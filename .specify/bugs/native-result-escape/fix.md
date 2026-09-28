# Bug Fix: Restore native Escape closing for result windows

- **Slug**: native-result-escape
- **Fixed**: 2026-09-28
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Restored native window closing for ordinary Escape and existing close shortcuts. A result window with its model chooser open forwards the original Escape to that chooser, with acknowledged per-window state keeping native and frontend behavior aligned. Removed the synthetic DOM key injection.

## Changes

| File                                                         | Change                                            | Notes                                                                                                                                                             |
| ------------------------------------------------------------ | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/windowing.rs`                                 | Modified native key handling                      | Direct native close on Windows/macOS; forward only chooser Escape; consume held-key repeats.                                                                      |
| `src-tauri/src/state.rs`                                     | Added transient chooser state                     | Scoped to active run IDs; opening validates the run; clearing is idempotent; `take_analysis` clears it.                                                           |
| `src-tauri/src/commands.rs`, `lib.rs`                        | Added and registered chooser state command        | Injected caller window determines the run. Payload contains only `open`.                                                                                          |
| `src/ipc.ts`, `src/App.tsx`                                  | Connected native chooser state                    | Existing source streaming subscriptions and native capabilities remain unchanged.                                                                                 |
| `src/views/Result.tsx`                                       | Serialized chooser transitions                    | Native acknowledgement precedes showing/hiding; stale unmount work clears native state; errors remain recoverable. Opener stays focusable during acknowledgement. |
| `src/components/ModelRetryDialog.tsx`                        | Used existing controls and errors for transitions | Busy disables model rows/cancel without progress copy. Close failures appear inside the modal.                                                                    |
| Frontend tests and `src-tauri/tests/desktop_lifecycle.rs`    | Added focused regression checks                   | Command contract, state acknowledgement, recovery, unmount cleanup, duplicate actions, synthetic-dispatch prohibition.                                            |
| `src-tauri/examples/result_escape_smoke.rs`, `tests/native/` | Added Windows native smoke test                   | Uses actual App/Result/chooser code, production capabilities, in-memory data and OS keyboard input to isolated test windows. No provider or user credentials.     |
| `src-tauri/build.rs`                                         | Linked existing Windows resource into examples    | Gives the native test the same Common Controls manifest as the application. Production binary resource settings are unchanged.                                    |

## Tests Added or Updated

- Native decision tests cover ordinary Escape, chooser-first Escape, direct close shortcuts and held-key repeats.
- Runtime state test covers per-run isolation, invalid opening, idempotent clearing and cleanup with analysis removal.
- Frontend tests cover delayed acknowledgement, duplicate clicks, failure/retry, successful model selection, pending-open/pending-close unmount and stale notifications.
- Native smoke scenarios verify denied frontend close permission, direct native Escape close, chooser dismissal, held Escape, focus restoration, a subsequent separate Escape, and Ctrl+W while the chooser is open.
- The existing 500-model focus assertion still checks the same element; `waitFor` now waits for its asynchronous focus effect.

## Local Verification

- New synthetic-dispatch regression failed against the old implementation; passed after the native fix.
- Four new frontend regressions failed before the frontend change. Final frontend suite: 16 files, 122 tests passed.
- Focused backend: 26 lifecycle, 2 native decision and 8 state tests passed.
- Windows native smoke: all three scenarios passed, exit 0. Initial real-WebView focus failure led to retaining opener focus during acknowledgement; the same focus assertion then passed.
- Full Rust suite: 27 library tests and all integration tests passed; unchanged `streaming_client_allows_a_long_first_token_wait` failed with response-body decoding, then passed in isolation. Existing feature validation records this intermittent failure.
- ESLint, TypeScript/frontend build and `cargo fmt --check` passed. Global Prettier reports only unchanged `AGENTS.md`.

## Deviations from Assessment

- Added `src-tauri/build.rs` to scope after the native smoke executable failed before main with `STATUS_ENTRYPOINT_NOT_FOUND`. `dumpbin` confirmed its `TaskDialogIndirect` import; `mt.exe` confirmed it had no manifest. The installed Tauri resource compiler links its Common Controls manifest to binaries only. Linking that existing generated resource to Windows MSVC examples fixes the test executable without adding dependencies or changing application permissions.
- Native testing found disabling the opener during the acknowledgement removed its focus before the dialog could remember it. The opener now uses `aria-disabled` plus the existing synchronous transition guard, preserving focus restoration and duplicate prevention.
- Kept version 0.17.0 for this corrective commit under the repository's same-session version rule.

## Follow-ups

- Complete verification, package/install, and commit/push this corrective change.
- macOS native interaction cannot be verified on this Windows host; record that limitation explicitly.
