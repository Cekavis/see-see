# Tasks: Result Window Size Memory

**Input**: Design documents from `specs/017-result-window-size-memory/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/window-size-preference.md`, `quickstart.md`

**Tests**: Behavioral changes use focused Rust storage/lifecycle tests and Vitest style assertions. User-facing behavior also receives representative manual visual review.

## Phase 1: Setup

**Purpose**: Establish regression coverage for durable result-window geometry and the compact preview limit.

- [x] T001 [P] [US1] Add persistence-after-reopen and legacy-schema migration assertions in `src-tauri/tests/storage_foundation.rs` and `src-tauri/src/database.rs`
- [x] T002 [P] [US1] Add result-window default, resize, preference application, close, and quit source-contract assertions in `src-tauri/tests/desktop_lifecycle.rs`
- [x] T003 [P] [US2] Update screenshot-preview style regression assertions in `src/views/Result.test.tsx` and `src/views/History.test.tsx`

## Phase 2: Foundational

**Purpose**: Add a durable, backward-compatible application-settings representation for result-window dimensions.

- [x] T004 Add nullable result-window dimension fields and the upgrade migration in `src-tauri/migrations/0001_init.sql`, `src-tauri/migrations/0009_result_window_size.sql`, and `src-tauri/src/database.rs`
- [x] T005 Add result-window dimension state, load, and save helpers in `src-tauri/src/state.rs` and `src-tauri/src/settings.rs`

**Checkpoint**: A valid preference can be loaded from a new or upgraded database and remains available in application runtime state.

## Phase 3: User Story 1 - Reuse a preferred result-window size (Priority: P1) 🎯 MVP

**Goal**: Use the 460×540 default until a user chooses another valid size, then reuse it in-session and after normal restart.

**Independent Test**: Persist a chosen dimension pair, reopen the database, construct a new result-window size, and verify the stored dimensions override the 460×540 default.

### Implementation for User Story 1

- [x] T006 [US1] Set the 460×540 default and add logical-size validation/conversion plus preference application in `src-tauri/src/windowing.rs`
- [x] T007 [US1] Seed runtime size from storage, apply it to new result windows, and flush it on explicit quit in `src-tauri/src/state.rs` and `src-tauri/src/commands.rs`
- [x] T008 [US1] Record valid logical result-window resize events and flush the latest dimensions before result cleanup in `src-tauri/src/lib.rs`

**Checkpoint**: Later result windows use the latest valid resize in the session, and normal close/quit makes it available after restart.

## Phase 4: User Story 2 - Keep screenshot previews compact (Priority: P2)

**Goal**: Apply the same 60-pixel maximum height to result, history-list, and history-detail screenshot previews.

**Independent Test**: Inspect the three CSS rules and render the existing Result/History image elements to confirm their original width fitting and accessible labels are unchanged while their maximum height is 60 pixels.

### Implementation for User Story 2

- [x] T009 [US2] Change the result, history-list, and history-detail screenshot preview height limits to 60 pixels in `src/styles.css`

**Checkpoint**: All three image placements remain proportionate and compact without changing their markup or behavior.

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Synchronize the behavior-release version and verify the complete feature.

- [x] T010 Update the synchronized patch version in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`
- [ ] T011 [P] Run focused frontend tests for `src/views/Result.test.tsx` and `src/views/History.test.tsx` with `powershell.exe -Command "npm test -- --runInBand"` or the project's supported focused Vitest command
- [ ] T012 [P] Run focused Rust storage and lifecycle tests with `cargo test --manifest-path src-tauri/Cargo.toml --test storage_foundation --test desktop_lifecycle`
- [ ] T013 [P] Run `powershell.exe -Command "npm run typecheck"`, `powershell.exe -Command "npm run lint"`, and formatting checks, recording any pre-existing repository-wide failures
- [x] T014 Run `powershell.exe -Command "npm run tauri build"` and install the locally built Windows package before commit, recording output or any blocker
- [ ] T015 Perform the manual result/history visual review from `specs/017-result-window-size-memory/quickstart.md` and record the outcome in `specs/017-result-window-size-memory/validation.md`

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Test assertions can be prepared before implementation.
- **Foundational (Phase 2)**: T004 must precede T005 because persistence requires schema support.
- **User Story 1 (Phase 3)**: Depends on T004–T005. T006 establishes geometry rules before T007–T008 connect native lifecycle behavior.
- **User Story 2 (Phase 4)**: May proceed independently after T003 because it changes only the existing CSS rules.
- **Polish (Phase 5)**: Follows code completion and combines automated, package, and visual validation.

### User Story Dependencies

- **User Story 1 (P1)**: Independent MVP.
- **User Story 2 (P2)**: Independent of the persisted window geometry behavior.

### Parallel Opportunities

- T001–T003 work on distinct test files and can proceed in parallel.
- T009 can proceed independently of T004–T008.
- T011–T013 can run in parallel after implementation.

## Implementation Strategy

### MVP First

1. Establish migration and durable storage for the preference.
2. Apply the persisted preference to native result-window construction.
3. Record resize events in runtime and persist the latest size on close/quit.
4. Validate storage, lifecycle, and default-size behavior.

### Incremental Delivery

1. Deliver User Story 1 for durable geometry reuse.
2. Deliver User Story 2 as the isolated CSS limit change.
3. Complete cross-cutting validation, packaging, local install, commit, push, tag, and release.
