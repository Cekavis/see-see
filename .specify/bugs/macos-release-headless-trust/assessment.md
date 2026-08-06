# Bug Assessment: Headless macOS runner stalls while trusting certificate

- **Slug**: macos-release-headless-trust
- **Created**: 2026-08-06
- **Source**: GitHub Actions run 31068109689 and local untrusted-certificate experiment
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

Both macOS release jobs remain indefinitely in the manual certificate import step after the workflow calls `security add-trusted-cert` and before Tauri starts. A fresh locally generated self-signed code-signing certificate proves that trust mutation and `security find-identity` are unnecessary for signing.

## Symptom

The manual keychain import workflow blocks on headless GitHub runners instead of reaching Tauri. The expected behavior is a completely non-interactive import that verifies the certificate and signing private key, then lets codesign use the identity from the keychain search list.

## Reproduction

1. Import a self-signed code-signing PKCS#12 into a temporary keychain on a fresh headless macOS runner.
2. Call `security add-trusted-cert` for user-domain code-signing trust.
3. Observe the workflow step remain in progress without reaching Tauri.
4. By contrast, import a fresh untrusted certificate, add the keychain to the search list, and call `codesign -s <identity>`; signing succeeds even though `security find-identity` reports zero valid identities.

## Suspected Code Paths

- `.github/workflows/release.yml:120` — mutates user trust in a headless environment.
- `.github/workflows/release.yml:139` — treats `security find-identity` as authoritative even though it excludes usable untrusted self-signed identities.
- `scripts/verify-release-config.mjs:30` — requires the interactive trust command instead of non-interactive certificate/key checks.

## Root Cause Hypothesis

Confidence: high. User trust settings are distinct from keychain import and may require SecurityAgent interaction on a fresh runner. Additionally, `find-identity` reports trust-valid identities rather than all private-key-backed identities usable by codesign. The release only needs the certificate and a signing-capable private key in a searched, unlocked keychain.

## Proposed Remediation

**Preferred**: Remove `security add-trusted-cert` and the `find-identity` gate. Verify the exact certificate with `security find-certificate` and the exact signing-capable private key with `security find-key -t private -s -l`, then invoke Tauri with `APPLE_SIGNING_IDENTITY`.

**Files likely to change**:

- `.github/workflows/release.yml`
- `scripts/verify-release-config.mjs`
- `.specify/bugs/macos-release-custom-cert-import/fix.md`

**Tests to add or update**:

- Require the non-interactive private-key check in `npm run test:release-config`.
- Reject `security add-trusted-cert` in the workflow.
- Run the full tag-triggered release workflow to publication.

## Risks & Considerations

- The private key label must match the certificate common name in the exported PKCS#12.
- The temporary keychain still must be unlocked, added to the search list, and granted codesign partition access.
- The workflow must continue cleaning keychain and certificate files with `always()`.

## Open Questions

- None.
