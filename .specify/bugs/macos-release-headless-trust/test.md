# Bug Verification: Headless macOS runner stalls while trusting certificate

- **Slug**: macos-release-headless-trust
- **Tested**: 2026-08-06
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The non-interactive import step completed immediately on both macOS runners, and both Apple Silicon and Intel builds signed and packaged successfully. The later publish failure was unrelated: macOS updater targets were not requested.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | GitHub Actions run `31068510004` | pass | Import step completed; both macOS jobs succeeded. |
| New / updated tests | `npm run test:release-config` | pass | Trust mutation is rejected and private-key validation is required. |
| Regression suite | `npm test` | pass | 14 files and 50 tests passed. |
| Lint / format | `npm run lint` and `npm run format:check` | pass | No local issues. |

## Output Excerpts

```text
Signing with identity "See See Local Release"
build (macos ... aarch64) succeeded in 5m42s
build (macos ... x86_64) succeeded in 5m45s
```

## Residual Risks

- The certificate remains self-signed, so distribution trust differs from an Apple Developer ID certificate; this is intentional to preserve the established local signing identity.

## Recommendation

Close this bug — the original headless trust stall is resolved end-to-end.
