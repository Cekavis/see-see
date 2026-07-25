# Quickstart: Validate Streamlined Model Configuration

## Prerequisites

- Node.js 24+ with locked dependencies installed
- Rust toolchain supported by the repository
- macOS or Windows desktop environment for final native review
- Optional HTTPS model endpoint and test key; a localhost mock may be used to avoid fees

## Automated validation

Run from the repository root:

```bash
npm run format:check
npm run lint
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

The model tests must demonstrate:

- the API key is saved in the same local configuration record as the endpoint and preserved by an edit that omits a replacement;
- a readable legacy credential migrates without loss, while a credential-store failure does not block startup;
- connection testing performs no model-configuration database write;
- any saved configuration can be activated without a test;
- copies preserve connection values, remain inactive, and receive unique bounded names;
- configuration cards do not expose key values and existing redaction tests still pass;
- the editor is absent initially, appears only for add/edit, closes after save/cancel, and remains after a failed save.

## Manual model workflow

1. Launch the app and open “模型”. Confirm no configuration fields are visible and “新增” is available.
2. Click “新增”. Enter a name, protocol, endpoint, model ID, and key.
3. Save without testing. Confirm the editor closes and the new card appears.
4. Set the untested card as current. Confirm it succeeds.
5. Edit the card. Confirm connection fields are restored, the masked key field is blank with a preserve-key hint, and cancel closes the editor without changes.
6. Edit again, leave the key blank, change draft values, and test. Confirm the request uses the saved key plus current draft fields and only a transient success/error notification appears; cancel and confirm the saved card did not change.
7. Copy the card. Confirm a uniquely named inactive copy appears, then edit it and verify all values match the source.
8. Delete the copy through confirmation and verify the original remains current.

## Upgrade validation

1. Start from a version 0.2.4 database containing a model with a system-credential reference.
2. Launch the upgraded app. Edit the model and verify its key is available.
3. Relaunch and use the model with the system credential removed; verify the database value remains usable.
4. Repeat with a missing or inaccessible legacy credential and verify startup succeeds and the model can be edited as keyless.

## Subtitle and visual review

Review General, Models, Prompts, History, About, Onboarding, and Result at 1024×720 and 720×520 in light and dark appearances where available.

Expected outcomes:

- Generic descriptive paragraphs are absent beneath primary page titles.
- Dynamic progress, permission recovery, readiness state, field hints, privacy notes, test-cost warnings, empty-state help, and confirmation text remain where relevant.
- The model list is the default focus of the page, the add action is easy to find, and the conditional editor does not create empty gaps.
- Cards and actions wrap without clipping; focus indicators remain visible; all controls are keyboard operable.

## Release validation

Run the required release build and locally install the generated bundle. On both macOS and Windows when available, repeat the add, save-without-test, test-draft, activate, edit, copy, delete, app-restart, and capture flows.
