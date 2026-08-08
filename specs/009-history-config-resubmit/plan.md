# Implementation Plan: History Configuration Resubmit

**Branch**: `master` | **Date**: 2026-08-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/009-history-config-resubmit/spec.md`

## Summary

Extend saved history entries with nullable model and prompt configuration identities while retaining their existing display snapshots. The history detail loads existing configuration lists, defaults each selector to the retained identity (with name/active/first fallbacks), and passes the locally selected identities to the resubmission command. The backend resolves those identities to the latest configuration values without updating global active settings.

## Technical Context

**Language/Version**: TypeScript 6, React 19; Rust 2024 edition; SQL migrations

**Primary Dependencies**: Existing React hooks, Tauri 2 IPC, rusqlite, Testing Library, Vitest; no new dependency

**Storage**: Existing SQLite `history_entries`, `model_configs`, and `prompt_presets` tables

**Testing**: Vitest 4 with Testing Library; Rust unit/integration tests with `cargo test`; lint, formatting, frontend build, Tauri release build, and local Windows installation

**Target Platform**: Tauri 2 desktop application on Windows and macOS

**Project Type**: Desktop application with React/TypeScript UI and Rust backend

**Performance Goals**: Load configuration choices in one parallel request group and start resubmission without additional persistence writes beyond the normal analysis/history flow

**Constraints**: Preserve historical display snapshots; avoid global configuration mutations; support existing databases and legacy history rows; add no dependency or new settings abstraction

**Scale/Scope**: One history-detail workflow, one IPC command contract, one additive database migration, and focused frontend/Rust regressions

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **Maintainability**: Pass. Reuses configuration list/load functions, existing history persistence, shared form/button styles, and the current analysis runner. The only schema additions are two nullable identity columns.
- **Testing**: Pass. Frontend tests cover defaults, local selection, submission arguments, loading/empty states, and unchanged history behavior. Rust tests cover identity persistence and migration availability.
- **User experience**: Pass. Existing history navigation and notification patterns remain. Selectors expose loading, empty, disabled, success, error, and fallback states.
- **UI quality**: Pass. Native selects, existing tokens, and the shared Button are reused. Keyboard labels, responsive layout, and representative visual review are included.

Post-design re-check: all gates remain passed. No constitution exception, new dependency, or speculative abstraction is required.

## Project Structure

### Documentation (this feature)

```text
specs/009-history-config-resubmit/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── history-resubmit.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── ipc.ts
├── styles.css
└── views/
    ├── History.tsx
    └── History.test.tsx

src-tauri/
├── migrations/
│   └── 0004_history_configuration_ids.sql
├── src/
│   ├── analysis.rs
│   ├── commands.rs
│   ├── database.rs
│   ├── history.rs
│   └── settings.rs
└── tests/
    ├── desktop_lifecycle.rs
    ├── history_integration.rs
    └── storage_foundation.rs

package.json
package-lock.json
src-tauri/Cargo.toml
src-tauri/Cargo.lock
src-tauri/tauri.conf.json
```

**Structure Decision**: Keep selection state in the existing History view, extend the existing IPC command, and route explicit configurations through the existing analysis input. Store stable references beside the current snapshots so display history and current-value execution remain separate concerns.

## Complexity Tracking

No constitution violations.
