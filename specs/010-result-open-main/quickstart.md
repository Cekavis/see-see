# Quickstart: Result Window Main Navigation

## Automated validation

```powershell
npm test -- src/views/Result.test.tsx src/ipc.test.ts
cargo test --manifest-path src-tauri/Cargo.toml terminal_analysis_states
npm run lint
npm run format:check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected outcomes:

- The result footer exposes `打开主窗口` and invokes its callback.
- The IPC wrapper sends the current run identity.
- Submitting and streaming remain active states; completed, failed, and cancelled remain terminal states.
- Existing result controls and window lifecycle tests stay green.

## Manual desktop validation

1. Start an analysis and activate `打开主窗口` while output is still streaming.
2. Confirm the main window is visible and focused while the result window remains open and continues updating.
3. Let the analysis complete, then activate `打开主窗口` again.
4. Confirm the main window is focused and the finished result window closes.
5. Repeat with a failed or cancelled analysis.
6. Minimize or hide the main window before activation and confirm it is restored.
7. Repeat at the normal result-window size and 420×360 minimum; check keyboard focus, footer wrapping, spacing, and horizontal overflow.

## Release validation

```powershell
npm run tauri build
```

Install the generated Windows bundle locally and repeat the primary flow before committing.
