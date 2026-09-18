# Implementation Plan: WebDAV 配置同步

**Branch**: `018-webdav-config-sync` | **Date**: 2026-09-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/018-webdav-config-sync/spec.md`

## Summary

Add a WebDAV settings and synchronization group to the existing 常规 page. Persist URL, username, and remote root in SQLite, persist the password through the existing cross-platform credential store, and add typed Tauri commands for saving settings, uploading a versioned JSON snapshot, and atomically downloading/merging model and prompt metadata. The snapshot excludes shortcuts and all credentials.

## Technical Context

**Language/Version**: TypeScript 6 / React 19; Rust 2024

**Primary Dependencies**: Existing Tauri 2 IPC, `reqwest` 0.12, `rusqlite`, `keyring`, `serde`/`serde_json`, shared `Field` and `Button` components

**Storage**: SQLite `app_settings` row for non-secret WebDAV fields; OS credential store for WebDAV password; remote versioned JSON file for model/prompt metadata

**Testing**: Vitest frontend tests, Rust unit/integration tests with in-memory SQLite and `wiremock`, plus typecheck/lint/format/build checks

**Target Platform**: Windows and macOS desktop builds supported by the existing Tauri application

**Project Type**: Desktop app

**Performance Goals**: A normal configuration snapshot should upload or download within 10 seconds on a healthy WebDAV connection; no blocking UI work during network calls

**Constraints**: HTTPS except loopback HTTP; no credentials in the remote snapshot; atomic download merge; reuse existing UI and error patterns; no new dependency unless required

**Scale/Scope**: One saved WebDAV connection and one snapshot file per app installation; up to hundreds of model/prompt entries per snapshot

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Maintainability**: Pass. Reuses existing settings, credential, database, HTTP, IPC, and UI abstractions. A small `webdav` module owns protocol details.
- **Testing**: Pass. Add focused Rust tests for validation, snapshot redaction, WebDAV status handling, and atomic merge; add frontend tests for persistence, loading, success, error, and disabled states.
- **User experience**: Pass. The new group follows `DesktopSettings` loading and notification patterns and defines all required operation states.
- **UI quality**: Pass. Reuse `Field`, `Button`, `settings-group`, `setting-row`, and existing responsive styles; perform desktop and narrow-width visual review.

## Project Structure

### Documentation

```text
specs/018-webdav-config-sync/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/ipc.md
└── tasks.md
```

### Source Code

```text
src/
├── ipc.ts
└── views/
    ├── DesktopSettings.tsx
    └── Settings.desktop.test.tsx

src-tauri/
├── migrations/0010_webdav_settings.sql
├── src/
│   ├── commands.rs
│   ├── database.rs
│   ├── lib.rs
│   ├── settings.rs
│   ├── state.rs
│   └── webdav.rs
└── tests/
    └── webdav_sync.rs
```

**Structure Decision**: Keep WebDAV transport and snapshot logic in a dedicated Rust module, keep SQLite model/prompt persistence helpers in `settings.rs`, expose only typed IPC functions through `commands.rs` and `ipc.ts`, and render the feature inside the existing 常规 page component.

## Complexity Tracking

No constitution violations. The dedicated WebDAV module is the smallest boundary that keeps HTTP protocol code out of settings persistence and commands while allowing focused tests.
