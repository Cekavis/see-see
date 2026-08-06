# Bug Verification: Tauri cannot import the stable custom macOS certificate

- **Slug**: macos-release-custom-cert-import
- **Tested**: 2026-08-06
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

The manual import path avoided Tauri's Apple-only identity resolver, but the first implementation added user trust and then gated on `security find-identity`. Both macOS jobs stalled in the import step on headless GitHub runners, so the run was canceled.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | GitHub Actions run `31068109689` | fail | Both macOS jobs remained in `Import macOS signing certificate` until cancellation. |
| New / updated tests | `npm run test:release-config` | pass | Static workflow contract passed but did not detect interactive trust behavior. |
| Local untrusted certificate experiment | Import fresh self-signed code-signing certificate without trust, then run `codesign` | pass | `find-identity` reported 0 valid identities, but signing and designated-requirement verification succeeded. |
| Regression suite | `npm test` | pass | 14 files and 50 tests passed. |

## Output Excerpts

```text
0 valid identities found
signed-test: valid on disk
signed-test: satisfies its Designated Requirement
Authority=See See Untrusted CI Test
```

## Residual Risks

- `security add-trusted-cert` can require UI interaction on a fresh headless runner.
- `security find-identity` rejects an otherwise usable untrusted self-signed codesign identity.

## Recommendation

Keep the manual temporary-keychain import, remove trust mutation and `find-identity`, and verify the imported certificate plus its signing-capable private key non-interactively before invoking Tauri.
