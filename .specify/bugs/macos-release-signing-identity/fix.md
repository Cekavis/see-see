# Bug Fix: macOS release cannot resolve self-signed signing identity

- **Slug**: macos-release-signing-identity
- **Fixed**: 2026-08-06
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The release workflow now tells Tauri to use the imported `See See Local Release` identity explicitly. The release configuration test requires the exact identity mapping so future workflow edits cannot silently remove it.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Added `APPLE_SIGNING_IDENTITY` to the Tauri action environment. |
| `scripts/verify-release-config.mjs` | updated test | Requires the exact self-signed identity in the workflow. |

## Tests Added or Updated

- `scripts/verify-release-config.mjs` — fails when the release action does not receive `APPLE_SIGNING_IDENTITY: "See See Local Release"`.

## Local Verification

- Commands run: `npm run test:release-config` → passed.
- Commands run: `npm run format:check` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm test` → 14 files and 50 tests passed.
- Manual checks: the configured identity exactly matches `security find-identity -v -p codesigning` and the certificate imported by GitHub Actions.

## Deviations from Assessment

None.

## Follow-ups

- Recreate the unpublished `v0.7.0` tag on the pushed fix commit and verify the full release workflow.
