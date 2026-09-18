---
description: "Task list for WebDAV configuration synchronization"
---

# Tasks: WebDAV 配置同步

**Input**: Design documents from `/specs/018-webdav-config-sync/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ipc.md

**Tests**: Behavioral changes include focused frontend and Rust regression tests.

## Phase 1: Setup

- [x] T001 Add the WebDAV feature artifacts and confirm the existing database migration/version pattern in `src-tauri/src/database.rs`.

## Phase 2: Foundational

- [x] T002 [P] Add `webdav_url`, `webdav_username`, and `webdav_remote_root` migration with the `see-see` default in `src-tauri/migrations/0010_webdav_settings.sql` and register it in `src-tauri/src/database.rs`.
- [x] T003 [P] Retain the existing credential store on `AppState` and expose WebDAV password load/save helpers in `src-tauri/src/state.rs` and `src-tauri/src/settings.rs`.
- [x] T004 [P] Extend `ErrorCode` or existing error mapping as needed for actionable WebDAV validation, authentication, missing-file, and transport failures in `src-tauri/src/error.rs` and `src-tauri/src/webdav.rs`.

## Phase 3: User Story 1 - 保存 WebDAV 连接设置 (Priority: P1)

**Goal**: Save and reload non-secret WebDAV fields while keeping the password in the OS credential store.

### Tests

- [x] T005 [P] [US1] Add Rust tests for default root, endpoint/path validation, password retention, and clear-password behavior in `src-tauri/src/webdav.rs`, `src-tauri/tests/webdav_configuration.rs`, and `src-tauri/tests/native_credentials.rs`.
- [x] T006 [P] [US1] Add frontend tests for initial loading, saved-password state, save success, validation/error recovery, and narrow-safe form controls in `src/views/WebdavSettings.test.tsx`.

### Implementation

- [x] T007 [US1] Define `WebdavSettings` and `WebdavSettingsInput` Rust types plus database load/save functions in `src-tauri/src/settings.rs`.
- [x] T008 [US1] Implement WebDAV settings commands in `src-tauri/src/commands.rs`, register them in `src-tauri/src/lib.rs`, and add typed wrappers in `src/ipc.ts`.
- [x] T009 [US1] Add the WebDAV settings form, password retention/clear behavior, and save feedback to `src/views/WebdavSettings.tsx` using existing `Field`, `Button`, and notification patterns.

## Phase 4: User Story 2 - 上传模型与提示词配置 (Priority: P1)

**Goal**: Upload a versioned snapshot including model API Keys to an application-owned remote file.

### Tests

- [x] T010 [P] [US2] Add Rust snapshot serialization tests proving model API Keys round-trip while shortcuts and WebDAV credentials are absent in `src-tauri/tests/webdav_configuration.rs`.
- [x] T011 [P] [US2] Add WebDAV transport tests for MKCOL/PUT, Basic auth, remote-root/file URL construction, and HTTP/transport failures using `wiremock` in `src-tauri/tests/webdav_sync.rs`.
- [x] T012 [P] [US2] Add frontend tests for upload button disabled/loading states, success counts, and recoverable errors in `src/views/WebdavSettings.test.tsx`.

### Implementation

- [x] T013 [US2] Implement the versioned sync snapshot structs and export logic in `src-tauri/src/settings.rs`, including model API Keys and excluding shortcuts and WebDAV credentials while retaining model/prompt metadata and active model selection.
- [x] T014 [US2] Implement WebDAV endpoint validation, root creation, authenticated PUT, status/error mapping, and size/time bounds in `src-tauri/src/webdav.rs`.
- [x] T015 [US2] Add the async upload command, register it in `src-tauri/src/lib.rs`, and expose the IPC result type in `src/ipc.ts`.
- [x] T016 [US2] Add the upload action and result notifications to `src/views/WebdavSettings.tsx`.

## Phase 5: User Story 3 - 下载并合并配置 (Priority: P1)

**Goal**: Download, validate, and atomically merge remote models (including API Keys) and prompts while preserving local shortcuts and local-only entries.

### Tests

- [x] T017 [P] [US3] Add Rust tests for ID/name matching, prompt shortcut preservation, new-prompt shortcut absence, local-only retention, invalid payload rejection, and atomic rollback in `src-tauri/tests/webdav_configuration.rs`.
- [x] T018 [P] [US3] Add frontend tests for download loading/disabled states, success counts, and failure recovery in `src/views/WebdavSettings.test.tsx`.

### Implementation

- [x] T019 [US3] Implement full snapshot validation and transaction-safe model/prompt merge helpers in `src-tauri/src/settings.rs`.
- [x] T020 [US3] Implement authenticated GET and JSON size/version/shape handling in `src-tauri/src/webdav.rs`.
- [x] T021 [US3] Add the async download command, register it in `src-tauri/src/lib.rs`, and expose the IPC wrapper in `src/ipc.ts`.
- [x] T022 [US3] Add the download action, success counts, error handling, and refresh behavior to `src/views/WebdavSettings.tsx`.

## Phase 6: Polish and validation

- [x] T023 [P] Update the relevant frontend and Rust tests for the expanded IPC/settings types and verify model API Keys are imported while shortcut and WebDAV credential fields remain untouched.
- [ ] T024 [P] Perform human visual review of the 常规 page at representative desktop and narrow widths. Browser visual checks and screenshots are complete; human acceptance remains pending as recorded in `specs/018-webdav-config-sync/validation.md`.
- [x] T025 Run typecheck, lint, formatting, Vitest, Rust tests, and the Windows Tauri build/install checks; resolve regressions and record results and external acceptance gaps in `specs/018-webdav-config-sync/validation.md`.
- [x] T026 Bump synchronized application versions for this behavioral feature, inspect status/diff, commit atomically, and push the completed change to `origin/master` per repository instructions.

## Dependencies and order

- Phase 2 blocks all user stories.
- US1 must land before upload/download actions can be exercised because both use saved WebDAV settings.
- US2 and US3 share the snapshot model and transport module; implement/export tests before the commands.
- Polish and validation follow all behavior changes.
