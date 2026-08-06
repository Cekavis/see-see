# Bug Fix: macOS updater artifacts are missing from release

- **Slug**: macos-updater-bundle-target
- **Fixed**: 2026-08-06
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Both macOS matrix entries now build the updater-enabled `app` target together with the installer DMG. Publish verification requires both signed updater archives before checking Darwin entries and making the release public.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Changed both macOS builds to `--bundles app,dmg`. |
| `.github/workflows/release.yml` | modified | Added exact counts for macOS updater archives and signatures. |
| `scripts/verify-release-config.mjs` | updated test | Requires both combined macOS bundle targets and updater asset assertions. |
| `.specify/bugs/macos-release-headless-trust/test.md` | added verification | Recorded successful macOS signing/build jobs. |

## Tests Added or Updated

- `scripts/verify-release-config.mjs` — requires two `--bundles app,dmg` occurrences and both updater asset checks.

## Local Verification

- Commands run: `npm run test:release-config` → passed.
- Commands run: `npm run format:check` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm test` → 14 files and 50 tests passed.
- Manual checks: successful macOS logs explicitly warned that `dmg` alone is not updater-enabled and listed the absent `.app.tar.gz` paths.

## Deviations from Assessment

None.

## Follow-ups

- Recreate the unpublished `v0.7.0` tag on the pushed fix commit and verify updater assets plus final publication.
