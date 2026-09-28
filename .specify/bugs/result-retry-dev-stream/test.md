# Bug Verification: Model retry window hangs and development streaming duplicates

- **Slug**: result-retry-dev-stream
- **Tested**: 2026-09-28
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

The development duplication was reproduced by broadcasting real analysis events to both StrictMode subscriptions; the regression passes after cleanup. Windows retry now satisfies the required asynchronous command contract, and the signed application builds, installs, and starts successfully. Native retry with a real provider and macOS interaction were not exercised, so native end-to-end verification remains partial.

## Checks Performed

All npm commands below were run through external `powershell.exe -Command`.

| Check                                      | Command / Action                                                               | Result  | Notes                                                                                                                                     |
| ------------------------------------------ | ------------------------------------------------------------------------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| StrictMode reproduction                    | App tests with every registered channel receiving each delta once              | pass    | Before: duplicated output and stale notifications; after: `aab` and `思思考` preserved exactly                                            |
| Async dispatch regression                  | Compile-time Future constraint for model retry                                 | pass    | Before: E0277 compile failure; after: native window commands all return Futures                                                           |
| Chooser behavior                           | 500 rows, duplicate names, quiet loading/pending, keyboard and lifecycle tests | pass    | No search/count/creation message; ID-based selection preserved                                                                            |
| Frontend full suite                        | `npm test`                                                                     | pass    | 16 files / 112 tests                                                                                                                      |
| Rust full suite                            | `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast`               | pass    | All enabled tests pass; two existing native-credential tests ignored                                                                      |
| Lint / frontend build                      | `npm run lint`; `npm run build`                                                | pass    | ESLint, TypeScript, Vite                                                                                                                  |
| Rust formatting                            | `cargo fmt --manifest-path src-tauri/Cargo.toml --check`                       | pass    | Exit 0                                                                                                                                    |
| Global formatting                          | `npm run format:check`                                                         | fail    | Only unchanged `AGENTS.md`; scoped changed files pass                                                                                     |
| Existing release configuration gate        | `npm run test:release-config`                                                  | fail    | Unchanged verifier expects `uploadUpdaterJson: true`, while unchanged workflow generates metadata separately                              |
| Browser smoke                              | `npm run test:e2e`                                                             | pass    | Primary flow                                                                                                                              |
| Browser visuals and interaction            | Real App with fictional saved models and mocked IPC                            | pass    | 460×500 and 420×360; 32px rows; 500 items; no horizontal overflow; Home/End, arrows, Tab, Escape; failures recover; distinct child result |
| Native package                             | `npm run tauri build` with existing signing credentials                        | pass    | NSIS/MSI bundles plus updater signatures                                                                                                  |
| Local installation and startup             | User-approved NSIS `/S`; binary comparison; start installed app                | pass    | Installer exit 0; version 0.17.0; expected NSIS marker; Responding=true                                                                   |
| Native provider retry / development stream | Manual interaction in Windows Tauri                                            | not-run | Automated equivalents cover command scheduling and subscription lifecycle, not a live provider/native window                              |
| macOS interaction                          | Manual desktop verification                                                    | not-run | No macOS runtime used                                                                                                                     |

## Output Excerpts

```text
Before backend fix: E0277 — Result<AnalysisStarted, AppError> is not a future
Before frontend fix: 思思思思考考 (expected 思思考)
Test Files 16 passed (16)
Tests 112 passed (112)
See See primary desktop flow
Installer exit code: 0
Verified complete installed binary against build with the documented NSIS bundle marker.
```

## Residual Risks

- Native Windows retry and macOS result-window behavior still need direct runtime interaction. Browser child-window testing used mocked IPC.
- The two unrelated repository-wide checks above remain failing; no assertions, coverage, or production settings were weakened to pass them.

## Recommendation

The code fixes, automated regressions, UI checks, packaging, and local installation are complete. Retain partial native verification status until a real Windows retry and macOS desktop interaction are observed.
