# Verification: history and performance metric presentation

- **Slug**: `history-performance-display`
- **Tested**: 2026-10-01
- **Assessment**: [assessment.md](./assessment.md)
- **Fix**: [fix.md](./fix.md)
- **Result**: verified

## Automated checks

- `npm test` — passed: 17 files, 129 tests.
- `npm run lint` — passed.
- `npm run build` — passed: TypeScript check and Vite production bundle.
- `npx prettier --check` on all changed frontend files — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed: all Rust tests; 2 credential-store tests remained ignored by their existing precondition.
- `npm run format:check` — reports only the pre-existing repository `AGENTS.md` formatting warning.

## Manual UI checks

A local fixture with fictional data was inspected at 1440-class desktop layout, 1024 × 720, and 420 × 360:

- Long and short performance values share the same label, bar, and value columns.
- Result view shows `输入/输出`, `TTFT`, and an unprefixed `tps` value.
- History list keeps all four statistics on one compact line; legacy placeholders remain `—`.
- Compact result view remains readable without horizontal clipping.

## Packaging

`npm run tauri build` compiled the release executable and produced signed Windows MSI and NSIS bundles, including updater signatures, using the existing local signing credential without printing it. Local installation was not attempted because an existing `see-see.exe` process was running and was not interrupted.
