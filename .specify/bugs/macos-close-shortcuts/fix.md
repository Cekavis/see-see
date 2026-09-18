# Bug Fix: macOS window close shortcuts

- **Slug**: macos-close-shortcuts (from the assessment in this task)
- **Fixed**: 2026-09-18
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Added an application-local AppKit key-down monitor so macOS close shortcuts are handled before native responders or WKWebView consume them. The handler calls Tauri's existing close path: result windows close and cancel their own analysis, while the main window hides and the application remains running.

## Changes

| File                            | Change                                    | Notes                                                                                                                                                                      |
| ------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/windowing.rs`    | Added native monitor and regression test  | Handles Escape for results and Command/Ctrl+W for main/results; matches the event's native window, consumes handled events and repeats, and rejects Shift/Option variants. |
| `src-tauri/src/lib.rs`          | Registered the monitor during macOS setup | One monitor for the application lifetime; no per-result registrations.                                                                                                     |
| `src/App.test.ts`               | Added Command+W coverage                  | Main/results, physical key fallback, and Shift/Option exclusions.                                                                                                          |
| Five synchronized version files | Updated to 0.16.1                         | Enabled required features of the existing AppKit dependency; no new dependency.                                                                                            |

## Tests Added or Updated

- `windowing::tests::macos_close_keys_respect_window_scope_and_modifiers`: native shortcut matching, result-only Escape, unrelated windows, modifiers, and Caps Lock.
- `src/App.test.ts`: Command+W behavior for main and result windows and modified shortcut exclusions.

## Local Verification

- `npm run lint`, `npm run format:check`, `npm test`, and `npm run build`: passed; 89 frontend tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed; 116 tests, with two pre-existing credential-store tests ignored.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `npm run test:e2e`: passed.
- `npm run tauri build`: produced the signed app and DMG, then stopped at updater signing because this Mac has no updater private key.
- `npm run tauri build -- --config '{"bundle":{"createUpdaterArtifacts":false}}'`: passed; local build override only, repository release configuration unchanged.
- `npm run verify:macos-signature`: passed with keychain access. Installed and new bundles have the same designated requirement and `See See Local Release` certificate.
- Installed 0.16.1 at `/Applications/See See.app`; installation and build binary SHA-256 values match. Backed up 0.16.0 at `/private/tmp/See See-0.16.0-backup.app`.
- Native UI: reproduced the old main-window Command+W failure, then verified the new main window stays open on Escape and hides on Command+W. The user confirmed the fix works on their Mac.

## Deviations from Assessment

None in implementation. Updater artifact signing was disabled only for the local installation build because its private key is unavailable on this Mac.

## Follow-ups

Windows manual checks were not available on this host; the Windows implementation is unchanged. No GitHub release was requested or published.
