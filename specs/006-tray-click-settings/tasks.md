# Tasks: Tray Click Settings

**Input**: Design documents from `/specs/006-tray-click-settings/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This behavioral change requires a focused Rust regression test plus existing project validation.

**Organization**: The single user story is independently testable and is the MVP.

## Phase 1: Setup

**Purpose**: Confirm the existing tray and settings-window path used by the change.

- [x] T001 Confirm tray construction and `show_main_window` remain centralized in `src-tauri/src/lib.rs`

---

## Phase 2: Foundational

**Purpose**: No new infrastructure is required; the installed Tauri tray API and existing window helper are reused.

- [x] T002 Confirm `show_menu_on_left_click` and tray click events are available through the locked Tauri dependency in `src-tauri/Cargo.lock`

---

## Phase 3: User Story 1 - Open Settings from Tray (Priority: P1) 🎯 MVP

**Goal**: Left-click opens settings directly; right-click alone displays the existing tray menu.

**Independent Test**: Hide the settings window, left-click the tray icon, and verify the existing window opens without a menu; right-click and verify the existing menu appears.

### Tests for User Story 1

- [x] T003 [US1] Add a focused regression test for left-button release filtering in `src-tauri/src/lib.rs`

### Implementation for User Story 1

- [x] T004 [US1] Disable primary-click menu display and open settings on left-button release in `src-tauri/src/lib.rs`
- [x] T005 [US1] Bump the synchronized compatible-feature version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`

**Checkpoint**: User Story 1 is implemented and covered by a focused automated test.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [x] T006 Run Rust tests, frontend tests, lint, formatting, frontend build, and the release bundle commands listed in `specs/006-tray-click-settings/quickstart.md`
- [ ] T007 Install the Windows release bundle and manually verify left-click and right-click tray behavior using `specs/006-tray-click-settings/quickstart.md`
- [x] T008 Record the macOS manual verification gap in `specs/006-tray-click-settings/quickstart.md` if macOS is unavailable

---

## Dependencies & Execution Order

- T001 and T002 confirm prerequisites before implementation.
- T003 must be added before T004 and must fail without the new predicate behavior.
- T004 depends on T003.
- T005 follows the behavior change.
- T006 through T008 follow implementation.

## Parallel Opportunities

None. The small change is concentrated in one Rust module and sequential verification avoids coordination overhead.

## Implementation Strategy

Complete the single P1 story, run all required validation, install the Windows bundle, and record the unavailable macOS check rather than adding test infrastructure for one platform interaction.
