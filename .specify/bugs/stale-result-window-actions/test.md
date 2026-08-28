# Bug Verification: Stale result-window actions report a missing analysis

- **Slug**: stale-result-window-actions
- **Tested**: 2026-08-28
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

Automated checks pass and confirm that stale result-window navigation no longer propagates `分析任务不存在`, while active-run terminal navigation and stale cleanup remain guarded. The exact two-window live reproduction was not run because it would invoke the configured external model.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Static Rust regression equivalent for stale `runId` handling | pass | `open_main_window` treats missing analysis as stale terminal and closes only its own window. |
| New / updated tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle stale_result_navigation_does_not_report_missing_analysis` | pass | Focused regression passed. |
| Regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | All Rust unit, integration, and doc tests passed. |
| Frontend tests | `C:\nvm4w\nodejs\npm.cmd test -- --run src/windowLabels.test.ts src/views/Result.test.tsx` | pass | 11 tests passed. |
| Lint / type-check | `C:\nvm4w\nodejs\npm.cmd run lint`; `C:\nvm4w\nodejs\npm.cmd run build`; `C:\nvm4w\nodejs\npm.cmd run format:check` | pass | ESLint, TypeScript/Vite build, and formatting passed. |
| Live multi-window reproduction | Complete two real model runs and click buttons in the older window | not-run | Requires the user's configured external model and manual desktop interaction. |

## Output Excerpts

```text
test stale_result_navigation_does_not_report_missing_analysis ... ok
test result: ok. 1 passed; 0 failed

Test Files  2 passed (2)
Tests       11 passed (11)

Finished `test` profile ...
test result: ok. 18 ...
✓ built in 120ms
```

## Residual Risks

- A live double-capture flow on Windows and macOS remains unverified.
- Reloading an older result window still cannot reconstruct its terminal snapshot because runtime state retains only the current analysis; this fix targets already-loaded windows.
- Retry/cancel from stale windows intentionally remain rejected to prevent mutating a newer run.

## Recommendation

Hold for one manual two-window check with an approved model configuration; automated verification is otherwise passing and the stale-navigation error path is covered.
