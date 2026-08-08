# Implementation Plan: Result Window Main Navigation

**Branch**: `master` | **Date**: 2026-08-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-result-open-main/spec.md`

## Summary

Add an always-available "打开主窗口" action to the existing result footer. Route the action through one Tauri command that reuses the current main-window show/focus path and checks the backend analysis state before closing the matching result window. Active analyses remain untouched; terminal results close through the existing window-close cleanup path.

## Technical Context

**Language/Version**: TypeScript 6, React 19; Rust 2024 edition

**Primary Dependencies**: Existing React hooks, Tauri 2 IPC/window APIs, Testing Library, Vitest; no new dependency

**Storage**: Existing in-memory runtime analysis state; no schema or settings change

**Testing**: Vitest 4 with Testing Library; Rust unit/integration tests with `cargo test`; lint, formatting, frontend build, Tauri release build, and local Windows installation

**Target Platform**: Tauri 2 desktop application on Windows and macOS

**Project Type**: Desktop application with React/TypeScript UI and Rust backend

**Performance Goals**: Show and focus the existing main window immediately from one user action without restarting or recreating it

**Constraints**: Never cancel an active analysis; use authoritative backend state; preserve result-window minimum sizing and existing controls; add no dependency or abstraction

**Scale/Scope**: One result-footer action, one IPC wrapper, one Rust command, and focused frontend/Rust regressions

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **Maintainability**: Pass. Reuses the shared Button, result footer, current main-window focus helper, existing terminal-state predicate, and close-event cleanup. No dependency or new layer is added.
- **Testing**: Pass. Focused component and IPC tests cover the action contract; Rust tests cover terminal-state boundaries and the existing full suites cover window lifecycle regressions.
- **User experience**: Pass. The action is present in every result state and failures use the current notification behavior. Active output is preserved.
- **UI quality**: Pass. Existing footer/button styles provide keyboard and focus behavior. Normal and 420×360 visual review are included.

Post-design re-check: all gates remain passed. No constitution exception is required.

## Project Structure

### Documentation (this feature)

```text
specs/010-result-open-main/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── result-main-navigation.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── App.tsx
├── ipc.ts
├── ipc.test.ts
└── views/
    ├── Result.tsx
    └── Result.test.tsx

src-tauri/
├── src/
│   ├── commands.rs
│   ├── lib.rs
│   └── state.rs
└── tests/
    └── desktop_lifecycle.rs

package.json
package-lock.json
src-tauri/Cargo.toml
src-tauri/Cargo.lock
src-tauri/tauri.conf.json
```

**Structure Decision**: Keep presentation in the existing Result component, wire it through ResultView and the existing IPC object, and place authoritative state/close behavior in the Rust command layer. Reuse the existing result close event for runtime cleanup.

## Complexity Tracking

No constitution violations.
