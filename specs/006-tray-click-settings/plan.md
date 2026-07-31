# Implementation Plan: Tray Click Settings

**Branch**: `master` | **Date**: 2026-07-31 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-tray-click-settings/spec.md`

## Summary

Change the existing tray configuration so primary-button clicks no longer show the menu and a completed primary-button click reuses the existing main-window display path. Keep the existing secondary-click menu and menu actions unchanged.

## Technical Context

**Language/Version**: Rust 1.95, TypeScript 6

**Primary Dependencies**: Tauri 2.11.5

**Storage**: N/A

**Testing**: Rust unit tests with `cargo test`; existing frontend Vitest suite

**Target Platform**: Windows and macOS desktop

**Project Type**: Tauri desktop application

**Performance Goals**: Open and focus the existing settings window immediately after a completed primary click

**Constraints**: Preserve secondary-click menu behavior, reuse the existing window, add no dependency or visual UI

**Scale/Scope**: One tray icon, one existing settings window, one Rust startup module

## Constitution Check

*GATE: Passed before research and after design.*

- **Maintainability**: Reuses `show_main_window` and Tauri's existing tray builder; no abstraction or dependency is added.
- **Testing**: Adds one focused unit test for the button/state predicate and runs the existing suites.
- **User experience**: Keeps existing labels, menu actions, and settings-window behavior; only click routing changes.
- **UI quality**: Adds no visual components or styles; manual desktop interaction review covers the changed behavior.

## Project Structure

### Documentation (this feature)

```text
specs/006-tray-click-settings/
├── checklists/requirements.md
├── contracts/tray-interaction.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
src-tauri/src/lib.rs
package.json
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
```

**Structure Decision**: Keep the behavior and its focused unit test in the existing tray setup module. Update only the three synchronized version files required by project policy.

## Complexity Tracking

No constitution violations or additional complexity are required.
