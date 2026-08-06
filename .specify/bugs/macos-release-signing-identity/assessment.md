# Bug Assessment: macOS release cannot resolve self-signed signing identity

- **Slug**: macos-release-signing-identity
- **Created**: 2026-08-06
- **Source**: GitHub Actions run 31065847046, jobs 92504947933 and 92504947945
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

The `v0.7.0` release workflow imports the `See See Local Release` PKCS#12 successfully on the macOS runner, but both Apple Silicon and Intel jobs fail during bundling with `failed codesign application: failed to resolve signing identity`.

## Symptom

The macOS release jobs compile the application and import one valid code-signing identity, but Tauri does not select the project's self-signed identity automatically. The expected behavior is for both macOS jobs to sign with `See See Local Release`, create their DMGs, and allow the publish job to run.

## Reproduction

1. Configure `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` with the `See See Local Release` identity.
2. Push the annotated `v0.7.0` tag to trigger `.github/workflows/release.yml`.
3. Observe `1 identity imported.` followed by `failed to resolve signing identity` in each macOS build job.

## Suspected Code Paths

- `.github/workflows/release.yml:93` — passes the certificate to Tauri but does not set `APPLE_SIGNING_IDENTITY`.
- `scripts/verify-release-config.mjs:24` — validates updater signing variables but does not assert the macOS identity selection required for a self-signed certificate.

## Root Cause Hypothesis

Confidence: high. Tauri can import the PKCS#12 but does not automatically select a custom self-signed identity. The workflow must pass the exact keychain identity through `APPLE_SIGNING_IDENTITY`; without it, code signing stops before DMG creation.

## Proposed Remediation

**Preferred**: Set `APPLE_SIGNING_IDENTITY` to `See See Local Release` in the `tauri-apps/tauri-action` environment and extend the release configuration test to require that exact mapping.

**Files likely to change**:

- `.github/workflows/release.yml`
- `scripts/verify-release-config.mjs`

**Tests to add or update**:

- Update `npm run test:release-config` so the workflow cannot omit or rename the expected macOS signing identity.
- Re-run the failed GitHub release jobs to exercise certificate import, identity selection, code signing, DMG creation, updater artifact upload, and final publication.

## Risks & Considerations

- The configured value must match the certificate common name exactly.
- The release tag currently points to the pre-fix commit; because the release remains an unpublished draft, it must be recreated on the pushed fix commit before rerunning the full workflow.
- Existing partial draft assets should be removed with the failed draft to prevent duplicate release asset names.

## Open Questions

- None.
