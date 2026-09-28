# Quickstart and Validation

## Automated gates

Run npm commands via `powershell.exe -Command` as required by AGENTS.md.

1. `npm test -- src/components/ModelRetryDialog.test.tsx src/views/Result.test.tsx src/ipc.test.ts`
2. `cargo test --manifest-path src-tauri/Cargo.toml --test analysis_flow`
3. `npm run lint`, `npm run format:check`, `npm test`, `npm run build`, full `cargo test`, and `cargo fmt --check`.
4. `npm run test:e2e` and `npm run test:release-config`.
5. `npm run tauri build`, then install locally and verify installed version 0.17.0.

## Visual and manual scenarios

- At 460×500 and 420×360, open the chooser from the bottom action. Dismissal stays visible; 500 name-only rows scroll inside the list without horizontal overflow. No search, model count, or progress copy appears.
- Navigate with arrows and Home/End, activate a result with the keyboard, and dismiss with Escape. Focus returns to the footer action.
- Verify loading stays quiet, no configurations shows the empty state, and failures allow recovery.
- From a completed, failed, or streaming source, select another model. A new result opens immediately with the same image and prompt; old result and global active model remain unchanged.
- Close either result and confirm the other still works.
- Under development StrictMode, deliver streaming text/thinking deltas to every attached channel. Each delta should append once, and legitimate repeated tokens should remain.

Record executed checks, screenshots, installed version, and any unavailable platform/provider validation in `validation.md`.
