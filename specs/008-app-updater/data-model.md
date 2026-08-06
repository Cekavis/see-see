# Data Model: App Updater

## Update Availability

- `version`: remote semantic version
- `body`: optional release notes
- `installHandle`: verified platform update returned by the updater

Exists only after a successful check finds a newer signed release.

## Update Operation

- `status`: `idle | checking | current | available | installing | restarting | failed`
- `downloadedBytes`: accumulated downloaded bytes
- `totalBytes`: optional declared content length
- `errorMessage`: recoverable user-facing failure text

### State Transitions

```text
idle/current/failed -> checking
checking -> current | available | failed
available -> installing
installing -> restarting | failed
```

Checking and installation actions are disabled in `checking`, `installing`, and `restarting`.

## Published Release

- `versionTag`: annotated `vX.Y.Z` tag matching all synchronized version files
- `notes`: installer guidance plus generated release notes
- `installers`: Windows MSI, Windows NSIS, Apple Silicon DMG, Intel DMG
- `updaterArtifacts`: signed platform update bundles and `.sig` files
- `metadata`: `latest.json` with Windows x64, macOS Apple Silicon, and macOS Intel entries
- `visibility`: draft until validation, then published/latest
