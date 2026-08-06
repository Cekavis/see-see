# Bug Verification: macOS updater artifacts are missing from release

- **Slug**: macos-updater-bundle-target
- **Tested**: 2026-08-06
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The full release workflow succeeded and published `v0.7.0`. Both macOS updater archives and signatures are present, `latest.json` contains Apple Silicon and Intel Darwin mappings, and both public DMGs verify with the stable signing authority.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | GitHub Actions run `31069383229` | pass | Both macOS jobs, Windows, and publish succeeded. |
| Updater assets | Public `v0.7.0` asset list | pass | Two `.app.tar.gz` archives and two 404-byte signatures are present. |
| Updater manifest | Download and inspect `latest.json` | pass | Contains `darwin-aarch64`, `darwin-x86_64`, and Windows mappings with non-empty signatures. |
| macOS signature | `codesign --verify --verbose=2 <dmg>` | pass | Both DMGs are valid and satisfy their designated requirements. |
| Stable authority | `codesign -d --verbose=4 <dmg>` | pass | Both DMGs report `Authority=See See Local Release`. |
| New / updated tests | `npm run test:release-config` | pass | Combined `app,dmg` targets and updater asset assertions are required. |
| Regression suite | `npm test` | pass | 14 files and 50 tests passed. |

## Output Excerpts

```text
publish succeeded in 3s
See.See_0.7.0_darwin_aarch64.dmg: valid on disk
See.See_0.7.0_darwin_x64.dmg: valid on disk
Authority=See See Local Release
```

## Residual Risks

- Windows installers were built and asset-validated by GitHub Actions but were not installed on this macOS workstation.
- The self-signed macOS authority preserves identity continuity but does not provide Apple notarization.

## Recommendation

Close the bug — verified end-to-end through public release publication.
