# Bug Fix: Release asset names and notes are malformed

- **Slug**: release-asset-names-changelog
- **Fixed**: 2026-08-06
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Release assets now use the stable `See-See_` prefix without a duplicate extension separator, and release creation no longer injects installer instructions. The published v0.7.0 release was repaired in place, and the persistent workflow fix is prepared as version 0.7.1.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | modified | Uses `See-See_[version]_[platform]_[arch][setup][ext]`, generates changelog-only notes, and verifies the exact asset set before publishing. |
| `scripts/verify-release-config.mjs` | updated test | Requires the corrected pattern and rejects custom installer notes, spaced `[name]`, and `.[ext]`. |
| `package.json` | modified | Bumped the corrective release to 0.7.1. |
| `package-lock.json` | modified | Synchronized version 0.7.1. |
| `src-tauri/Cargo.toml` | modified | Synchronized version 0.7.1. |
| `src-tauri/Cargo.lock` | modified | Synchronized the application package version 0.7.1. |
| `src-tauri/tauri.conf.json` | modified | Synchronized version 0.7.1. |
| GitHub release `v0.7.0` | modified | Renamed ten assets from `See.See_` to `See-See_` and reduced the body to Full Changelog only. |

## Diff Highlights

```yaml
releaseAssetNamePattern: "See-See_[version]_[platform]_[arch][setup][ext]"
```

The `gh release create` invocation retains `--generate-notes` and no longer passes `--notes`.

## Tests Added or Updated

- `scripts/verify-release-config.mjs` — pins the safe asset pattern and changelog-only release configuration.
- The publish job compares the complete actual asset list against eleven expected filenames before making a draft public.

## Local Verification

- `npm run test:release-config` → passed.
- `npm run build` → passed.
- `npm test` → 14 files and 50 tests passed.
- `npm run lint` → passed.
- `npm run format:check` → passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` outside the sandbox → passed.
- GitHub API readback of v0.7.0 → all ten binary/signature assets use `See-See_`; body contains only Full Changelog; asset IDs and binary digests are unchanged.
- `npm run tauri build` → application frontend compiled, but the local macOS 27 / Rust 1.97.1 release toolchain produced invalid proc-macro dylibs (`mis-aligned LINKEDIT` / missing derive crates) before application linking. Formal release build is delegated to GitHub's clean macOS runner.

## Deviations from Assessment

- `latest.json` uses stable GitHub asset API IDs rather than filename URLs. Renaming the existing assets preserved those IDs, so the updater URLs did not require textual changes; the reuploaded file retained its original SHA-256 digest.

## Follow-ups

- Publish and verify v0.7.1 on GitHub Actions, including exact asset names, updater entries, macOS signatures, and changelog-only body.
- Investigate the local macOS 27 Rust release proc-macro corruption separately if local release builds are needed on this machine.
