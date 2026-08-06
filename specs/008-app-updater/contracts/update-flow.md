# Contract: Update Flow

## About Page

| State | Visible result | Primary action | Action state |
|---|---|---|---|
| Idle | Installed version | Check for updates | Enabled |
| Checking | Checking status | Checking | Disabled, busy |
| Current | Latest-version confirmation | Check again | Enabled |
| Available | New version and release notes | Install version | Enabled |
| Installing | Downloaded percentage or byte count | Installing | Disabled, busy |
| Restarting | Restart status | Restarting | Disabled, busy |
| Failed | Recoverable error notification/status | Retry | Enabled |

- Repeated activation MUST NOT start concurrent checks or installations.
- Release notes MUST render as plain text.
- A failed operation MUST retain the currently installed application.

## Release Workflow

- Trigger: pushed annotated tag matching `v*`.
- Preconditions: tag equals every synchronized version; macOS certificate and updater signing secrets are non-empty.
- Outputs: one MSI, one NSIS setup executable, two DMGs, signed updater bundles/signatures, and `latest.json` entries for all three updater targets.
- Publication rule: the release remains a draft until every required output is verified.
- Retry rule: an existing draft may be reused; an already published tag is immutable and requires a new patch version.
