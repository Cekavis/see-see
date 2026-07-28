# Bug Assessment: Windows build warns about macOS-only capture helpers

- **Slug**: windows-macos-dead-code-warnings
- **Created**: 2026-07-28
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

The Windows Rust build reports three `dead_code` warnings that appear to come from macOS-only capture code. Remove the warnings without hiding legitimate unused-code diagnostics.

## Symptom

`cargo check --manifest-path src-tauri/Cargo.toml` succeeds on Windows but warns that `permission_status_from_grant`, `MonitorMetadata`, and `frozen_monitor_from_bgra` are unused. These symbols should not be compiled into a normal Windows library build because they are used only by the macOS capture implementation and unit tests.

## Reproduction

1. Check out commit `8367144` on Windows.
2. Run `cargo check --manifest-path src-tauri/Cargo.toml`.
3. Observe three `dead_code` warnings in `src-tauri/src/capture.rs`.

## Suspected Code Paths

- `src-tauri/src/capture.rs:28` — helper is called by macOS permission code and tests only.
- `src-tauri/src/capture.rs:303` — metadata type is consumed by the macOS capture module and tests only.
- `src-tauri/src/capture.rs:311` — BGRA adapter is consumed by the macOS capture module and tests only.
- `src-tauri/src/capture/macos.rs` — confirms all production callers are macOS-specific.

## Root Cause Hypothesis

High confidence: the three shared definitions lack the same platform boundary as their production callers. The macOS module is excluded on Windows, leaving these definitions present but unused in the normal Windows library target. Unit tests use them only when Rust compiles the test configuration.

## Proposed Remediation

**Preferred**: add `#[cfg(any(target_os = "macos", test))]` to the three definitions. This keeps them available to macOS production code and cross-platform unit tests while removing them entirely from normal non-macOS builds.

**Alternatives**:
- `#[allow(dead_code)]` would silence rather than fix the platform-boundary mismatch and could hide future unused code.

**Files likely to change**:
- `src-tauri/src/capture.rs`

**Tests to add or update**:
- No new test is needed; existing capture adapter and permission mapping tests require the definitions under `cfg(test)`.
- Run `cargo check` to prove the Windows warnings are gone and `cargo test` to prove the existing tests still compile and pass.

## Risks & Considerations

- An incorrect condition could remove symbols from macOS production builds; retain `target_os = "macos"` explicitly.
- macOS compilation should be checked if the target toolchain is available locally.

## Open Questions

- None.
