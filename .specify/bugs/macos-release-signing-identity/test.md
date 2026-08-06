# Bug Verification: macOS release cannot resolve self-signed signing identity

- **Slug**: macos-release-signing-identity
- **Tested**: 2026-08-06
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

The local release configuration test passed, but the original GitHub Actions symptom still reproduced on both macOS architectures. Tauri imported one identity and received `APPLE_SIGNING_IDENTITY`, then still failed with `failed to resolve signing identity`.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | GitHub Actions run `31067029676` | fail | Apple Silicon job `92506800091` reproduced the original failure. |
| New / updated tests | `npm run test:release-config` | pass | The workflow contains the configured identity. |
| Regression suite | `npm test` | pass | 14 files and 50 tests passed. |
| Lint / type-check | `npm run lint` and `npm run format:check` | pass | No local issues. |

## Output Excerpts

```text
APPLE_SIGNING_IDENTITY: See See Local Release
1 identity imported.
failed to bundle project: failed codesign application: failed to resolve signing identity
```

## Residual Risks

- Passing the identity name does not override Tauri's certificate-import resolution path.
- Rotating to an Apple-shaped certificate would change the signing identity and risk losing macOS TCC permission continuity.

## Recommendation

Reassess with the new Tauri source evidence and use a manual temporary-keychain import so Tauri signs by the stable identity name without processing `APPLE_CERTIFICATE` itself.
