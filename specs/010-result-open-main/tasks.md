# Tasks: Result Window Main Navigation

**Input**: Design documents from `/specs/010-result-open-main/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Behavioral changes require focused frontend and Rust regression tests before implementation.

**Organization**: One user story owns the complete result-to-main navigation path.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the existing project and window lifecycle patterns needed by the feature.

- [x] T001 Verify ignore coverage and trace the result footer, IPC registry, terminal-state predicate, main-window focus helper, and result close cleanup in .gitignore, eslint.config.js, .prettierignore, src/views/Result.tsx, src/ipc.ts, src/App.tsx, src-tauri/src/commands.rs, src-tauri/src/lib.rs, and src-tauri/src/state.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add failing contracts for the requested action and state boundary.

- [x] T002 Add failing footer callback and IPC argument tests in src/views/Result.test.tsx and src/ipc.test.ts
- [x] T003 Add terminal-state boundary and result-navigation command contract regressions in src-tauri/src/state.rs and src-tauri/tests/desktop_lifecycle.rs

**Checkpoint**: Tests describe the button action and active-versus-terminal close boundary.

---

## Phase 3: User Story 1 - Open the Main Window from Results (Priority: P1) 🎯 MVP

**Goal**: Open and focus the main window from every result state, retaining active results and closing terminal results.

**Independent Test**: Activate the footer action during streaming and after a terminal outcome; verify the main window opens in both cases and only the terminal result closes.

### Implementation for User Story 1

- [x] T004 [US1] Add the always-available shared button and error handling in src/views/Result.tsx
- [x] T005 [US1] Wire the result run identity through the frontend IPC contract in src/App.tsx and src/ipc.ts
- [x] T006 [US1] Implement the authoritative show/focus and conditional result close command in src-tauri/src/commands.rs and register it in src-tauri/src/lib.rs

**Checkpoint**: The complete user story works without adding a dependency or duplicating cleanup state.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Synchronize release metadata and verify the complete user-facing change.

- [x] T007 Synchronize version 0.9.0 in package.json, package-lock.json, src-tauri/Cargo.toml, src-tauri/Cargo.lock, and src-tauri/tauri.conf.json
- [x] T008 Run focused tests, full tests, lint, formatting, frontend build, and cargo test using specs/010-result-open-main/quickstart.md
- [x] T009 Run npm run tauri build and install the generated Windows bundle locally
- [x] T010 Perform human visual review at the normal result-window size and 420×360 minimum and record evidence in specs/010-result-open-main/validation.md
- [x] T011 Inspect final status/diff, mark all tasks complete, commit the atomic feature, and push origin/master

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on the traced execution path from Phase 1.
- **User Story 1 (Phase 3)**: Depends on failing behavioral contracts from Phase 2.
- **Polish (Phase 4)**: Depends on the complete user story.

### User Story Dependencies

- **User Story 1 (P1)**: Independent; it is the complete requested feature and MVP.

### Within User Story 1

- Add focused tests before implementation and confirm they fail for the missing behavior.
- Add the UI callback contract before wiring IPC and backend behavior.
- Use the backend state at action time before closing a result window.
- Run automated checks before release build, installation, and visual review.

### Parallel Opportunities

- No implementation tasks are marked parallel because the smallest change follows one UI-to-window lifecycle path and is safer as a single traced edit.
- Independent validation commands may run concurrently after implementation when they do not write overlapping build artifacts.

---

## Implementation Strategy

### MVP First

1. Add the footer action and focused tests.
2. Route it through one IPC command.
3. Reuse terminal state and close lifecycle behavior.
4. Validate active and terminal flows independently.

## Notes

- Reuse existing Button, notification, main-window focus, and result-close patterns.
- Do not add frontend window permissions, new state abstractions, or dependencies.
- Every task uses exact repository paths and the existing error contract.
