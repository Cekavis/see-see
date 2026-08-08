# Contract: Result Window Main Navigation

## Result footer

- Render a button with visible and accessible name `打开主窗口` in every analysis state.
- Activation invokes the navigation callback once.
- A rejected navigation callback uses the existing error notification and leaves the result UI available.
- Existing cancel, retry, copy, and always-on-top controls retain their current state rules.

## IPC command

### Request

- Command: `open_main_window`
- Input: `runId`, the analysis identity owned by the calling result window.

### Success behavior

1. Restore, show, and focus the existing `main` window.
2. Read the matching analysis snapshot from backend runtime state.
3. If the state is submitting or streaming, return with the result window unchanged.
4. If the state is completed, failed, or cancelled, close the matching result window through its normal lifecycle.

### Failure behavior

- A missing main window, inaccessible runtime state, stale run identity, or window operation failure returns the existing structured application error form.
- The command does not cancel active work.
