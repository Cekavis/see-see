---

description: "Task list for WebDAV configuration synchronization"
---

# Tasks: WebDAV 配置同步

**Input**: Design documents from `/specs/018-webdav-config-sync/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ipc.md

**Tests**: Behavioral changes include focused frontend and Rust regression tests.

## Phase 1: Setup

- [ ] T001 Add the WebDAV feature artifacts and confirm the existing database migration/version pattern in `src-tauri/src/database.rs`.

## Phase 2: Foundational

- [ ] T002 [P] Add `webdav_url`, `webdav_username`, and `webdav_remote_root` migration with the `see-see` default in `src-tauri/migrations/0010_webdav_settings.sql` and register it in `src-tauri/src/database.rs`.
- [ ] T003 [P] Retain the existing credential store on `AppState` and expose WebDAV password load/save helpers in `src-tauri/src/state.rs` and `src-tauri/src/settings.rs`.
- [ ] T004 [P] Extend `ErrorCode` or existing error mapping as needed for actionable WebDAV validation, authentication, missing-file, and transport failures in `src-tauri/src/error.rs` and `src-tauri/src/webdav.rs`.

## Phase 3: User Story 1 - 保存 WebDAV 连接设置 (Priority: P1)

**Goal**: Save and reload non-secret WebDAV fields while keeping the password in the OS credential store.

### Tests

- [ ] T005 [P] [US1] Add Rust tests for default root, endpoint/path validation, password retention, and clear-password behavior in `src-tauri/src/webdav.rs` or `src-tauri/tests/webdav_sync.rs`.
- [ ] T006 [P] [US1] Add frontend tests for initial loading, saved-password state, save success, validation/error recovery, and narrow-safe form controls in `src/views/Settings.desktop.test.tsx`.

### Implementation

- [ ] T007 [US1] Define `WebdavSettings` and `WebdavSettingsInput` Rust types plus database load/save functions in `src-tauri/src/settings.rs`.
- [ ] T008 [US1] Implement WebDAV settings commands in `src-tauri/src/commands.rs`, register them in `src-tauri/src/lib.rs`, and add typed wrappers in `src/ipc.ts`.
- [ ] T009 [US1] Add the WebDAV settings form, password retention/clear behavior, and save feedback to `src/views/DesktopSettings.tsx` using existing `Field`, `Button`, and notification patterns.

## Phase 4: User Story 2 - 上传模型与提示词配置 (Priority: P1)

**Goal**: Upload a versioned, credential-free snapshot to an application-owned remote file.

### Tests

- [ ] T010 [P] [US2] Add Rust snapshot serialization tests proving shortcuts, API Keys, WebDAV passwords, and other credentials are absent in `src-tauri/tests/webdav_sync.rs`.
- [ ] T011 [P] [US2] Add WebDAV transport tests for MKCOL/PUT, Basic auth, remote-root/file URL construction, and HTTP/transport failures using `wiremock` in `src-tauri/tests/webdav_sync.rs`.
- [ ] T012 [P] [US2] Add frontend tests for upload button disabled/loading states, success counts, and recoverable errors in `src/views/Settings.desktop.test.tsx`.

### Implementation

- [ ] T013 [US2] Implement the versioned sync snapshot structs and export logic in `src-tauri/src/settings.rs`, excluding shortcuts and secrets while retaining model/prompt metadata and active model selection.
- [ ] T014 [US2] Implement WebDAV endpoint validation, root creation, authenticated PUT, status/error mapping, and size/time bounds in `src-tauri/src/webdav.rs`.
- [ ] T015 [US2] Add the async upload command, register it in `src-tauri/src/lib.rs`, and expose the IPC result type in `src/ipc.ts`.
- [ ] T016 [US2] Add the upload action and result notifications to `src/views/DesktopSettings.tsx`.

## Phase 5: User Story 3 - 下载并合并配置 (Priority: P1)

**Goal**: Download, validate, and atomically merge remote model/prompt metadata while preserving local shortcuts and local-only entries.

### Tests

- [ ] T017 [P] [US3] Add Rust tests for ID/name matching, prompt shortcut preservation, new-prompt shortcut absence, local-only retention, invalid payload rejection, and atomic rollback in `src-tauri/tests/webdav_sync.rs`.
- [ ] T018 [P] [US3] Add frontend tests for download loading/disabled states, success counts, and failure recovery in `src/views/Settings.desktop.test.tsx`.

### Implementation

- [ ] T019 [US3] Implement full snapshot validation and transaction-safe model/prompt merge helpers in `src-tauri/src/settings.rs`.
- [ ] T020 [US3] Implement authenticated GET and JSON size/version/shape handling in `src-tauri/src/webdav.rs`.
- [ ] T021 [US3] Add the async download command, register it in `src-tauri/src/lib.rs`, and expose the IPC wrapper in `src/ipc.ts`.
- [ ] T022 [US3] Add the download action, success counts, error handling, and refresh behavior to `src/views/DesktopSettings.tsx`.

## Phase 6: Polish and validation

- [ ] T023 [P] Update the relevant frontend and Rust tests for the expanded IPC/settings types and verify no shortcut or credential fields are written by import.
- [ ] T024 [P] Perform human visual review of the 常规 page at representative desktop and narrow widths; record evidence in `specs/018-webdav-config-sync/quickstart.md`.
- [ ] T025 Run typecheck, lint, formatting, Vitest, Rust tests, and the relevant Tauri build/install checks; resolve regressions and record results.
- [ ] T026 Bump synchronized application versions for this behavioral feature, inspect status/diff, commit atomically, and push the completed change to `origin/master` per repository instructions.

## Dependencies and order

- Phase 2 blocks all user stories.
- US1 must land before upload/download actions can be exercised because both use saved WebDAV settings.
- US2 and US3 share the snapshot model and transport module; implement/export tests before the commands.
- Polish and validation follow all behavior changes.
