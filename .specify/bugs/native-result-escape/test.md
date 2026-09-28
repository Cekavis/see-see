# Bug Verification: Native result Escape

- **Slug**: native-result-escape (resolved from the current assessment/fix context)
- **Tested**: 2026-09-28
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

The reported Windows regression is fixed and verified through real WebView2 windows and OS keyboard input. Overall verification remains partial because the unchanged full Rust suite has a documented intermittent failure, global formatting reports unchanged `AGENTS.md`, and macOS native interaction was not available on this Windows host.

## Checks Performed

| Check                          | Command / Action                                                                                                                                       | Result                               | Notes                                                                                                                                                                                                                                                                            |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows native reproduction    | Start `powershell.exe -Command "npm run dev -- --host 127.0.0.1"`, then `cargo run --manifest-path src-tauri/Cargo.toml --example result_escape_smoke` | pass (0)                             | Actual production capabilities reject the former frontend close call. Actual native Escape closes; chooser Escape dismisses only the chooser; held Escape does not close; focus returns; the next Escape closes. Ctrl+W closes with chooser open. All three scenarios completed. |
| Frontend regression suite      | `powershell.exe -Command "npm test"`                                                                                                                   | pass (0)                             | 16 files, 122 tests. Includes chooser acknowledgement, errors, cleanup, duplicate prevention, 500-model list and existing StrictMode stream checks.                                                                                                                              |
| Backend lifecycle tests        | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle`                                                                             | pass (0)                             | 26 tests.                                                                                                                                                                                                                                                                        |
| Backend state/native decisions | Focused library runs for `state::tests` and `windowing::tests`                                                                                         | pass (0)                             | 8 state and 2 native-decision tests.                                                                                                                                                                                                                                             |
| Full Rust suite                | `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast`                                                                                       | fail (1), existing intermittent test | 27 library and 95 integration tests passed; one unchanged first-token-wait test failed while decoding a response. Two existing credential-store tests remain ignored by their original declarations.                                                                             |
| First-token test in isolation  | `cargo test --manifest-path src-tauri/Cargo.toml --lib providers::tests::streaming_client_allows_a_long_first_token_wait -- --exact`                   | pass (0)                             | No source, assertions or timeout changed. Prior failures are documented in `specs/019-result-model-retry/validation.md` and `specs/016-result-window-image-preview/validation.md`.                                                                                               |
| Browser regression flow        | `powershell.exe -Command "npm run test:e2e"`                                                                                                           | pass (0)                             | Primary desktop flow. Owned Vite server was stopped first.                                                                                                                                                                                                                       |
| Lint                           | `powershell.exe -Command "npm run lint"`                                                                                                               | pass (0)                             | Full ESLint.                                                                                                                                                                                                                                                                     |
| Frontend typecheck/build       | `powershell.exe -Command "npm run build"`                                                                                                              | pass (0)                             | TypeScript and Vite production build.                                                                                                                                                                                                                                            |
| Formatting                     | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and Prettier on changed frontend/fixture files                                                | pass (0)                             | Scoped formatting passed.                                                                                                                                                                                                                                                        |
| Global formatting              | `powershell.exe -Command "npm run format:check"`                                                                                                       | fail (1), existing file              | Only unchanged `AGENTS.md` is reported; left outside this bug's scope.                                                                                                                                                                                                           |
| Signed Windows package         | `powershell.exe -Command "npm run tauri build"` with existing local signing configuration                                                              | pass (0)                             | Built 0.17.0 MSI/NSIS and both updater signatures. No signing material recorded.                                                                                                                                                                                                 |
| Local installation             | Run `See See_0.17.0_x64-setup.exe /S`                                                                                                                  | pass (0)                             | Existing user authorization covered closing/installing the app. Installer exited 0.                                                                                                                                                                                              |
| Installed binary and startup   | Compare installed bytes to built binary after the single Tauri `UNK` to `NSS` bundle marker replacement; start installed app                           | pass (0)                             | Exact binary match. File version 0.17.0. Started from the installation directory and was responding.                                                                                                                                                                             |
| macOS native keys              | Escape and Cmd+W in macOS result windows                                                                                                               | not-run                              | Windows host; no claim of macOS runtime verification.                                                                                                                                                                                                                            |

## Output Excerpts

```text
PASS: production capabilities reject the former frontend close route
PASS: native close for scenario 0
PASS: native Escape dismissed only the chooser
PASS: held Escape did not close the source window
PASS: native close for scenario 1
PASS: native close for scenario 2

Test Files  16 passed (16)
     Tests  122 passed (122)

InstallerExitCode=0
Installed binary exactly matches this build with the NSIS bundle marker.
```

Installed executable SHA-256: `4731338bf166249d691ef3adc4387033cfe29eeb19fc35d5207af77fd1c855fe`.

## Residual Risks

- macOS shares the native decision logic and received removal of synthetic injection, but its native runtime path still needs a macOS check.
- The existing first-token test can fail in a full parallel Rust run. This fix does not touch provider transport code or weaken that test.
- Native smoke uses the actual UI and native handler with isolated in-memory data. It does not send a real paid provider request or inspect user credentials. Source-result retry behavior remains covered by existing frontend/backend tests.

## Recommendation

Deliver the Windows corrective fix: its original symptom and chooser-first behavior are verified through the native event path, and the rebuilt application is installed. Keep the macOS validation gap and unrelated pre-existing check failures explicit; do not present the complete repository check set as entirely green.
