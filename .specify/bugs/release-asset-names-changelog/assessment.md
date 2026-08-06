# Bug Assessment: Release asset names and notes are malformed

- **Slug**: release-asset-names-changelog
- **Created**: 2026-08-06
- **Source**: pasted text and published GitHub release v0.7.0
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

The published files use names such as `See.See_0.7.0_windows_x64.msi`, while the GitHub Actions upload log first constructs `See See_0.7.0_windows_x64..msi`. The release body also contains installer-selection instructions, but it should contain only the generated full changelog.

## Symptom

Release asset names contain a dot-substituted product name and are generated from a template with a duplicate extension separator. The release notes prepend installer instructions instead of containing only GitHub's generated changelog.

## Reproduction

1. Push an annotated version tag that triggers `.github/workflows/release.yml`.
2. Observe the upload log construct names with `..` before the extension.
3. Inspect the published assets and observe that GitHub normalizes `See See` to `See.See`.
4. Inspect the release body and observe the `## 安装包` section before the full changelog.

## Suspected Code Paths

- `.github/workflows/release.yml:60` — passes custom installer notes alongside `--generate-notes`.
- `.github/workflows/release.yml:148` — uses `[name]`, which contains spaces, and adds a literal dot before `[ext]`, which already includes its own leading dot.
- `scripts/verify-release-config.mjs` — does not lock down the release asset pattern or reject custom installer notes.

## Root Cause Hypothesis

Confidence: high. `tauri-action` documents `[ext]` as values such as `.msi` and `.dmg`; therefore `.[ext]` creates a duplicate separator in the action's intermediate name. Its GitHub-safe name function replaces spaces with dots and collapses consecutive dots, producing published names such as `See.See_...`. Separately, the workflow explicitly supplies the installer section through `gh release create --notes`, so generated release notes include both that section and the full changelog.

## Proposed Remediation

**Preferred**: Use a filesystem-safe literal product prefix and the extension placeholder without an extra dot: `See-See_[version]_[platform]_[arch][setup][ext]`. Remove the custom `--notes` argument while retaining `--generate-notes`. Strengthen release verification to require the exact asset prefixes and add configuration regression assertions.

Repair v0.7.0 by renaming every existing `See.See_` asset to `See-See_`, updating the matching URLs inside `latest.json`, and replacing the release body with its generated full changelog only.

**Files likely to change**:

- `.github/workflows/release.yml`
- `scripts/verify-release-config.mjs`
- synchronized version files for the corrective patch release

**Tests to add or update**:

- Require `releaseAssetNamePattern: "See-See_[version]_[platform]_[arch][setup][ext]"`.
- Reject `--notes` and the installer-description heading in the workflow.
- Require published installer and updater asset names to begin with `See-See_<version>_`.
- Verify the repaired release body and updater URLs through the GitHub API.

## Risks & Considerations

- Renaming updater archives without updating `latest.json` would break update downloads.
- Existing signatures remain valid because only GitHub asset names and URLs change, not archive contents.
- A new patch release must use the corrected workflow so the fix persists beyond v0.7.0.

## Open Questions

- None.
