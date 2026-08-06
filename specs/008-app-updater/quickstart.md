# Quickstart: App Updater

## Prerequisites

- Node 24+, Rust toolchain, Tauri platform build prerequisites
- Authenticated `gh` CLI with write access to `Cekavis/see-see`
- macOS certificate secrets `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`
- Encrypted updater private key stored outside the repository and matching the public key in `src-tauri/tauri.conf.json`

## Repository Secrets

Set these without printing their values:

```powershell
Get-Content -Raw $env:USERPROFILE\.tauri\see-see.key | gh secret set TAURI_SIGNING_PRIVATE_KEY
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
gh secret set APPLE_CERTIFICATE
gh secret set APPLE_CERTIFICATE_PASSWORD
```

## Automated Validation

```powershell
npm run format:check
npm run lint
npm test
npm run test:macos-signing
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.tauri\see-see.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<private local value>"
npm run tauri build
```

Expected Windows bundles include MSI and NSIS installers, matching `.sig` updater signatures, and valid version `0.7.0` metadata inputs.

## Manual About-Page Validation

1. Open Settings > About at normal desktop width and the compact breakpoint.
2. Confirm the installed version is visible and the check action is keyboard reachable.
3. Verify current, available with multiline notes, checking, download progress, and recoverable failure states.
4. Confirm repeated activation cannot start a second operation.
5. With a valid staged update, install and verify the application restarts into the new version.

## Release Validation

1. Commit and push the synchronized version to `origin/master`.
2. Push annotated tag `vX.Y.Z`.
3. Confirm the GitHub release remains draft while matrix jobs run.
4. Confirm the published release contains one MSI, one NSIS setup executable, two DMGs, signed updater assets, `latest.json`, and generated notes.
5. Download `latest.json` and confirm Windows x64, macOS Apple Silicon, and macOS Intel platform entries resolve to uploaded signed artifacts.

## Platform Gap

Windows packaging and installation are validated locally. macOS signing, packaging, installation, screen-permission continuity, and visual review require macOS runners/devices and must be recorded from the release workflow or a real Mac.

## Validation Record — 2026-08-06

- Passed Prettier, ESLint, 50 Vitest tests, 3 macOS signing tests, release configuration assertions, frontend build, and the full Rust test suite.
- Built signed Windows MSI and NSIS installers with both updater signatures.
- Installed See See `0.7.0` to `%LOCALAPPDATA%\See See`; the user completed and approved the About-page visual review.
- Stored the updater signing Secrets in GitHub Actions; Apple certificate Secrets remain an external prerequisite.
- macOS build, installation, permission continuity, and visual checks remain unavailable on this Windows workstation.
