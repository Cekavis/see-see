# Validation: Result Model Retry

Date: 2026-09-28. Version: 0.17.0. Platform: Windows x64.

## Automated evidence

- New frontend and backend regressions were run before their implementations and failed for the missing feature.
- Final frontend suite: 16 files, 112 tests passed. Focused integration checks cover every source state, original-result preservation, selected model/run IPC identities, Escape dismissal before window closure, and StrictMode subscription cleanup.
- Chooser coverage includes 500 name-only rows; no search/count/progress copy; loading and submission errors; duplicate selection; fresh opening; stale load/submission completions; arrows, Home/End, Tab cycling, Escape, and focus restoration.
- The Windows async-return regression failed before the retry command was changed to async; final lifecycle and analysis tests passed (34 total). Four StrictMode regressions failed before effect cleanup and passed afterward, preserving legitimate repeated text/thinking tokens and ignoring stale attach responses.
- `cargo test --manifest-path src-tauri/Cargo.toml --test analysis_flow`: 10 passed. Four added regressions verify frozen input, selected or absent credentials, independent cancellation in all source states, and missing-source errors.
- `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast`: complete suite passed. The existing optional native-credential tests remain ignored by their original configuration.
- Two earlier full Rust attempts hit the unchanged `streaming_client_allows_a_long_first_token_wait` test with a response-decoding error; its focused rerun and final complete suite passed. The previous feature's validation also records this intermittent test. No assertions or tests were disabled or changed to work around it.
- ESLint, TypeScript/Vite production build, Rust formatting, and `git diff --check` passed.
- `npm run test:e2e`: existing primary desktop smoke flow passed.
- Changed frontend files and feature documents were formatted with Prettier.

## Existing unrelated check failures

- `npm run format:check` reports only unchanged `AGENTS.md`.
- `npm run test:release-config` fails because its unchanged verifier expects `uploadUpdaterJson: true`; the unchanged workflow uses `false` with separately generated updater metadata. This also appears in the previous feature's validation.

## Visual verification

A local fixture rendered the actual App and Result components with fictional configuration data and a mocked IPC backend. No real model request or credential was used.

At 460×500 and 420×360, the chooser holds 500 rows in an internal scrollbar and has no horizontal page overflow. Cancellation stays visible. Rows show only the saved configuration name at 32px height with 4px spacing; long names use single-line ellipsis and retain the full name in the title and accessible label. Search, model counts, and loading/creation progress text are absent. Verified arrow/Home/End navigation, Enter selection, empty state, loading-error retry, selection-error recovery, Escape dismissal with trigger focus restoration, and explicit Tab/Shift+Tab cycling. Selecting configuration 500 opened a distinct mocked child result with the selected name while preserving the source.

Screenshots:

- `result-460x500.png`
- `result-420x360.png`
- `chooser-460x500.png`
- `chooser-420x360.png`
- `chooser-load-error.png`
- `chooser-submit-error.png`

The browser review identified that native modal focus could move outside the chooser at its boundaries. A focused regression was added, confirmed failing, and an explicit Tab loop implemented and verified in tests and the browser.

## Packaging and delivery state

`npm run tauri build` succeeded with the existing local signing credential, without printing key/password contents. The 0.17.0 installers were rebuilt after the final chooser, async-command, and stream-cleanup fixes without another version bump. It produced both Windows x64 installers and updater signatures:

- `src-tauri/target/release/bundle/msi/See See_0.17.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/See See_0.17.0_x64-setup.exe`

After the user explicitly approved closing the application and installing, the NSIS installer exited 0. The installed 0.17.0 executable matches the complete release binary after the documented Tauri NSIS bundle marker substitution (`UNK` → `NSS`); its initial raw hash difference was confined to those three marker bytes. The installed application restarted from the expected local path and reported Responding=true with the See See main window. The implementation, evidence, and synchronized version are ready for one atomic master commit/push. No release tag is requested or created.

## Remaining verification

- Native Windows result retry and native Escape interaction with a real provider were not performed; local installation and main-window startup were verified.
- No macOS native runtime validation was performed.
- New native windows use the existing `start_analysis` creation path; automated tests and mocked browser interactions do not replace native end-to-end validation.
- Detailed bug verification, including partial native coverage, is recorded in `.specify/bugs/result-retry-dev-stream/test.md`.
