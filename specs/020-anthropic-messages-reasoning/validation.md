# Validation: Anthropic Messages Reasoning Effort

**Date**: 2026-10-13
**Version**: 0.19.0

| Check | Result | Evidence |
|---|---|---|
| Frontend focused tests | PASS | `npm test -- src/views/Settings.model.test.tsx` — 12 tests passed |
| Rust provider contract tests | PASS | `cargo test --manifest-path src-tauri/Cargo.toml --test provider_contracts` — 15 tests passed |
| Rust model configuration tests | PASS | `cargo test --manifest-path src-tauri/Cargo.toml --test model_config` — 6 tests passed |
| Frontend full tests | PASS | `npm test` — 17 files, 130 tests passed |
| Rust full tests | PASS | `cargo test --manifest-path src-tauri/Cargo.toml` — all executed tests passed; 2 credential-store tests ignored by design |
| Type check and frontend build | PASS | `npm run build` |
| ESLint | PASS | `npm run lint` |
| Rust formatting | PASS | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` |
| Changed-file formatting | PASS | targeted Prettier check for changed TypeScript and feature documents |
| Full repository formatting | UNRELATED FAILURE | `npm run format:check` reports the pre-existing `AGENTS.md` only; changed files pass the targeted check |
| Release bundle | PASS | signed MSI and NSIS installers plus updater signatures generated for 0.19.0 |
| Local installation | PASS | NSIS installer exit code 0; installed executable reports product version 0.19.0 |

The native settings window was not available for an interactive screenshot review in this run because the installed tray application exposed no visible window to the available desktop surface. Automated settings tests cover the selector, default, protocol switching, and saved payload behavior.
