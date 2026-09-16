# Bug Assessment: Release Workflow Publish Validation Failure

- Date: 2026-09-16
- Repository: `Cekavis/see-see`
- Affected commit/tag: `b203247570bcb0101982358ad1cbbff59e7572d6` / `v0.15.1`
- Workflow run: `35087633672`
- Status: Valid bug
- Severity: High — the release remains a draft and the updater manifest is incomplete.

## Report

The GitHub Release workflow for `v0.15.1` completed all build jobs but failed at the final publish validation step.

## Evidence

1. `prepare` succeeded, including version checks and signing-secret presence checks.
2. All three matrix build jobs succeeded: Windows, macOS Apple Silicon, and macOS Intel.
3. All 11 expected release assets were uploaded, including both macOS updater archives and signatures.
4. The `publish` job failed at `.github/workflows/release.yml:201-208`, where `jq` requires:
   - `windows-x86_64`
   - `darwin-aarch64`
   - `darwin-x86_64`
5. The final `latest.json` for `v0.15.1` contains `darwin-aarch64` and the Windows entries, but does not contain `darwin-x86_64`. The corresponding `v0.15.0` manifest contains both macOS platform entries.
6. The release is currently a draft (`publishedAt: null`), so the final `gh release edit --draft=false --latest` command was not reached.

## Root cause

Each build matrix job enables `uploadUpdaterJson: true` at `.github/workflows/release.yml:136-149`. Therefore, the three jobs concurrently read, regenerate, delete, and upload the same `latest.json` asset. The last writer can overwrite entries written by another matrix job with a manifest derived from only the artifacts visible to that job at that moment.

The GitHub Actions `concurrency` group in this workflow serializes separate workflow runs for the same ref; it does not serialize the jobs inside one matrix run. The observed upload timestamps confirm overlapping `latest.json` uploads. The final file retained the ARM Mac and Windows entries but lost Intel Mac, which caused the explicit `jq` validation to fail.

This is a race in updater-manifest generation, not a missing secret, build failure, asset-name mismatch, or signing failure.

## Recommended remediation

Disable `uploadUpdaterJson` in the parallel build jobs and generate/upload `latest.json` exactly once in a serialized job after all build jobs complete. That job should validate all required platform keys before publishing the release.

Loosening the `jq` assertion to accept only `darwin-aarch64-app` would mask the incomplete updater manifest and is not a sufficient fix.

## Verification plan

- Confirm each matrix build still uploads its installer, archive, and signature assets.
- Confirm the single updater-manifest step produces `darwin-aarch64`, `darwin-x86_64`, and the required Windows entries in one `latest.json`.
- Confirm the publish job transitions the release from draft to published only after those checks pass.
