# Bug Assessment: Windows launch opens an extra console

- **Slug**: windows-console-logs
- **Created**: 2026-07-23
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (summarized)

Launching See See opens the onboarding window and an additional terminal window. The terminal displays Tauri debug output such as `Asset favicon.ico not found`. The expected behavior is to show only application windows and write logs to the application log directory.

## Symptom

The Windows release executable is built as a console application, and the logging plugin writes to stdout as well as the log directory. Users therefore see a console window containing runtime logs.

## Reproduction

1. Build or install See See 0.1.0 on Windows.
2. Launch the application from Explorer or the Start menu.
3. Observe an extra console window beside the onboarding window.

## Suspected Code Paths

- `src-tauri/src/main.rs:1` — the binary entry point lacks the standard release-only Windows GUI subsystem attribute.
- `src-tauri/src/lib.rs` — `tauri_plugin_log::Builder::new()` keeps its default stdout and log-directory targets.

## Root Cause Hypothesis

Confidence: high. The binary is linked with the Windows console subsystem, while `tauri-plugin-log` defaults to both `Stdout` and `LogDir`. The observed terminal and log lines follow directly from those two defaults.

## Proposed Remediation

**Preferred**: mark release builds as Windows GUI applications in `main.rs`, and replace the logging plugin targets with a single `LogDir` target. Keep debug builds attached to their development terminal. Increment the patch version to 0.1.1.

**Files likely to change**:

- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- Compile and run existing Rust tests.
- Build the Windows release bundle and verify the PE subsystem is GUI rather than console.
- Launch the installed build and confirm logs are created in the application log directory without a console window.

## Risks & Considerations

- Release-mode panic output will no longer be visible in a console; it must be diagnosed from logs.
- Debug builds intentionally retain terminal output for development.
- The missing favicon message may remain in the log file; it is unrelated to the extra window.

## Open Questions

None.
