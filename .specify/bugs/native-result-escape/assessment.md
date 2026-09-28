# Bug Assessment: Native result Escape stopped closing the window

- **Slug**: native-result-escape
- **Created**: 2026-09-28
- **Source**: user report and local Git history
- **Verdict**: valid
- **Severity**: high

## Report

Windows result windows no longer close with Escape after commit `11e4198` (2026-09-28 20:20:42 +08:00). The previous native handler called `target.close()`; the regression consumes the real key and injects a DOM keydown, relying on frontend close instead. The user requests fixing the regression.

## Symptom and Reproduction

Open a result window on Windows with no chooser open and press Escape. The result remains open. With the chooser open, Escape should dismiss the chooser first; the next separate Escape should close the result. Ctrl+W must continue to close the window directly.

## Suspected Code Paths

- `src-tauri/src/windowing.rs::dispatch_result_escape`: replaces native closing with script evaluation, after the native key has already been consumed.
- `src/App.tsx::App`: synthetic keydown eventually calls `getCurrentWebviewWindow().close()`.
- `src-tauri/capabilities/default.json` grants `core:default`, which does not include `core:window:allow-close`. The local generated ACL manifest confirms the close operation is outside the default set.
- `src/App.test.ts` mocks `close()` as successful, so its keyboard tests do not exercise native dispatch or permission enforcement.

## Root Cause Hypothesis

**Confidence: high.** The changed route removed the independent native close action and introduced an unauthorized frontend close call. Synthetic DOM testing did not catch the native/IPC regression. Native reproduction should verify the actual WebView2 key path, not merely re-dispatch a DOM event.

## Proposed Remediation

Restore direct native closing for ordinary Escape and existing close shortcuts. Track only the current result window's transient model chooser state in Rust through a scoped `set_result_model_chooser_open` IPC command (`{ open: boolean }`, caller window injected by Tauri). With a chooser open, leave the original Escape unhandled so the real dialog receives it; otherwise call native close directly. Remove synthetic key injection on both affected platforms. Clean up transient state with the corresponding analysis/window. Keep this state out of persistence and other windows.

The frontend must acknowledge the native state change before showing or dismissing the chooser, serialize conflicting transitions, report synchronization failures, and clear stale state on unmount. Preserve the source result, existing stream cleanup, and selection behavior. Do not add blanket window-close permission as a substitute for restoring the native route. Preserve 0.17.0 per the same-session corrective-commit rule.

## Files Likely to Change

- `src-tauri/src/windowing.rs`, `state.rs`, `commands.rs`, `lib.rs`
- `src-tauri/tests/desktop_lifecycle.rs` and state unit tests
- `src/App.tsx`, `src/App.test.ts`, `src/ipc.ts`, `src/ipc.test.ts`
- `src/views/Result.tsx`, `src/views/Result.test.tsx`; `src/components/ModelRetryDialog.tsx` only if lifecycle integration requires it
- `src-tauri/examples/result_escape_smoke.rs` and `tests/native/` fixture files for a real Windows WebView2 keyboard smoke test
- This bug's assessment/fix/test reports; feature validation notes if needed

## Tests and Verification

- Backend decision/state tests: ordinary Escape closes, open chooser receives the real Escape, per-window isolation, close/removal cleanup, Ctrl+W remains direct, held keys do not accidentally close after dismissing a chooser.
- Frontend tests: native acknowledgement precedes modal visibility, failure stays recoverable, repeated actions and unmount cannot leave stale modal state, cancellation and successful selection clear state.
- Native smoke harness using an isolated in-memory app, fictional models, actual result/chooser code, existing native shortcut handler, production capabilities, and OS keyboard events addressed to the test window. No real account, API credentials, capture, or network model call is needed.
- Run relevant test suites, lint, formatting, builds, Windows installation, and document unavailable macOS interaction honestly.

## Risks and Constraints

- Do not replace the failed path with another synthetic-key or timeout fallback chain.
- The original Escape must reach the dialog only when its own result chooser is open; other result windows must remain independently closable.
- Frontend/native state synchronization must not reintroduce StrictMode stale callbacks or leave a modal flag after the window closes.
- Native test inputs must target only the isolated test window. Leave user application data and credentials untouched.

## Open Questions

None needed for implementation. Record native reproduction evidence and platform limitations in the verification report.
