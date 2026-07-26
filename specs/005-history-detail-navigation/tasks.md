# Tasks: History Detail Navigation

**Input**: Design documents from `/specs/005-history-detail-navigation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Focused component regressions are required for the history behavior changes. Sidebar sizing receives build and representative visual verification.

**Organization**: Tasks are grouped by user story so navigation, summary readability, and sidebar sizing can be validated independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the existing project safeguards needed by implementation.

- [x] T001 Verify Node/Rust/build/environment ignore coverage remains complete in `.gitignore`, `eslint.config.js`, and `.prettierignore`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: No new shared infrastructure is required; the existing History API, shared UI components, and settings scroll container are reused as documented in `specs/005-history-detail-navigation/research.md`.

**Checkpoint**: Existing foundations are ready for story work.

---

## Phase 3: User Story 1 - Open and Return from History Detail (Priority: P1) 🎯 MVP

**Goal**: Open history detail as a dedicated view and return to the preserved list context.

**Independent Test**: Apply filters, set a list scroll offset, open detail, then return and verify exclusive rendering, retained filter/result state, restored scroll offset, and no new list query.

### Tests for User Story 1 ⚠️

- [x] T002 [US1] Add failing dedicated-detail, return-state, scroll-restoration, and detail-load-failure regressions in `src/views/History.test.tsx`

### Implementation for User Story 1

- [x] T003 [US1] Implement mutually exclusive list/detail rendering, accessible return navigation, and layout-phase scroll restoration in `src/views/History.tsx`

**Checkpoint**: User Story 1 passes independently, including existing copy and resubmit actions.

---

## Phase 4: User Story 2 - Read Structured History Summaries (Priority: P2)

**Goal**: Preserve stored newlines and blank lines in list summaries without horizontal overflow.

**Independent Test**: Render a preview containing consecutive newline characters and verify it uses whitespace-preserving summary markup with unchanged text.

### Tests for User Story 2 ⚠️

- [x] T004 [US2] Add a failing structured-summary rendering regression in `src/views/History.test.tsx`

### Implementation for User Story 2

- [x] T005 [US2] Render history previews with whitespace-preserving accessible markup in `src/views/History.tsx` and wrapping styles in `src/styles.css`

**Checkpoint**: User Story 2 passes independently for both multiline and long-line summaries.

---

## Phase 5: User Story 3 - Use a More Compact Sidebar (Priority: P3)

**Goal**: Recover desktop content width while keeping navigation labels and responsive behavior usable.

**Independent Test**: At 1094×768, confirm the sidebar is at least 15% narrower with all labels visible; at 780×800 and 540×800, confirm existing horizontal navigation remains usable.

### Implementation for User Story 3

- [x] T006 [US3] Reduce the desktop sidebar grid track while preserving existing compact breakpoints in `src/styles.css`

**Checkpoint**: User Story 3 is visually complete at representative widths.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release synchronization and full verification across all stories.

- [x] T007 Synchronize the compatible-feature minor version in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- [x] T008 Run focused and full automated validation from `specs/005-history-detail-navigation/quickstart.md`
- [x] T009 Perform accessibility and human visual review of list/detail states at 1094×768, 780×800, and 540×800 and record results in `specs/005-history-detail-navigation/quickstart.md`
- [x] T010 Build the Tauri release bundle and install the local application according to `specs/005-history-detail-navigation/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup verification precedes all implementation.
- User Story 1, User Story 2, and User Story 3 use existing foundations and are independently testable.
- Polish and release tasks depend on all selected user stories.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on another story; this is the MVP.
- **User Story 2 (P2)**: No behavioral dependency on User Story 1, though both touch the History files and execute sequentially.
- **User Story 3 (P3)**: No behavioral dependency on User Stories 1 or 2; stylesheet edits execute after User Story 2 to avoid file conflicts.

### Within Each User Story

- Write and run the focused regression test before corresponding behavior.
- Implement the smallest change that satisfies the story contract.
- Re-run the focused test at each checkpoint.

### Parallel Opportunities

- No tasks are marked `[P]` because the small task set shares `History.tsx`, `History.test.tsx`, or `styles.css`, and ordered execution avoids overlapping edits.
- After implementation, frontend and Rust test commands may run concurrently where resource limits allow.

---

## Implementation Strategy

### MVP First

1. Verify setup safeguards.
2. Add and pass User Story 1 regressions.
3. Stop and validate the dedicated detail/return journey independently.

### Incremental Delivery

1. Add User Story 2 summary semantics and wrapping.
2. Add User Story 3 compact desktop sizing.
3. Synchronize the release version.
4. Run automated, visual, release-build, and local-install checks.

## Notes

- All tasks include concrete file paths and follow the required checklist format.
- Do not change history storage or IPC contracts.
- Existing tests must remain enabled and must not be weakened.
