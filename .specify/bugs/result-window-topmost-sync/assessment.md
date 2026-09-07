# Bug Assessment: 结果窗口置顶按钮未同步

- **Slug**: result-window-topmost-sync
- **Created**: 2026-09-07
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 当一个结果窗口改变了置顶状态，其他结果窗口的置顶按钮也要同时改变

## Symptom

When one result window changes the always-on-top setting, the backend updates the native state of all result windows, but the checkboxes in the other result windows keep their previous values. All open result-window controls should reflect the new shared setting immediately.

## Reproduction

1. Keep two result windows open.
2. Toggle “窗口置顶” in one result window.
3. Observe the native window state and the “窗口置顶” checkbox in the other result window.

## Suspected Code Paths

- `src-tauri/src/commands.rs:466-481` — `set_result_always_on_top` applies the native state to every result window and persists the shared preference, but does not notify result-window frontends.
- `src/App.tsx:50-73` — each result window stores `alwaysOnTop` in local React state and only initializes it from `getAppSnapshot()`.
- `src/App.tsx:75-86` — the initiating window optimistically updates only its own local state before invoking the shared command.
- `src/views/Result.tsx:104-119` — the checkbox is controlled by the local `alwaysOnTop` prop, so other windows cannot reflect backend changes without a new snapshot.

## Root Cause Hypothesis

**Confidence: high.** The preference is shared in the Rust backend and native window updates already iterate over all result windows, but there is no cross-webview event or refresh path for the controlled React checkbox. Each window therefore remains visually stale until it is recreated or otherwise reloaded.

## Proposed Remediation

**Preferred**: After the backend successfully applies and persists the shared setting, emit a typed application event such as `result-always-on-top-changed` with the boolean value. Subscribe to that event in every `ResultView` and update local state from the payload, while retaining the existing initial `getAppSnapshot()` load and optimistic update for the initiating window.

Add focused frontend coverage for applying the shared event payload and backend/source coverage that the command emits the event after persisting the setting. Keep the existing native loop and database schema unchanged.

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src/App.tsx`
- `src/App.test.ts`
- synchronized application version files

**Tests to add or update**:

- Verify the frontend applies a boolean always-on-top change received from the shared event.
- Verify `set_result_always_on_top` emits the shared event after the database transaction.
- Run the focused frontend and Rust tests plus the project lint, format, build, and release validation required by the repository.

## Risks & Considerations

- Event subscription must be removed when a result window unmounts to avoid stale handlers.
- The event must be emitted only after the native updates and preference persistence succeed, so a failed command does not advertise an uncommitted state.
- This behavior fix requires a synchronized patch version bump.

## Open Questions

- None.
