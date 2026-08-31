# Tasks: Concurrent Analysis Requests

**Input**: Design documents from `/specs/013-concurrent-analysis-requests/`

## Phase 1: Setup

- [X] T001 Review existing run-ID window labels and analysis lifecycle in `src-tauri/src/commands.rs` and `src/App.tsx`

## Phase 2: Foundational

- [X] T002 [P] Define run-ID keyed runtime ownership in `src-tauri/src/state.rs`
- [X] T003 [P] Define original request snapshot storage and retry input in `src-tauri/src/analysis.rs`

## Phase 3: User Story 1 - Run multiple analyses concurrently (Priority: P1)

**Goal**: Permit multiple unfinished analyses and keep their result windows/events independent.

**Independent Test**: Two `ActiveAnalysis` values and two runtime map entries coexist; interleaved events preserve each run ID and text.

- [X] T004 [P] [US1] Add concurrent stream isolation regression coverage in `src-tauri/tests/analysis_flow.rs`
- [X] T005 [US1] Remove analysis singleton guards and insert runs by ID in `src-tauri/src/commands.rs`
- [X] T006 [US1] Cancel all retained runs during application exit in `src-tauri/src/commands.rs`
- [X] T007 [US1] Keep result-window close cleanup exact to the requested ID in `src-tauri/src/lib.rs`

## Phase 4: User Story 2 - Retry the failed request with its original configuration (Priority: P1)

**Goal**: Retry a failed run with its own immutable request snapshot.

**Independent Test**: A failed run returns the original model/prompt/endpoint/key/image/history values after active settings change.

- [X] T008 [P] [US2] Add original snapshot retry regression coverage in `src-tauri/tests/analysis_flow.rs`
- [X] T009 [US2] Use stored snapshot input in `retry_analysis` in `src-tauri/src/commands.rs`
- [X] T010 [US2] Preserve same run identity and reset terminal state in `src-tauri/src/analysis.rs`

## Phase 5: User Story 3 - Manage each result window independently (Priority: P2)

**Goal**: Prevent attach/navigation actions from displaying or mutating another run.

**Independent Test**: An attach snapshot with a mismatched run ID is ignored and live updates are not overwritten by a stale attach response.

- [X] T011 [US3] Guard attached frontend snapshots by URL run ID in `src/App.tsx`
- [X] T012 [P] [US3] Add lifecycle contract checks for map insertion and retry source in `src-tauri/tests/desktop_lifecycle.rs`

## Phase 6: Polish & Validation

- [X] T013 [P] Update feature specification, plan, research, data model, quickstart, and runtime contract under `specs/013-concurrent-analysis-requests/`
- [X] T014 Bump synchronized application version files from `0.11.4` to `0.11.5`
- [X] T015 Run formatting, frontend tests/build, Rust tests, and final diff review

## Dependencies & Execution Order

- T001 precedes T002-T003.
- T002-T003 precede T004-T012.
- T004/T008/T012 are regression evidence for T005/T009/T011.
- T013-T015 follow implementation and tests.

## MVP Scope

User Story 1 and User Story 2 are the MVP: concurrent runs plus deterministic retries. User Story 3 completes lifecycle hardening.
