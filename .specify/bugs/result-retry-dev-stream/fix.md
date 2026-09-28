# Bug Fix: Model retry window hangs and development streaming duplicates

- **Slug**: result-retry-dev-stream
- **Fixed**: 2026-09-28
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Changed model retry to an asynchronous Tauri command so native Windows window creation does not run inside synchronous IPC handling. Result subscription cleanup now ignores obsolete channel events and attach responses, preventing StrictMode from appending each delta twice. Simplified the chooser to compact saved-name rows with scrolling, without search, counts, or loading/creation progress copy.

## Changes

| File                                       | Change                            | Notes                                                                                                                         |
| ------------------------------------------ | --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/commands.rs`                | modified                          | Declare `retry_analysis_with_model` async; preserve request and response contract                                             |
| `src-tauri/tests/desktop_lifecycle.rs`     | extended test                     | Require a Future for model retry alongside capture/history window creation                                                    |
| `src/App.tsx`                              | modified                          | Effect-local active guard for events, attach success/mismatch, and rejection; invalidate in cleanup                           |
| `src/App.test.ts`                          | added tests                       | StrictMode broadcasts, legitimate repeated tokens, stale responses; update chooser Escape interaction                         |
| `src/components/ModelRetryDialog.tsx`      | modified                          | Remove search/count/progress UI; retain errors, pending guard, name-only list, focus restoration; add Home/End navigation     |
| `src/components/ModelRetryDialog.test.tsx` | updated tests                     | 500 configurations, ID-based duplicate-name selection, quiet pending/loading, keyboard controls, recovery and lifecycle races |
| `src/styles.css`                           | modified                          | Remove the unused chooser search-field selector                                                                               |
| `specs/019-result-model-retry/`            | updated documents and screenshots | Align the ongoing feature with the final requested interaction                                                                |

## Tests Added or Updated

- `result_window_creation_stays_out_of_synchronous_windows_commands`: failed to compile before the fix with E0277 because the retry command returned Result rather than Future; passed after async dispatch.
- `applies each streamed delta once under StrictMode while preserving repeated tokens`: broadcasts to both registered channels. Before the fix, thinking became `思思思思考考`; afterward it remains `思思考` and text remains `aab`.
- Three stale attach cases (snapshot, mismatched snapshot, failure) failed before cleanup and passed afterward without stale output or notifications.
- Chooser tests retain immediate submission, duplicate prevention, error recovery, fresh loading, stale operation isolation, Escape, Tab containment, and arrow navigation. The 500-row test selects the last of two identical names using its configuration ID.

## Local Verification

- Frontend full suite: 16 files, 112 tests passed.
- Rust full suite: all enabled unit, integration, and doc tests passed; the two existing native-credential tests remain ignored by their own annotations.
- Targeted backend lifecycle/analysis tests: 34 passed. Targeted App tests: 18 passed. Chooser tests: 11 passed.
- ESLint, TypeScript/Vite build, Rust formatting, and `git diff --check` passed.
- Browser review at 460×500 and 420×360: 500 compact 32px rows; internal scrolling, ellipsis, focus/keyboard navigation, errors, empty state, and independent mocked child result all checked.
- `npm run test:e2e`: primary smoke passed.
- Signed `npm run tauri build`: succeeded; Windows NSIS/MSI installers and signatures produced.
- User explicitly approved closing the running application and installing. NSIS installer exited 0; installed 0.17.0 restarted and reported Responding=true. Full binary comparison matched the release build with the documented `__TAURI_BUNDLE_TYPE_VAR_NSS` installer marker replacing `UNK`.
- Global format check still flags only unchanged `AGENTS.md`; changed files and documents were formatted. The unrelated release-config failure was already recorded for this objective.

## Deviations from Assessment

None in product scope. The initial raw executable hash comparison differed by three bytes; inspection showed exactly Tauri's documented NSIS bundle-type marker. A complete byte comparison against the expected NSIS-patched build then passed; no arbitrary mismatch was ignored and no product file was changed for this check.

## Follow-ups

- Native Windows result retry with a real provider and macOS runtime interaction were not exercised in this environment. Automated async-dispatch constraints and mocked browser interactions are not a substitute for those checks.
