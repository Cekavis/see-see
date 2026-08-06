# Research: App Updater

## Update Integration

**Decision**: Use the official Tauri updater JavaScript binding in the About view and initialize its Rust plugin in the application builder.

**Rationale**: It provides signed metadata verification, platform installer selection, download progress, and installation without a custom IPC contract or updater implementation.

**Alternatives considered**: A Rust command wrapper would add state and IPC code without improving the single-view flow. Opening the release page would not satisfy one-click installation.

## Restart Integration

**Decision**: Use the official process plugin's restart capability after installation.

**Rationale**: Restart behavior is platform-specific and already implemented by Tauri.

**Alternatives considered**: Quitting and asking the user to reopen adds another manual step. A custom process spawn is less portable and harder to secure.

## Distribution Endpoint

**Decision**: Use the public GitHub Releases `latest.json` asset as the updater endpoint.

**Rationale**: The repository already publishes releases there, and the Tauri action can generate correct signed platform metadata from the same builds.

**Alternatives considered**: A separate update server adds infrastructure, credentials, and availability concerns with no current benefit.

## Release Assembly

**Decision**: Create a draft first, build the three target architectures in a matrix, validate installers and updater metadata, then publish the draft as latest.

**Rationale**: Users never see a partially populated release, while failed jobs can be retried against the retained draft.

**Alternatives considered**: Publishing from each matrix job exposes partial releases. A separate artifact aggregation workflow adds unnecessary transfers and scripts.

## Signing

**Decision**: Keep platform signing and updater signing separate. Reuse `See See Local Release` for macOS application identity and generate one encrypted Tauri updater key whose public key is embedded in the app and private key is stored locally plus in GitHub Secrets.

**Rationale**: Tauri requires updater signatures on all platforms, while the existing macOS certificate protects screen-capture permission identity.

**Alternatives considered**: Unsigned updates are rejected by design. Ad-hoc macOS signing would break the stable TCC identity. Committing a private key is prohibited.
