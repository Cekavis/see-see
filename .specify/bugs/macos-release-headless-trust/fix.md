# Bug Fix: Headless macOS runner stalls while trusting certificate

- **Slug**: macos-release-headless-trust
- **Fixed**: 2026-08-06
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The macOS import step no longer changes user trust or gates on trust-valid identities. It verifies the exact certificate and signing-capable private key non-interactively, while preserving the temporary keychain import, search-list setup, and cleanup.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Removed `add-trusted-cert` and `find-identity`; added exact certificate/private-key checks. |
| `scripts/verify-release-config.mjs` | updated test | Requires `security find-key` and rejects trust mutation. |
| `.specify/bugs/macos-release-custom-cert-import/fix.md` | updated report | Recorded the headless trust finding and follow-up. |
| `.specify/bugs/macos-release-custom-cert-import/test.md` | added verification | Captured the stalled/canceled end-to-end run. |

## Tests Added or Updated

- `scripts/verify-release-config.mjs` — requires non-interactive private-key validation and fails if `security add-trusted-cert` returns.

## Local Verification

- Commands run: `npm run test:release-config` → passed.
- Commands run: `npm run format:check` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm test` → 14 files and 50 tests passed.
- Manual checks: a fresh untrusted self-signed code-signing certificate signed and verified a test executable without trust mutation; `Authority=See See Untrusted CI Test` was present even though `find-identity` reported zero valid identities.

## Deviations from Assessment

None.

## Follow-ups

- Recreate the unpublished `v0.7.0` tag on the pushed fix commit and verify the full release workflow.
