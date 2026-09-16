# Implementation Plan: Result Window Size Memory

**Branch**: `master` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/017-result-window-size-memory/spec.md`

## Summary

Set the result-window default to 460×540 logical pixels, remember the latest user-selected logical dimensions across result windows and normal application restarts, and cap result/history screenshot previews at 60 pixels high. Reuse the existing SQLite application-settings store, result-window event path, and image style rules; do not add a visible settings control or dependency.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 2024 / TypeScript 6

**Primary Dependencies**: Tauri 2, React 19, rusqlite

**Storage**: Existing local SQLite `app_settings` row; two nullable integer columns for logical result-window width and height

**Testing**: Rust unit/integration tests, Vitest component/style assertions, formatter, lint, TypeScript build, native build

**Target Platform**: Windows and macOS desktop

**Project Type**: Tauri desktop application

**Performance Goals**: Resizing remains responsive; a resize is kept in memory immediately and committed only on result-window close or explicit application quit.

**Constraints**: Preserve the 420×540 minimum, support high-DPI monitors by persisting logical rather than physical dimensions, maintain upgrade compatibility for existing local databases, and retain existing result/history layout behavior.

**Scale/Scope**: One persisted window preference and three existing image style rules; no new screen, IPC command, dependency, or history schema change.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Maintainability**: Pass. Reuses the existing `app_settings` migration and settings-access patterns, plus the existing result-window event handler. No dependency or duplicate preferences store is introduced.
- **Testing**: Pass. Adds persisted-storage, migration, window-size selection, resize/close lifecycle, and CSS regression coverage.
- **User experience**: Pass. The preference applies silently and does not change existing result actions, error recovery, or navigation.
- **UI quality**: Pass. Existing image elements, labels, width constraints, object fit, and result/history layouts remain in use; visual review covers default, minimum, and remembered dimensions.

## Project Structure

### Documentation (this feature)

```text
specs/017-result-window-size-memory/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
src/
├── styles.css                         # Result and history image-height rules
└── views/
    ├── Result.test.tsx                # Result image style regression assertion
    └── History.test.tsx               # History list/detail image style assertions

src-tauri/
├── migrations/
│   └── 0009_result_window_size.sql    # Adds nullable persisted dimensions
├── src/
│   ├── database.rs                    # Applies and validates the migration
│   ├── settings.rs                    # Loads/saves persisted dimensions
│   ├── state.rs                       # Keeps the current session's latest size
│   ├── windowing.rs                   # Defaults, validation, DPI conversion, builder size
│   ├── commands.rs                    # Uses remembered dimensions and flushes on quit
│   └── lib.rs                         # Records result-window resize and flushes it on close
└── tests/
    ├── storage_foundation.rs          # Persistence-after-reopen regression test
    └── desktop_lifecycle.rs           # Result-window lifecycle and default-size contract tests
```

**Structure Decision**: Retain the existing React frontend and Rust Tauri backend split. The preference is a native application setting because window creation and native resize events occur in Rust; only CSS changes are needed in the frontend.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations or added complexity require justification.
