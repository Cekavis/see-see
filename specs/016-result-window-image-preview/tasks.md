# Tasks: Result Window Image Preview

**Input**: Design documents from `specs/016-result-window-image-preview/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/ipc.md`, `quickstart.md`

**Tests**: Behavioral changes use focused frontend and Rust regression tests; visual behavior also requires manual review at the updated default and minimum result-window sizes.

## Phase 1: Setup

**Purpose**: Establish failing regression coverage for the requested result-window behavior.

- [x] T001 [P] [US1] Add Result component regression assertions for the original screenshot preview, accessible label, bounded image height, and four-row result layout in `src/views/Result.test.tsx`
- [x] T002 [P] [US1] Add App result-window assertions for run-scoped image loading, object-URL cleanup, and non-blocking retrieval failure in `src/App.test.ts`
- [x] T003 [P] [US1] Add Rust source-contract assertions for `get_analysis_image`, command registration, run isolation, and the updated native size values in `src-tauri/tests/desktop_lifecycle.rs`

## Phase 2: Foundational

**Purpose**: Add the run-scoped image access path and preserve the existing runtime boundaries.

- [x] T004 [US1] Implement `get_analysis_image` using the existing `active_analysis` lookup and `ActiveAnalysis::image_png` in `src-tauri/src/commands.rs`
- [x] T005 [US1] Register `get_analysis_image` in the Tauri command list in `src-tauri/src/lib.rs`
- [x] T006 [US1] Add the typed `getAnalysisImage` IPC wrapper returning binary image data in `src/ipc.ts`

## Phase 3: User Story 1 - Compare the screenshot with the result (Priority: P1) 🎯 MVP

**Goal**: Show the current analysis screenshot above the result while preserving streaming, scrolling, actions, and compact-window usability.

**Independent Test**: Render a Result with a valid preview URL and run the focused frontend/Rust tests; then visually review wide and tall screenshots at 460×750 and 420×540 in streaming and completed states.

### Implementation for User Story 1

- [x] T007 [US1] Load and revoke the current run's image object URL, report retrieval failures through existing notifications, and pass the URL to Result in `src/App.tsx`
- [x] T008 [US1] Render the optional original-screenshot preview row with meaningful alternative text in `src/views/Result.tsx`
- [x] T009 [US1] Add responsive result-preview styling with width fitting, preserved proportions, a 220-pixel maximum height, and a four-row result grid in `src/styles.css`
- [x] T010 [US1] Increase result-window default/minimum heights to 750/540 while retaining 460/420 widths in `src-tauri/src/windowing.rs`

**Checkpoint**: The result window shows the correct screenshot for its run, the result text remains the scrolling region, and existing actions remain reachable at both supported sizes.

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Verify the complete feature and record evidence.

- [x] T011 [P] Run focused frontend tests for `src/views/Result.test.tsx` and `src/App.test.ts`, then run the full frontend test suite with `powershell.exe -Command "npm test"`
- [x] T012 [P] Run `powershell.exe -Command "npm run typecheck"`, `powershell.exe -Command "npm run lint"`, and formatting checks; the changed files pass while the full repository check reports only the pre-existing `AGENTS.md`
- [x] T013 [P] Run `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` and the full Rust test suite, recording unrelated failures if any
- [ ] T014 Perform manual visual review using `specs/016-result-window-image-preview/quickstart.md` and record pass/fail evidence for wide/tall images, streaming/completed states, 460×750, 420×540, and light/dark appearances

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No implementation dependencies; tests should fail or identify the missing behavior before code changes.
- **Foundational (Phase 2)**: Depends on the failing test definitions in Phase 1; T005 depends on T004, and T006 can be completed independently of the Rust command implementation.
- **User Story 1 (Phase 3)**: Depends on T004–T006 for the data path; T007–T010 touch separate implementation areas except that T007 and T008 are integrated through the Result prop.
- **Polish (Phase 4)**: Depends on all implementation tasks and includes the final automated/manual verification.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on another user story; it is the complete MVP.

### Parallel Opportunities

- T001, T002, and T003 can run in parallel because they update separate test files.
- T005 and T006 can run in parallel after T004 is designed, because command registration and the frontend wrapper are separate files.
- T008, T009, and T010 can run in parallel after T007's prop shape is agreed, because they update separate source areas.
- T011, T012, and T013 can run in parallel after implementation; T014 follows the automated checks for the visual evidence.

## Parallel Example: User Story 1

```text
Task T001: Result component and CSS regression assertions in src/views/Result.test.tsx
Task T002: App image lifecycle assertions in src/App.test.ts
Task T003: Rust command/size source-contract assertions in src-tauri/tests/desktop_lifecycle.rs
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Add the focused failing tests in T001–T003.
2. Implement the current-run image path in T004–T006.
3. Implement the preview, layout, and window-size changes in T007–T010.
4. Run T011–T013, then complete the visual review in T014.

### Incremental Delivery

The feature is intentionally one independently testable story. The safe increment is the full image-preview path plus the proportional window-height change; no history schema or unrelated settings behavior is included.
