# Bug Fix: Screen recording permission prompt repeats on every launch

- **Slug**: screen-recording-permission-loop
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Replaced the macOS capture-based permission probe with a passive CoreGraphics preflight and separated permission checking, explicit user-requested authorization, Settings recovery, and post-Settings display refresh. Actual capture now stops with a specific recoverable permission error before invoking xcap when access is unavailable.

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src-tauri/src/capture.rs` | modified | Uses `CGPreflightScreenCaptureAccess` for passive macOS checks, `CGRequestScreenCaptureAccess` only for explicit requests, adds a capture guard and permission-specific error mapping. |
| `src-tauri/src/commands.rs` | modified | Guards capture before xcap, focuses the main window on missing access, and exposes the explicit permission request command. |
| `src-tauri/src/lib.rs` | modified | Registers the new permission request IPC command. |
| `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock` | modified | Adds the target-specific CoreGraphics binding and updates the package version. |
| `src/ipc.ts` | modified | Adds the shared permission type and request IPC wrapper. |
| `src/views/Onboarding.tsx` | modified | Separates request and Settings recovery states, preserves denial during passive refresh, refreshes on focus, and shows recovery after completed onboarding if access is revoked. |
| `src/views/Onboarding.test.tsx` | updated tests | Covers explicit request, denial recovery, focus refresh, and revoked-access display. |
| `src/styles.css` | modified | Lays out the new permission recovery controls responsively. |
| `package.json` / `package-lock.json` / `src-tauri/tauri.conf.json` | modified | Synchronizes the patch release at `0.2.3`. |
| `AGENTS.md` | modified | Records the user's repository-level preference to auto-generate Spec Kit workflow names without confirmation. |

## Tests Added or Updated

- `capture::tests::passive_preflight_maps_missing_access_without_claiming_denial` — pins granted and unknown passive-preflight mapping without invoking capture.
- `capture::tests::capture_permission_guard_and_backend_error_use_recovery_code` — pins the pre-capture denial guard and backend permission error mapping.
- `src/views/Onboarding.test.tsx` — pins explicit authorization requests, denied recovery, passive focus refresh, and post-onboarding recovery display.

## Local Verification

- Commands run: `npm test -- --run src/views/Onboarding.test.tsx` → passed, 6 tests.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml capture --lib --test capture_flow` → passed the 2 new permission unit tests and existing matching library test; the name filter excluded the unrelated `capture_flow` cases.
- Commands run: `npm run typecheck` → passed.
- Manual checks: code-path inspection confirmed application snapshot loading no longer calls xcap on macOS; native authorization is reachable only from the explicit request command.

## Deviations from Assessment

- Permission guard tests were placed beside the private status-mapping and backend-error helpers in `src-tauri/src/capture.rs` instead of `src-tauri/tests/capture_flow.rs`, allowing direct coverage without exposing implementation details.
- `src/styles.css`, lockfiles, and `AGENTS.md` were added to the changed-file set. Styling supports the new recovery actions, lockfiles reflect dependency/version metadata, and `AGENTS.md` is the user's separate workflow preference requested during this fix.

## Follow-ups

- Complete the full regression, lint, formatting, release build, locally installed app launch, and relaunch verification in `test.md` before publishing.
