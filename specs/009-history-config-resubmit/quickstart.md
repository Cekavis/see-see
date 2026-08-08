# Quickstart: History Configuration Resubmit

## Automated validation

```powershell
npm test -- src/views/History.test.tsx src/ipc.test.ts
cargo test --manifest-path src-tauri/Cargo.toml history
npm run lint
npm run format:check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected outcomes:

- History detail defaults to the original configuration identities even when current names differ from historical snapshots.
- Changing either selector sends the selected model and prompt identities to resubmission.
- No global activation operation is invoked.
- History persistence returns the selected identities and existing history behavior remains green.

## Manual desktop validation

1. Configure two models and two prompts; make one pair globally active.
2. Capture and save a history entry with that pair.
3. Edit the original model ID or prompt body without replacing the configuration record.
4. Open the saved history detail and confirm the original pair remains selected under its current names.
5. Choose the other model and prompt and activate "重新选择配置提交".
6. Confirm a result window opens, the selected current values are used, and returning to settings shows the original global active pair unchanged.
7. Repeat at approximately 1094×768, 780×800, and 540×800; check labels, focus order, disabled/loading states, wrapping, and horizontal overflow.

## Release validation

```powershell
npm run tauri build
```

Install the generated Windows bundle locally and repeat the primary manual flow before committing.
