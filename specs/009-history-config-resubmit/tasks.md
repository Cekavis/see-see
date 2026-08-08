# Tasks: History Configuration Resubmit

**Input**: Design documents from `/specs/009-history-config-resubmit/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Behavioral changes require focused frontend and Rust regression tests before implementation.

**Organization**: Tasks are grouped by user story and executed sequentially where they share the history submission path.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the additive persistence contract used by both stories.

- [x] T001 Add the history configuration identity migration in src-tauri/migrations/0004_history_configuration_ids.sql
- [x] T002 Register and test the migration version in src-tauri/src/database.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend shared history and configuration loading types before UI and submission behavior.

- [x] T003 Extend history persistence/detail identity fields and focused regressions in src-tauri/src/history.rs and src-tauri/tests/history_integration.rs
- [x] T004 Add direct prompt loading by identity in src-tauri/src/settings.rs
- [x] T005 Update frontend history detail and IPC types for nullable configuration identities in src/ipc.ts

**Checkpoint**: History rows can retain and expose stable configuration identities.

---

## Phase 3: User Story 1 - Choose Configurations for Resubmission (Priority: P1) 🎯 MVP

**Goal**: Let the user choose a model and prompt locally and submit the saved image without changing global active settings.

**Independent Test**: Choose a non-default pair in history detail, submit it, and verify the command receives those identities without any activation call.

### Tests for User Story 1

- [x] T006 [US1] Add failing selector and submission contract tests in src/views/History.test.tsx and src/ipc.test.ts
- [x] T007 [US1] Update the Rust command signature compile contract in src-tauri/tests/desktop_lifecycle.rs

### Implementation for User Story 1

- [x] T008 [US1] Accept explicit model/prompt identities and reuse the existing analysis runner in src-tauri/src/commands.rs
- [x] T009 [US1] Load selectors, keep local selection state, and submit chosen identities in src/views/History.tsx
- [x] T010 [US1] Add responsive selector layout using existing design tokens in src/styles.css

**Checkpoint**: Any available model/prompt pair can be submitted from history detail without global changes.

---

## Phase 4: User Story 2 - Default to Original Configuration Identities (Priority: P2)

**Goal**: Default to original stable identities and use their current values, with safe legacy/deletion fallbacks.

**Independent Test**: Change the current name/content of the original configurations, load the detail, and verify those IDs remain selected; verify legacy/unavailable IDs fall back predictably.

### Tests for User Story 2

- [x] T011 [US2] Add failing default, edited-name, legacy fallback, and empty-state tests in src/views/History.test.tsx
- [x] T012 [US2] Add persistence/backfill assertions for configuration identities in src-tauri/tests/history_integration.rs and src-tauri/src/database.rs

### Implementation for User Story 2

- [x] T013 [US2] Persist selected configuration identities for successful and failed analyses in src-tauri/src/analysis.rs
- [x] T014 [US2] Implement retained-ID, name, active, and first-option defaults with loading/error/disabled states in src/views/History.tsx

**Checkpoint**: Original configurations stay selected across edits and legacy/unavailable rows remain usable.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Synchronize release metadata and verify the complete user-facing change.

- [x] T015 Synchronize the minor version in package.json, package-lock.json, src-tauri/Cargo.toml, src-tauri/Cargo.lock, and src-tauri/tauri.conf.json
- [x] T016 Run focused tests, full tests, lint, formatting, frontend build, and cargo test using specs/009-history-config-resubmit/quickstart.md
- [x] T017 Run npm run tauri build and install the generated Windows bundle locally
- [x] T018 Perform human visual review at 1094×768, 780×800, and 540×800 and record evidence in specs/009-history-config-resubmit/validation.md
- [x] T019 Inspect final status/diff, commit the atomic feature, and push origin/master

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on the migration contract from Phase 1.
- **User Story 1 (Phase 3)**: Depends on foundational types and loaders.
- **User Story 2 (Phase 4)**: Builds on the selectors and explicit submission path from User Story 1.
- **Polish (Phase 5)**: Depends on both user stories.

### User Story Dependencies

- **User Story 1 (P1)**: Delivers explicit local selection and submission.
- **User Story 2 (P2)**: Depends on User Story 1's selectors, then adds stable defaults and fallbacks.

### Within Each User Story

- Add focused tests before implementation and confirm they fail for the missing behavior.
- Complete persistence and command behavior before wiring the final UI path.
- Complete automated checks before release build, installation, and visual review.

### Parallel Opportunities

- No implementation tasks are marked parallel because the smallest change shares the history contract across frontend and backend and is safer as one traced path.
- Independent validation commands may run concurrently after implementation when they do not write overlapping build artifacts.

---

## Implementation Strategy

### MVP First

1. Complete migration and shared identity fields.
2. Add explicit model/prompt selection and resubmission without global mutations.
3. Validate User Story 1 before adding fallback refinements.

### Incremental Delivery

1. Stable persistence contract.
2. Local selector and submission contract.
3. Original/legacy default selection behavior.
4. Full release validation and installation.

## Notes

- Reuse existing configuration lists and snapshot types; add no dependency.
- Historical snapshots remain display data, while stable IDs select current configuration values.
- Every task uses the repository's existing error and notification patterns.
