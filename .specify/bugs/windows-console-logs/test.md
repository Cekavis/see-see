# Bug Verification: Windows launch opens an extra console

- **Slug**: windows-console-logs
- **Tested**: 2026-07-23
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The installed 0.1.1 release opens its `See See` application window from a Windows GUI-subsystem executable, and startup diagnostics update the application log file. Automated regression suites and release packaging pass.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Launch installed `see-see.exe` | pass | Main window opened as `See See`; executable subsystem is GUI (`2`). |
| Log destination | Compare `See See.log` timestamp before and after launch | pass | `%LOCALAPPDATA%\app.seesee.desktop\logs\See See.log` was updated. |
| Configuration | Inspect entry point and log targets | pass | Release GUI attribute exists; `LogDir` is the only configured target. |
| Frontend regression | `npm test` and `npm run test:e2e` | pass | 15 Vitest tests and 1 WebdriverIO test passed. |
| Rust regression | `cargo test` and targeted `desktop_lifecycle` test | pass | All suites passed; targeted suite passed 3 tests. |
| Quality checks | Prettier, ESLint, rustfmt, Clippy | pass | No formatting, lint, or warning failures. |
| Release packaging | `npm run tauri build` | pass | MSI and NSIS 0.1.1 bundles produced. |
| Local installation | NSIS `/S` | pass | Installer exited with code 0. |

## Output Excerpts

- `PE_SUBSYSTEM=2`
- `MAIN_WINDOW=See See`
- `LOG_UPDATED=True`
- `Test Files 8 passed (8); Tests 15 passed (15)`
- `1 passing` WebdriverIO spec
- `Finished 2 bundles`

## Residual Risks

- Debug builds intentionally retain their development console.
- Counting `conhost.exe` processes is not a reliable assertion inside the terminal-based test runner; the PE subsystem header is the deterministic release check.
- macOS behavior is unchanged and was not part of this Windows-specific reproduction.

## Recommendation

Close the bug. The release executable no longer requests a Windows console, and application diagnostics are written to the configured log directory.
