# Bug Fix: Release Workflow Publish Validation Failure

- **Slug**: release-workflow-missing-updater-platform
- **Fixed**: 2026-09-16
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The parallel build jobs no longer upload `latest.json`. The existing publish job now generates one updater manifest after all installer assets are available, validates all seven required platform entries, uploads it once, and only then publishes the release.

The existing `v0.15.1` release was repaired using its already-successful installer and signature assets and published successfully.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Disable per-matrix updater JSON uploads; generate and validate `latest.json` once in `publish`. |

## Tests Added or Updated

- No application tests were needed; the change is limited to release automation.

## Local Verification

- `npm exec prettier -- --check .github/workflows/release.yml` → pass.
- Extracted publish shell script → `bash -n` pass.
- Rebuilt `v0.15.1`'s updater manifest from the existing release assets → 7 platform entries validated.
- Uploaded the repaired `latest.json` and ran `gh release edit v0.15.1 --draft=false --latest` → pass.
- Final release verification → 11 assets, `isDraft: false`, and all required updater entries present.
- `npm run format:check` → failed on the pre-existing formatting issue in `AGENTS.md`; no application files were changed.

## Deviations from Assessment

The assessment recommended a dedicated post-build updater job. The fix keeps the existing `publish` job and makes it the single serialized manifest-generation step, avoiding an additional job while preserving the same concurrency guarantee.

## Follow-ups

- Verify the fixed workflow end-to-end on the next version tag.
