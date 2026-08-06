# Bug Fix: Tauri cannot import the stable custom macOS certificate

- **Slug**: macos-release-custom-cert-import
- **Fixed**: 2026-08-06
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The workflow now imports and trusts the stable self-signed certificate in a temporary macOS keychain before invoking Tauri. The Tauri action receives only the exact signing identity, so it uses the default-keychain signing path instead of its Apple-team certificate discovery path.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Added manual macOS PKCS#12 import, trust, search-list setup, identity verification, and cleanup. |
| `.github/workflows/release.yml` | modified | Removed `APPLE_CERTIFICATE` and its password from the Tauri action environment while retaining `APPLE_SIGNING_IDENTITY`. |
| `scripts/verify-release-config.mjs` | updated test | Requires manual import/cleanup and rejects passing the certificate directly to Tauri. |
| `.specify/bugs/macos-release-signing-identity/fix.md` | updated report | Recorded why the first remediation was insufficient. |
| `.specify/bugs/macos-release-signing-identity/test.md` | added verification | Captured the failed end-to-end result. |

## Tests Added or Updated

- `scripts/verify-release-config.mjs` — requires manual certificate import, code-signing trust, cleanup, and an action environment without `APPLE_CERTIFICATE`.

## Local Verification

- Commands run: `npm run test:release-config` → passed.
- Commands run: `npm run format:check` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm test` → 14 files and 50 tests passed.
- Manual checks: reproduced the workflow locally by exporting and base64-decoding the PKCS#12, importing and trusting it in a temporary keychain, adding that keychain to the user search list, signing a copied executable by the identity name without `--keychain`, and verifying its designated requirement. `codesign` reported `Authority=See See Local Release`.

## Deviations from Assessment

The first implementation added explicit user-domain code-signing trust, but GitHub Actions verification showed that trust mutation stalls on headless runners. A fresh untrusted-certificate experiment proved that codesign can use the imported identity without trust even though `security find-identity` reports zero valid identities. The non-interactive correction is tracked under `macos-release-headless-trust`.

## Follow-ups

- Recreate the unpublished `v0.7.0` tag on the pushed fix commit and verify the full release workflow.
- Remove the headless trust mutation and validate certificate/private-key presence instead.
