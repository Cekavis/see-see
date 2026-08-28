# Bug Assessment: Stale result-window actions report a missing analysis

- **Slug**: stale-result-window-actions
- **Created**: 2026-08-28
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

在存在多个结果窗口时，在旧的结果窗口点击“打开主窗口”等按钮会报“分析任务不存在”。旧结果窗口的操作应正常完成，不应因为运行时已经切换到新分析而显示错误。

## Symptom

After a second analysis replaces the first run in the single-slot runtime state, an older result window still uses its original `runId`. Clicking “打开主窗口” (and other run-scoped actions where present) calls a command that only searches the current analysis and returns `分析任务不存在`.

## Reproduction

1. Complete one screenshot translation and leave its result window open.
2. Start and complete a second screenshot translation so a second result window remains open.
3. In the older result window, click “打开主窗口” (or another available run action).
4. Observe the `分析任务不存在` notification instead of a successful navigation/cleanup action.

## Suspected Code Paths

- `src-tauri/src/commands.rs:431-459` — `close_result` and `open_main_window` call `active_analysis`, which only matches the current runtime analysis.
- `src-tauri/src/commands.rs:820-833` — `active_analysis` returns `NotFound` for completed runs removed from runtime state.
- `src/App.tsx:38-64` — every result window binds its controls to its URL `runId`, including stale windows.

## Root Cause Hypothesis

**High confidence:** result windows are intentionally run-specific, but runtime state retains only the latest analysis. Commands that only need to operate on the originating window (especially opening the main window and closing a terminal result) still require the run to be the active runtime entry. Once a newer run starts, those commands incorrectly surface the missing-analysis error.

## Proposed Remediation

**Preferred:** Make result-window navigation and cleanup tolerant of stale run IDs. `open_main_window` should always focus the main window, then close the requesting result window when its active analysis is terminal or no longer present; it should not return `分析任务不存在` for a stale result. Keep cancellation/retry guarded to the active run because stale windows cannot safely mutate a newer analysis. Add focused Rust regression coverage for stale navigation and preserve existing run-aware close semantics.

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`

**Tests to add or update**:

- Verify stale `open_main_window` handling does not propagate `ErrorCode::NotFound` after the runtime has moved to another run.
- Verify active-run navigation still closes only terminal result windows.

## Risks & Considerations

- A stale window must never cancel or clear a newer active analysis.
- The requesting result window may already be gone; cleanup should remain idempotent.
- The existing in-memory runtime cannot reconstruct a stale result after reload; this fix only covers actions from an already-loaded window.

## Open Questions

- None.
