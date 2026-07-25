# Quickstart: Validate Unified App Feedback

## Prerequisites

- Node.js 24 or newer
- Rust toolchain required by the existing Tauri project
- macOS test host for native visual inspection

## Automated validation

Run from the repository root:

```bash
npm run format:check
npm run lint
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected results:

- Notification tests prove error persistence, success timeout, repeated-message identity, recovery action, dismissal, and semantic roles.
- Existing view tests prove errors and useful successes publish through the shared service.
- The frontend and Rust suites remain green.

## Desktop package validation

```bash
npm run tauri build
```

Install the resulting macOS application bundle locally, launch See See, and use native UI inspection for the following scenarios.

## Scenario A: Error remains visible in a compact scrolled settings window

1. Resize the main window to approximately 320×240 or the smallest permitted size.
2. Open **模型**, scroll the form so the page header is not visible, and submit an invalid or incomplete connection test.
3. Confirm the error appears within the current viewport in under one second.
4. Confirm the page does not scroll, no blank layout space appears, the complete message can be read, and the dismiss button is reachable.

## Scenario B: Success is visible and temporary

1. At a compact window size, trigger a successful operation that has a useful confirmation, such as copying result text or saving a valid configuration.
2. Confirm success uses the same placement and structure with lighter visual emphasis.
3. Confirm it remains for approximately four seconds, can be manually dismissed, and its removal does not move content.

## Scenario C: Recovery action

1. Trigger a recoverable loading error using a test double or a temporarily unavailable local service.
2. Activate **重试** using only the keyboard.
3. Confirm the old error is removed before retry and a later outcome is not contradicted by stale feedback.

## Scenario D: Multiple messages and long text

1. Trigger several outcomes close together, including the same message twice.
2. Confirm newest-first ordering, independent dismissal, no overlap, and internal vertical scrolling when required.
3. Confirm an unusually long endpoint or model identifier wraps with no horizontal clipping.

## Scenario E: Result window and appearance modes

1. Trigger an analysis failure and a successful copy action in the compact result window.
2. Confirm the notification stays inside the current result viewport.
3. Repeat representative checks in available light and dark appearances and with reduced motion enabled.

Record native observations in the feature task list or final verification report.
