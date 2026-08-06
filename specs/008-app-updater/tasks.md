# Tasks: App Updater

**Input**: Design documents from `/specs/008-app-updater/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: The update state machine, user interaction, signing configuration, and release asset gate require focused automated checks.

**Organization**: Tasks are grouped into check, install, and publish stories; the existing About view and release workflow remain the only integration points.

## Phase 1: Setup

- [x] T001 Confirm the existing About view, settings feedback patterns, Tauri builder, capability file, and release workflow integration points in `src/views/SettingsShell.tsx`, `src/components/Notifications.tsx`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`, and `.github/workflows/release.yml`
- [x] T002 Add the official updater and process dependencies in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`

---

## Phase 2: Foundational

- [x] T003 Generate the encrypted updater signing key outside the repository, embed only its public key in `src-tauri/tauri.conf.json`, and configure updater/restart capabilities in `src-tauri/capabilities/default.json`
- [x] T004 Initialize updater and process plugins in `src-tauri/src/lib.rs` and configure the GitHub latest-release endpoint plus updater artifacts in `src-tauri/tauri.conf.json`

---

## Phase 3: User Story 1 - Check for Updates (Priority: P1) 🎯 MVP

**Goal**: Check manually from About and show current, available, notes, and recoverable failure states.

**Independent Test**: Mock current, available, and failed checks and verify version, status, notes, busy state, and retry behavior.

### Tests for User Story 1

- [x] T005 [US1] Add failing About-page tests for current, available with release notes, checking, and failed/retry states in `src/views/SettingsShell.test.tsx`

### Implementation for User Story 1

- [x] T006 [US1] Implement the manual update check state and plain-text release-note presentation in `src/views/SettingsShell.tsx`
- [x] T007 [US1] Reuse existing settings tokens for update status and notes at desktop and compact widths in `src/styles.css`

---

## Phase 4: User Story 2 - Install and Restart (Priority: P2)

**Goal**: Download and install the checked update with one action, report progress, block duplicates, and restart.

**Independent Test**: Mock download events, completion, restart, and failure; verify progress, disabled controls, retry, and restart calls.

### Tests for User Story 2

- [x] T008 [US2] Add failing install progress, duplicate-action, restart, and failure tests in `src/views/SettingsShell.test.tsx`

### Implementation for User Story 2

- [x] T009 [US2] Implement download/install progress, busy-state protection, failure recovery, and restart in `src/views/SettingsShell.tsx`

---

## Phase 5: User Story 3 - Publish Update-Ready Releases (Priority: P3)

**Goal**: Publish complete signed updater metadata and installers only after every target succeeds.

**Independent Test**: Parse the workflow and validate version/signing preconditions, updater signing environment, required assets, metadata platforms, and draft-to-published gate.

### Tests for User Story 3

- [x] T010 [US3] Add workflow/config assertions for the updater endpoint, public key, signing secrets, updater metadata, and platform gate in `scripts/verify-release-config.mjs` and `package.json`

### Implementation for User Story 3

- [x] T011 [US3] Extend `.github/workflows/release.yml` with updater signing secrets, updater metadata upload, `latest.json` platform validation, and draft-only failure behavior
- [x] T012 [US3] Record updater key handling, release tagging, secret setup, retry behavior, and signed local build requirements in `AGENTS.md`

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T013 Run focused updater/config tests, full frontend/Rust tests, lint, formatting, frontend build, and signed Windows Tauri bundle validation from `specs/008-app-updater/quickstart.md`
- [x] T014 Install the Windows bundle, visually review the About update states at normal and compact widths, and record the unavailable macOS manual verification gap in `specs/008-app-updater/quickstart.md`
- [x] T015 Inspect final status and diff, exclude signing material/build output, commit atomically, and push `master` to `origin/master`

---

## Dependencies & Execution Order

- T001 confirms the existing path before dependency and configuration changes.
- T002 precedes T003-T009 because the official APIs and plugin crates must exist.
- T003 and T004 establish signed updater configuration before UI implementation.
- T005 must fail before T006-T007; T008 must fail before T009.
- T010 must fail before T011-T012.
- T013-T015 depend on all user stories.

## Parallel Opportunities

- After T002, T005 and T010 affect independent files and can be prepared independently.
- T007 can proceed after the About markup from T006 is stable.
- Full frontend and Rust validation can run in parallel during T013.

## Implementation Strategy

1. Establish signed updater configuration with the minimum official dependencies.
2. Deliver manual check behavior in the existing About page.
3. Add one-click install, progress, and restart.
4. Make release publication contingent on signed updater metadata and every installer.
5. Validate, install locally, document platform gaps, then publish the repository changes.
