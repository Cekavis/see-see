# Implementation Plan: History Detail Navigation

**Branch**: `master` | **Date**: 2026-07-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-history-detail-navigation/spec.md`

## Summary

Replace the responsive two-column history list/detail layout with mutually exclusive list and detail views. Keep browsing state in the mounted History component, capture the settings content scroll offset before opening a record, and restore it after returning. Preserve summary whitespace with presentation styling and reduce the desktop sidebar width using the existing responsive grid.

## Technical Context

**Language/Version**: TypeScript 6, React 19; CSS; Rust 2024 edition remains unchanged

**Primary Dependencies**: Existing React hooks, Testing Library, Vitest, and shared See See UI components; no new dependency

**Storage**: Existing SQLite history storage remains unchanged

**Testing**: Vitest 4 with Testing Library; existing lint, formatting, frontend build, Rust tests, Tauri release build, and manual local app review

**Target Platform**: Tauri 2 desktop application on macOS and Windows

**Project Type**: Desktop application with React/TypeScript UI and Rust backend

**Performance Goals**: Return to the prior list state within one rendered frame and without issuing another list query

**Constraints**: Preserve current IPC contracts and history data; no router or new navigation dependency; maintain responsive navigation at widths of 780 px and below

**Scale/Scope**: One history screen, its focused regression tests, and one shared desktop sidebar grid rule

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **Maintainability**: Pass. The design uses existing component state, shared buttons, and the settings scroll container. It adds no dependency or application-wide router.
- **Testing**: Pass. Focused component regressions cover exclusive detail navigation, state/query preservation, scroll restoration, whitespace rendering, and existing actions.
- **User experience**: Pass. Existing list loading, empty, pagination, notification, confirmation, and detail action patterns remain; detail failure leaves the recoverable list visible.
- **UI quality**: Pass. Existing design tokens and components are reused. Keyboard naming, wrapping, desktop/narrow viewports, and human visual review are included in validation.

Post-design re-check: all gates remain passed. No constitution exceptions or added complexity require justification.

## Project Structure

### Documentation (this feature)

```text
specs/005-history-detail-navigation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── history-navigation.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── styles.css
└── views/
    ├── History.tsx
    └── History.test.tsx

package.json
package-lock.json
src-tauri/
├── Cargo.toml
└── tauri.conf.json
```

**Structure Decision**: Keep the feature within the existing `History` view and shared stylesheet. No backend, IPC, persistence, or routing files need structural changes.

## Complexity Tracking

No constitution violations.
