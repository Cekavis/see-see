# Implementation Plan: WebDAV 配置同步

**Branch**: `master` | **Date**: 2026-09-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/018-webdav-config-sync/spec.md`

## Summary

Add a WebDAV settings and synchronization group to the existing 常规 page. Persist URL, username, and remote root in SQLite, persist the password through the existing cross-platform credential store with native Windows/macOS backends enabled, and add typed Tauri commands for saving settings, uploading a versioned JSON snapshot, and atomically downloading/merging models (including API Keys) and prompts. The snapshot excludes shortcuts and WebDAV login information.

## Technical Context

**Language/Version**: TypeScript 6 / React 19; Rust 2024

**Primary Dependencies**: Existing Tauri 2 IPC, `reqwest` 0.12, `rusqlite`, `keyring`, `serde`/`serde_json`, shared `Field` and `Button` components

**Storage**: SQLite `app_settings` row for non-secret WebDAV fields; OS credential store for WebDAV password; remote versioned JSON file for models including API Keys and prompt content

**Testing**: Vitest frontend tests, Rust unit/integration tests with in-memory SQLite and `wiremock`, plus typecheck/lint/format/build checks

**Target Platform**: Windows and macOS desktop builds supported by the existing Tauri application

**Project Type**: Desktop app

**Performance Goals**: A normal configuration snapshot should upload or download within 10 seconds on a healthy WebDAV connection; no blocking UI work during network calls

**Constraints**: HTTPS except loopback HTTP; only model API Keys participate in sync; atomic download merge; no secret logging or echo; reuse existing UI and error patterns; native keyring feature activation is required for persistence

**Scale/Scope**: One saved WebDAV connection and one snapshot file per app installation; up to hundreds of model/prompt entries per snapshot

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **Maintainability**: Pass. Reuses existing settings, credential, database, HTTP, IPC, and UI abstractions. A small `webdav` module owns protocol details.
- **Testing**: Pass. Add focused Rust tests for validation, API Key round-trips and shortcut exclusion, WebDAV status handling, and atomic merge; add frontend tests for persistence, loading, success, error, and disabled states.
- **User experience**: Pass. The new group follows `DesktopSettings` loading and notification patterns and defines all required operation states.
- **UI quality**: Existing `Field`, `Button`, `settings-grid`, and responsive styles are reused. Browser checks at desktop and minimum window sizes are complete; human visual acceptance remains pending.

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
    ├── SettingsShell.tsx
    ├── WebdavSettings.tsx
    └── WebdavSettings.test.tsx

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
    ├── webdav_sync.rs
    ├── webdav_configuration.rs
    └── native_credentials.rs
```

**Structure Decision**: Keep WebDAV transport in its own Rust module and snapshot persistence in `settings.rs`. Expose typed IPC commands and compose a dedicated `WebdavSettings` component in the existing 常规 page. Desktop settings and shortcut helpers keep their existing responsibilities.

## Complexity Tracking

No constitution violations. The dedicated WebDAV module is the smallest boundary that keeps HTTP protocol code out of settings persistence and commands while allowing focused tests.
