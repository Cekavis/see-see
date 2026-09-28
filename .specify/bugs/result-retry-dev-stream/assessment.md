# Bug Assessment: Model retry window hangs and development streaming duplicates

- **Slug**: result-retry-dev-stream
- **Created**: 2026-09-28
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: high

## Report

The model retry chooser gets stuck creating a result window. Under `npm run tauri dev`, streaming characters appear twice (for example `aabb`). The user also requests removing the chooser search box, model count, and creation progress copy while preserving compact rows displaying only saved configuration names.

## Symptom

Selecting a saved model does not finish opening the independent result. During development, a single stream updates the result twice per delta.

## Reproduction

1. On Windows, open an analysis result, click the footer model retry button, and select a saved configuration. The synchronous command reaches native WebView creation and hangs.
2. Run the app with `npm run tauri dev`, open a result, and observe streaming text before completion. The entry point uses React StrictMode, which replays effect setup and cleanup; both registered channels append each delta.
3. Automated equivalents: require the window-creating command to return a Future; mount the real App under StrictMode and deliver each text/thinking delta once to every registered channel.

## Suspected Code Paths

- `src-tauri/src/commands.rs::retry_analysis_with_model` is synchronous and calls `start_analysis`, then `create_result_window` and `WebviewWindowBuilder::build`.
- Locked Tauri 2.11.5 source `src/webview/webview_window.rs:58` documents the Windows deadlock for synchronous commands. The repository already fixed this same constraint for `finish_capture`; its regression test currently covers only capture and history resubmission.
- `src/main.tsx` wraps the App in StrictMode.
- `src/App.tsx::ResultView` attaches a Channel inside an effect without cleanup or a guard against stale callbacks or attach responses.
- `src-tauri/src/analysis.rs::ActiveAnalysis::subscribe` retains each channel; each emitted delta reaches both development subscriptions.
- `src/components/ModelRetryDialog.tsx` adds search/count/progress UI that the user has explicitly rejected.

## Root Cause Hypothesis

**Confidence: high.** The new retry command repeats the previously documented Windows synchronous window-creation deadlock. Runtime map locks are already scoped before window creation. The development duplication follows directly from effect replay combined with two live unguarded Channel callbacks. The fixes should address command scheduling and subscription lifecycle, not text de-duplication or disabling StrictMode.

## Proposed Remediation

Make `retry_analysis_with_model` asynchronous, matching the existing window-creating commands, and extend the compile-time regression constraint. Guard the result subscription's event callback, attach resolution, and rejection with an effect-local active flag that cleanup invalidates. Preserve legitimate repeated text and thinking deltas.

Remove the chooser search/filter logic, model count, and loading/creation progress copy. Retain a bounded scrollable list, saved names only, immediate selection, duplicate-submit protection, explicit errors/empty state, keyboard navigation, dismissal, and focus restoration. Update the current feature artifacts to reflect the revised UX. Keep the existing 0.17.0 version for this unfinished objective.

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src/App.tsx`
- `src/App.test.ts`
- `src/components/ModelRetryDialog.tsx`
- `src/components/ModelRetryDialog.test.tsx`
- `src/styles.css`
- `specs/019-result-model-retry/` documents and chooser screenshots

**Tests to add or update**:

- Async return-type check for the new retry command, retaining all existing checks.
- StrictMode streaming regression that broadcasts genuine repeated text and thinking deltas to every attached channel; stale attach success/failure after cleanup must not overwrite state or notify.
- Chooser tests for 500 name-only rows, no search/count/progress text, keyboard focus and scrolling, immediate ID-based selection, pending duplicate prevention, recoverable errors, and stale operations across reopening.
- Relevant frontend/Rust suites, lint, formatting, frontend/native builds, and browser checks at representative result sizes.

## Risks & Considerations

- Mocked frontend IPC cannot prove native Windows window creation. Record native verification separately from automated scheduling checks.
- Cleanup must be effect-local; a shared ref set active by a later effect would reactivate the old subscription.
- Do not remove StrictMode, strip duplicate characters, change snapshot merge behavior, or introduce new dependencies.
- Existing installation/commit work from the feature remains pending. Inspect running applications before installation; do not force-close user work.

## Open Questions

- None required for implementation. Native Windows and macOS verification availability must be stated in the report.
