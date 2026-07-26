# Bug Assessment: Re-capture fails while a result window is open

- **Slug**: result-window-recapture
- **Created**: 2026-07-26
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 如果当前有打开的结果窗口，再次截图翻译时不应该报错，而是新开一个截图窗口。如果保留多个窗口技术上很难实现，应该自动关闭之前的窗口并继续，而不是报错。

## Symptom

After a completed translation leaves its result window open, starting another screenshot can fail when the next analysis tries to create its result window. The new screenshot and translation should continue, preferably while keeping completed result windows available; otherwise the prior result window should be closed without surfacing an error.

## Reproduction

1. Complete a screenshot translation and leave its result window open.
2. Start another screenshot translation.
3. Complete the selection so the application creates the next result window.
4. Observe that result-window creation can fail instead of opening the next result.

## Suspected Code Paths

- `src-tauri/src/commands.rs:781` — `create_result_window` destroys the fixed `result` label and immediately rebuilds that same label, so asynchronous teardown can leave the label occupied.
- `src-tauri/src/commands.rs:369` — always-on-top updates address only the single fixed `result` label.
- `src-tauri/src/commands.rs:816` — active-result focusing assumes the fixed `result` label.
- `src-tauri/src/lib.rs:121` — closing any fixed result window takes the sole active analysis; this must become run-aware if completed result windows coexist.
- `src/App.tsx:114` — frontend routing recognizes only the exact `result` label.

## Root Cause Hypothesis

Confidence: high. Every analysis uses the same Tauri window label, `result`. The existing window is destroyed immediately before a new window with that label is built, but destruction and platform webview teardown are not guaranteed to complete synchronously. Reusing the occupied label makes `WebviewWindowBuilder::build` fail and is collapsed into the generic “无法创建结果窗口” error. The single-label assumptions in routing, focus, close handling, and window settings also prevent safe coexistence.

## Proposed Remediation

**Preferred**: Give each result window a deterministic label derived from its analysis run ID (`result-<run-id>`), allowing completed result windows to remain open while a new capture and analysis proceed. Centralize label construction/parsing, route all result-prefixed windows to `ResultView`, focus the window belonging to the active run, apply always-on-top changes to every result window, and only cancel/clear runtime analysis when the closing result window belongs to that active run.

**Alternative**:

- Destroy the prior result and wait/retry until the fixed label is released. This preserves a single result window but discards useful completed output and depends on platform-specific teardown timing.

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/state.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src/App.tsx`
- synchronized application version files

**Tests to add or update**:

- Verify result labels are unique per run and can be parsed without matching unrelated windows.
- Verify closing a stale completed result cannot clear the current active analysis.
- Verify result window routing recognizes unique result labels.

## Risks & Considerations

- Closing an older result window must not cancel a newer active analysis.
- Result window preferences must remain consistent across all open result windows.
- Runtime state retains only the current analysis; an already-loaded completed window can keep its React snapshot, but reloading that old window is not supported by the current in-memory design.
- This behavior fix requires a synchronized patch version bump.

## Open Questions

- None.
