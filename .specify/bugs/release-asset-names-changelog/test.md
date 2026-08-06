# Bug Verification: Release asset names and notes are malformed

- **Slug**: release-asset-names-changelog
- **Tested**: 2026-08-06
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The malformed filename and release-note symptoms no longer reproduce. Both the repaired v0.7.0 release and the newly published v0.7.1 release use `See-See_` asset names, contain only Full Changelog in the body, and retain functional updater artifacts.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Inspect v0.7.0 and v0.7.1 through `gh release view` | pass | No `See.See_`, spaced product prefix, or duplicate extension separator remains. |
| New / updated tests | `npm run test:release-config` | pass | Correct pattern is required; custom installer notes and malformed patterns are rejected. |
| Frontend regression | `npm run build`; `npm test` | pass | Production frontend build passed; 14 files and 50 tests passed. |
| Rust regression | `cargo test --manifest-path src-tauri/Cargo.toml` outside sandbox | pass | All unit, integration, benchmark, and doc-test targets passed. |
| Quality checks | `npm run lint`; `npm run format:check`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | pass | ESLint, Prettier, and rustfmt passed. |
| Release workflow | GitHub Actions run `31071108138` | pass | Prepare, Windows, both macOS architectures, and publish jobs succeeded. |
| Asset-name verification | Inspect v0.7.1 assets and Windows upload log | pass | All ten binary/signature assets use `See-See_0.7.1_`; upload log contains no duplicate dots. |
| Release body | Inspect v0.7.1 body | pass | Body is only `Full Changelog` comparing v0.7.0 to v0.7.1. |
| Updater verification | Inspect v0.7.1 `latest.json` | pass | Windows x64, Darwin aarch64, and Darwin x86_64 entries exist with signatures. |
| macOS signature | `codesign --verify --deep --strict` on Apple Silicon DMG app | pass | Identifier is `app.seesee.desktop`; authority is `See See Local Release`. |
| Local installation | Install Apple Silicon DMG to `/Applications/See See.app` | pass | Installed version reports 0.7.1 and passes strict signature verification. |

## Output Excerpts

```text
Uploading See-See_0.7.1_windows_x64.msi...
See-See_0.7.1_windows_x64.msi successfully uploaded.
installed_version=0.7.1
/Applications/See See.app: valid on disk
Authority=See See Local Release
```

## Residual Risks

- The local macOS 27 / Rust 1.97.1 release toolchain produced corrupted proc-macro dylibs during a local release build. GitHub's clean macOS runners built and signed both architectures successfully, so this does not affect the published release but remains a separate local-toolchain issue.

## Recommendation

Close the bug — verified end-to-end on the published release and the locally installed Apple Silicon application.
