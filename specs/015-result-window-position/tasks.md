# Tasks: Remember Result Window Position

**Input**: Design documents from `/specs/015-result-window-position/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md)

**Tests**: Behavioral change tasks include focused Rust state and desktop lifecycle regression coverage.

**Organization**: Tasks are grouped by the single independently testable user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No project or dependency setup is required; the feature reuses the existing Tauri runtime and test structure.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the session-scoped value that the event and window-presentation paths share.

- [X] T001 Add `ResultWindowPosition` and the optional latest-position field/update helper to `src-tauri/src/state.rs`

**Checkpoint**: The runtime can represent an empty position and replace it with the latest complete coordinate pair.

## Phase 3: User Story 1 - Reuse the latest result window position (Priority: P1) 🎯 MVP

**Goal**: After any result window is moved, each subsequently created result window opens at the latest moved location while the first-open fallback and existing window behavior remain unchanged.

**Independent Test**: Run the focused Rust tests, then manually open, move, and create result windows; verify the new window reuses the latest location without moving existing windows.

### Tests for User Story 1 ⚠️

> Write these tests before the corresponding behavior implementation and confirm they fail for the missing behavior.

- [X] T002 [P] [US1] Add runtime-state regression tests for empty, latest-wins, and close-does-not-clear position behavior in `src-tauri/src/state.rs`
- [X] T003 [P] [US1] Add desktop lifecycle source-contract tests for result-only move tracking, remembered-position placement, center fallback, and show/focus ordering in `src-tauri/tests/desktop_lifecycle.rs`

### Implementation for User Story 1

- [X] T004 [US1] Handle `Moved` events only for `result-*` windows and update the shared position in `src-tauri/src/lib.rs`
- [X] T005 [US1] Read the remembered position during result-window creation and pass it to presentation in `src-tauri/src/commands.rs`
- [X] T006 [US1] Apply the remembered physical position before showing/focusing result windows while retaining centered fallback and platform policies in `src-tauri/src/windowing.rs`

**Checkpoint**: User Story 1 is independently functional and covered by automated regressions.

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Keep release metadata synchronized and complete proportionate validation.

- [X] T007 Update the synchronized application version from `0.12.1` to `0.12.2` in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`
- [ ] T008 Run formatting, lint, frontend build, focused Rust tests, full Rust tests, and `npm run tauri build`; record desktop visual review for `specs/015-result-window-position/quickstart.md`

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; no setup edits are required.
- **Foundational (Phase 2)**: Starts immediately and blocks user-story work.
- **User Story 1 (Phase 3)**: Depends on T001; write T002 and T003 before T004–T006.
- **Polish (Phase 4)**: Depends on the completed implementation and passing focused tests.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on another user story.

### Parallel Opportunities

- T002 and T003 can run in parallel after T001 because they touch separate test sections/files and do not depend on each other's results.
- T004 can be implemented independently of the presentation edits in T006 after T001, but T005 and T006 should be completed as one ordered window-creation/presentation change because they share the updated function contract.

## Parallel Example: User Story 1

```text
Task: T002 Add runtime-state regression tests in src-tauri/src/state.rs
Task: T003 Add desktop lifecycle source-contract tests in src-tauri/tests/desktop_lifecycle.rs
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001.
2. Add T002 and T003 and verify the missing behavior is exposed.
3. Complete T004–T006.
4. Run the focused result-window tests and perform the manual move-and-create scenario.

### Incremental Delivery

1. The feature is a single user story, so no later story is required for value.
2. Complete T007 and T008 before release handoff.

## Notes

- The position is session-scoped and intentionally not persisted in SQLite.
- Negative coordinates are valid for displays arranged above or left of the primary display.
- Existing result-window size, always-on-top, focus, close, streaming, retry, and navigation behavior must remain unchanged.
