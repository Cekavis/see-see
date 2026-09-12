# Quickstart: Remember Result Window Position

## Prerequisites

- A Windows or macOS desktop environment with the See See application configured to run an analysis.
- Rust toolchain and the repository dependencies installed.

## Automated validation

From the repository root, run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle result_window
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected outcome: the focused result-window lifecycle tests and the complete Rust test suite pass.

## Manual desktop validation

1. Launch See See and create one result window. Confirm that the first result window still opens centered, with the existing compact dimensions.
2. Drag that result window to a clearly identifiable location on the current display.
3. Start another analysis that creates a new result window. Confirm that its top-left corner opens at the location chosen in step 2.
4. Move either result window to a different location and create a third result window. Confirm that the third window uses the most recently moved location and that already open windows do not move.
5. Close the window that established the remembered location, create another result window, and confirm the location remains remembered for the rest of the session.
6. Move the main window or a capture overlay, then create a result window. Confirm those non-result windows do not change the remembered result-window location.

## Expected behavior

- No new setting, prompt, or frontend control appears.
- The first result window uses the existing centered fallback when no result window has moved.
- Later result windows reuse the latest result-window location while preserving size, focus, always-on-top, streaming, retry, close, and navigation behavior.

## Visual review evidence

Record a screenshot or manual review note for the existing 460×500 default result window and 420×360 minimum result window in both centered-first-open and remembered-position states on each available supported desktop platform.

## Validation record — September 12, 2026

- Passed `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`.
- Passed `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings`.
- Passed `cargo test --manifest-path src-tauri/Cargo.toml`: all Rust unit, integration, benchmark, and doc-test groups passed.
- Passed `cargo test --manifest-path src-tauri/Cargo.toml --lib --test desktop_lifecycle`: 18 unit tests and 20 desktop lifecycle tests passed, including the new position regressions.
- Passed `npm test` (67 tests), `npm run lint`, and `npm run build`.
- Passed changed-file Prettier validation. The repository-wide `npm run format:check` still reports the pre-existing, unchanged `AGENTS.md` file.
- Passed signed `npm run tauri build`; MSI, NSIS, and updater signature artifacts were generated for version `0.12.2`.
- Passed local NSIS installation; the installed executable reports version `0.12.2`.
- Interactive drag-and-create verification and macOS visual review were not run in this Windows automation session.
