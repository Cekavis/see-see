# Bug Assessment: Tauri cannot import the stable custom macOS certificate

- **Slug**: macos-release-custom-cert-import
- **Created**: 2026-08-06
- **Source**: GitHub Actions run 31067029676 and Tauri `tauri-macos-sign` source
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

The release workflow imports `See See Local Release`, but Tauri's certificate path rejects it during identity discovery even when `APPLE_SIGNING_IDENTITY` is set. The official Tauri `keychain::identity::list` implementation only searches Apple-standard common-name prefixes and requires an organizational unit.

## Symptom

Both macOS architectures compile successfully but cannot reach codesign or DMG generation with the stable self-signed certificate. The expected behavior is to preserve that existing certificate for TCC continuity while allowing Tauri to sign and package both architectures.

## Reproduction

1. Pass the stable PKCS#12 through `APPLE_CERTIFICATE` and its password to the Tauri CLI.
2. Set `APPLE_SIGNING_IDENTITY` to `See See Local Release`.
3. Run a macOS Tauri bundle build.
4. Observe a successful import followed by `failed to resolve signing identity`.

## Suspected Code Paths

- `.github/workflows/release.yml:93` — gives the certificate directly to Tauri, activating Tauri's Apple-only identity discovery path.
- `scripts/verify-release-config.mjs:24` — does not distinguish manual certificate import from passing the certificate to the Tauri action.
- Tauri `crates/tauri-macos-sign/src/keychain/identity.rs:list()` — searches only Apple-standard certificate prefixes and requires an organizational unit.

## Root Cause Hypothesis

Confidence: high. `APPLE_SIGNING_IDENTITY` only controls the default-keychain path. When `APPLE_CERTIFICATE` is also provided, Tauri creates its own temporary keychain and ignores the configured identity while discovering an Apple team certificate. The custom self-signed certificate has the stable common name required by existing installations but intentionally lacks Apple team metadata.

## Proposed Remediation

**Preferred**: Add a macOS-only workflow step that decodes and imports the PKCS#12 into a temporary keychain, adds that keychain to the user search list, and verifies the exact identity. Remove `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` from the Tauri action environment while retaining `APPLE_SIGNING_IDENTITY`. Add an `always()` cleanup step for the temporary keychain.

**Files likely to change**:

- `.github/workflows/release.yml`
- `scripts/verify-release-config.mjs`
- `.specify/bugs/macos-release-signing-identity/fix.md`
- `.specify/bugs/macos-release-signing-identity/test.md`

**Tests to add or update**:

- Require the manual import and cleanup steps in `npm run test:release-config`.
- Assert that the Tauri action environment no longer contains `APPLE_CERTIFICATE`.
- Run the full tag-triggered GitHub release workflow and verify signed DMGs plus publication.

## Risks & Considerations

- The temporary keychain must be unlocked and grant codesign partition access non-interactively.
- The keychain must be appended to, not replace, the runner's existing search list.
- Secret files and the temporary keychain must be removed even on failure.
- The existing certificate must not be rotated because that would undermine TCC permission continuity.

## Open Questions

- None.
