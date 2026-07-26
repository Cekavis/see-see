# Bug Verification: Re-capture fails while a result window is open

- **Slug**: result-window-recapture
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

All automated checks pass and confirm that each translation receives a unique result window, that older windows cannot clear a newer analysis, and that the frontend routes those windows correctly. The exact live reproduction was not run because completing it would use the user's configured external model, so verification remains partial pending one manual double-capture check.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Automated equivalent: unique result labels plus stale-window isolation tests | pass | Conflicting fixed-label creation path no longer exists; live model request not run. |
| New / updated tests | Focused Rust tests and `npm test -- --run src/windowLabels.test.ts` | pass | All three regression scenarios passed. |
| Regression suite | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 41 frontend tests and all Rust unit/integration/doc tests passed. Rust tests requiring local mock ports were rerun outside the restricted sandbox. |
| Lint / type-check | `npm run format:check`; `npm run lint`; `npm run build` | pass | Formatting, ESLint, TypeScript, and Vite production build passed. |
| Release package | `npm run tauri build`; `npm run verify:macos-signature` | pass | Created signed 0.4.1 app and DMG; signature verified. |
| Local install | Install bundle to `/Applications/See See.app`, then inspect version and signature | pass | Installed version is 0.4.1 with identifier `app.seesee.desktop` and authority `See See Local Release`. |

## Output Excerpts

```text
Test Files  13 passed (13)
Tests       41 passed (41)

test result: ok. 12 passed; 0 failed
test result: ok. 6 passed; 0 failed  # provider contracts

Finished 2 bundles at:
  .../bundle/macos/See See.app
  .../bundle/dmg/See See_0.4.1_aarch64.dmg

0.4.1
Identifier=app.seesee.desktop
Authority=See See Local Release
```

## Residual Risks

- A live sequence of completing one translation, leaving its result open, and completing a second translation was not exercised because it would invoke the configured external model.
- Windows manual verification remains outstanding.
- Reloading an older completed result window is not supported by the existing single-active-analysis in-memory architecture; keeping the loaded window open is supported.

## Recommendation

The implementation and regression coverage are ready. Perform one manual double-capture translation with an approved model configuration on macOS; if both result windows remain available and no error appears, close the bug as verified. Repeat the window behavior check on Windows before the next cross-platform release.
