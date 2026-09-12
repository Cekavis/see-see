# Implementation Plan: Remember Result Window Position

**Branch**: `master` | **Date**: 2026-09-12 | **Spec**: [spec.md](spec.md)

## Summary

Record the most recent position of any result window in the existing application runtime state. When a new result window is created, use that position before showing it; if no position has been recorded, retain the existing centered placement. The implementation stays in the native Tauri window lifecycle and does not add a user-facing setting or database migration.

## Technical Context

**Language/Version**: Rust 2024; existing TypeScript/React frontend is unchanged

**Primary Dependencies**: Tauri 2.11 window events and window APIs; existing `Mutex<RuntimeState>`

**Storage**: In-memory application runtime state; no SQLite persistence for this session-scoped feature

**Testing**: `cargo test --manifest-path src-tauri/Cargo.toml`; focused Rust unit and desktop lifecycle regression tests

**Target Platform**: Tauri desktop targets, with explicit Windows and macOS behavior coverage and Linux-compatible native APIs

**Project Type**: Desktop application

**Performance Goals**: Position updates remain constant-time and do not perform disk or network I/O during window movement

**Constraints**: Preserve current centering fallback, compact dimensions, minimum dimensions, focus, always-on-top, fullscreen-space, close, and analysis behavior; use physical screen coordinates consistently across displays and scale factors

**Scale/Scope**: One shared optional position for all result windows during one application session; no cross-restart persistence and no frontend IPC contract changes

## Constitution Check

*GATE: Pass before Phase 0 research; re-checked after Phase 1 design.*

- **Maintainability**: Pass. Reuses `RuntimeState`, `WindowEvent`, `windowing::present_result_window`, and the existing result-window label helper. No new dependency or persistence layer.
- **Testing**: Pass. Adds pure runtime-state coverage and source/lifecycle regression checks for move-event filtering and placement order.
- **User experience**: Pass. First-open behavior remains centered; later windows follow the user's latest placement without changing existing windows or controls.
- **UI quality**: Pass. No new visual pattern. Manual review covers the existing compact result-window sizes and both initial and remembered placement states.

## Project Structure

### Documentation (this feature)

```text
specs/015-result-window-position/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code

```text
src-tauri/
├── src/
│   ├── commands.rs       # Read the remembered position when creating results
│   ├── lib.rs            # Record result-window move events
│   ├── state.rs          # Store the session-scoped position
│   └── windowing.rs      # Apply remembered or centered placement before show
└── tests/
    └── desktop_lifecycle.rs  # Native lifecycle regression coverage
```

**Structure Decision**: Keep the change in the existing Rust/Tauri desktop layers. `commands.rs` owns result-window creation, `lib.rs` owns global window events, `state.rs` owns shared runtime data, and `windowing.rs` owns platform-specific presentation. No frontend or database changes are needed.

## Implementation Design

1. Add an optional `ResultWindowPosition` value to `RuntimeState` with a small update helper so the latest coordinates replace older ones.
2. Extend the global window-event handler to handle `Moved` only for labels recognized by `result_run_id`, then update the shared runtime position. Close handling remains unchanged and does not clear the value.
3. Read the optional position in `create_result_window` before building the window and pass it to `present_result_window`.
4. Update `present_result_window` to apply a remembered physical position before show/focus, or call the existing center behavior when the option is empty. Keep the macOS main-thread policy setup and all existing errors/ordering intact.
5. Add focused tests for the runtime state and the result-window event/presentation source contract, then run the Rust test suite and project validation commands.

## Complexity Tracking

None. The feature adds one optional in-memory value and reuses existing window lifecycle hooks.
