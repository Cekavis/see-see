# Bug Verification: Result copy button remains visible

- **Slug**: result-copy-button-clipped
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

The corrected three-row CSS contract, focused regression, full frontend and Rust suites, release build, installation, and installed version check all pass. A fresh long-result window was not produced because doing so would retransmit a stored user screenshot to the configured external model provider without explicit approval.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Inspect the 460 × 500 result layout contract and launch installed 0.3.6 app | partial | CSS now constrains the text to the flexible row; a new provider-backed result run was intentionally not submitted. |
| New / updated tests | `npm test -- src/views/Result.test.tsx` | pass | 3 tests passed, including the result-grid regression. |
| Frontend regression suite | `npm test` | pass | 12 files and 38 tests passed. |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | All unit, integration, benchmark, and doc tests passed with localhost mock-server access. |
| Lint / format / type-check | `npm run lint && npm run format:check && npm run build` | pass | ESLint, Prettier, TypeScript, and Vite production build passed. |
| macOS signing tests | `npm run test:macos-signing` | pass | 3 tests passed. |
| Release package | `npm run tauri build` | pass | Signed `.app` and `See See_0.3.6_aarch64.dmg` were produced. |
| Local installation | Install to `/Applications/See See.app`, restart, inspect About page | pass | Installed application reports version 0.3.6. |
| Browser E2E | Layout harness with Playwright | not-run | The pinned Playwright browser executable is not installed in this environment. |
| Windows manual check | Windows result window | not-run | Current verification host is macOS. |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests       38 passed (38)
```

```text
test result: ok. 6 passed; 0 failed
Finished 2 bundles at:
  .../bundle/macos/See See.app
  .../bundle/dmg/See See_0.3.6_aarch64.dmg
```

```text
About See See → Version 0.3.6
```

## Residual Risks

- The exact user-provided long result was not resubmitted after installation, so the final native result-window appearance remains unobserved.
- Windows manual verification remains outstanding.

## Recommendation

The implementation is safe to ship based on the deterministic layout correction and passing regression coverage. Confirm the next naturally occurring long result keeps the copy button fully visible; no special retest submission is needed unless immediate native-window confirmation is desired.
