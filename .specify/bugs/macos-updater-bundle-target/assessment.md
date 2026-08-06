# Bug Assessment: macOS updater artifacts are missing from release

- **Slug**: macos-updater-bundle-target
- **Created**: 2026-08-06
- **Source**: GitHub Actions run 31068510004, publish job 92512830916
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

Both macOS jobs sign and upload their DMGs successfully, but Tauri warns that no updater-enabled target was built. The draft `latest.json` contains only Windows platforms, so the publish job fails its Darwin updater assertions.

## Symptom

The release has two valid macOS DMGs but no `.app.tar.gz` updater archives or signatures, and macOS clients cannot discover the release through `latest.json`. The expected behavior is to publish both installer DMGs and signed updater archives for Apple Silicon and Intel.

## Reproduction

1. Run the macOS matrix entries with `--bundles dmg` while `createUpdaterArtifacts` is enabled.
2. Observe Tauri's warning that no updater-enabled targets were built.
3. Inspect the draft assets and find only DMGs for macOS.
4. Inspect `latest.json` and find only Windows platform entries.

## Suspected Code Paths

- `.github/workflows/release.yml:74` — Apple Silicon builds only the `dmg` target.
- `.github/workflows/release.yml:77` — Intel builds only the `dmg` target.
- `.github/workflows/release.yml:166` — publish verifies Darwin JSON entries but not the updater archive assets that produce them.
- `scripts/verify-release-config.mjs:27` — does not require an updater-enabled macOS `app` target.

## Root Cause Hypothesis

Confidence: high. In Tauri 2, `dmg` is an installer target but not an updater-enabled target. The macOS `app` target must also be built so Tauri can create and sign `See See.app.tar.gz`; tauri-action then uploads the archive/signature and adds Darwin entries to `latest.json`.

## Proposed Remediation

**Preferred**: Change both macOS matrix arguments to `--bundles app,dmg`. Extend publish verification to require exactly two `.app.tar.gz` archives and two matching `.sig` files, and extend the release configuration test to require the combined bundle target.

**Files likely to change**:

- `.github/workflows/release.yml`
- `scripts/verify-release-config.mjs`

**Tests to add or update**:

- Require two macOS `--bundles app,dmg` matrix entries.
- Require updater archive/signature asset assertions.
- Re-run the full release workflow and inspect `latest.json` for both Darwin platforms.

## Risks & Considerations

- The release will upload additional macOS updater assets beyond the user-facing DMGs.
- Asset name matching must distinguish `.app.tar.gz` from `.app.tar.gz.sig`.
- The current release is still an unpublished draft and can be safely recreated on the fix commit.

## Open Questions

- None.
