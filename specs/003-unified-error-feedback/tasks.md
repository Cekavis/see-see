# Tasks: Unified App Feedback

**Input**: Design documents from `/specs/003-unified-error-feedback/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/notification-ui.md, quickstart.md

**Tests**: Behavioral changes require focused component and view regression tests before implementation, followed by full automated and native UI validation.

**Organization**: Tasks are grouped by user story so each outcome remains independently testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when dependencies are already satisfied
- **[Story]**: Maps the task to a user story in spec.md
- Every task includes the affected file paths

## Phase 1: Setup and Baseline

**Purpose**: Confirm the existing project state and preserve the current release version.

- [X] T001 Verify the clean `master` baseline, existing ignore rules, and synchronized unchanged version 0.2.2 in `.gitignore`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`

---

## Phase 2: User Story 1 - See operation errors immediately (Priority: P1) 🎯 MVP

**Goal**: All operation errors appear in a persistent, viewport-fixed, non-layout notification with dismissal and optional recovery.

**Independent Test**: In a short scrolled settings window and the result window, trigger errors and verify immediate visibility, zero page movement, dismissal, and retry behavior.

### Tests for User Story 1

- [X] T002 [US1] Add failing tests for persistent error publication, repeated-message identity, dismissal, and recovery actions in `src/components/Notifications.test.tsx`
- [X] T003 [P] [US1] Update view regression expectations for shared error publication in `src/views/Settings.model.test.tsx`, `src/views/Settings.desktop.test.tsx`, `src/views/Prompts.test.tsx`, `src/views/History.test.tsx`, `src/views/Onboarding.test.tsx`, and `src/views/Result.test.tsx`

### Implementation for User Story 1

- [X] T004 [US1] Implement the window-scoped provider, error API, portal viewport, dismissal, and recovery lifecycle in `src/components/Notifications.tsx` and wrap the app root in `src/main.tsx`
- [X] T005 [P] [US1] Add error and close severity cues to the shared icon set in `src/components/Icon.tsx`
- [X] T006 [US1] Migrate operation errors in `src/views/DesktopSettings.tsx`, `src/views/Onboarding.tsx`, `src/views/Settings.tsx`, `src/views/Prompts.tsx`, `src/views/History.tsx`, and `src/views/Result.tsx`, preserving persistent history failure content in `src/components/ErrorNotice.tsx`

**Checkpoint**: Error feedback is independently functional in every existing operation surface.

---

## Phase 3: User Story 2 - Receive consistent success confirmation (Priority: P2)

**Goal**: Useful asynchronous successes use the same viewport-fixed pattern, remain readable for four seconds, and disappear without reflow.

**Independent Test**: Trigger save, copy, activation, deletion, and export outcomes; verify consistent success semantics, timeout, manual dismissal, and no stale contradictory error.

### Tests for User Story 2

- [X] T007 [US2] Add failing success timeout, manual dismissal, and same-message repetition tests in `src/components/Notifications.test.tsx`
- [X] T008 [P] [US2] Add success integration expectations for representative actions in `src/views/Settings.model.test.tsx`, `src/views/Settings.desktop.test.tsx`, `src/views/Prompts.test.tsx`, `src/views/History.test.tsx`, and `src/views/Result.test.tsx`

### Implementation for User Story 2

- [X] T009 [US2] Add success publication and automatic lifecycle behavior to `src/components/Notifications.tsx`
- [X] T010 [US2] Publish useful success outcomes and clear stale feedback in `src/views/Settings.tsx`, `src/views/DesktopSettings.tsx`, `src/views/Prompts.tsx`, `src/views/History.tsx`, and `src/views/Result.tsx`

**Checkpoint**: Success and error outcomes share one coherent lifecycle without changing page layout.

---

## Phase 4: User Story 3 - Understand feedback accessibly and responsively (Priority: P3)

**Goal**: Notifications remain distinguishable, readable, announced, and operable at compact sizes, in both themes, with keyboard and reduced-motion preferences.

**Independent Test**: At 320×240 and larger viewports, verify semantic roles, focusable named controls, wrapping, stack scrolling, theme tokens, and reduced motion.

### Tests for User Story 3

- [X] T011 [US3] Extend notification tests for alert/status semantics, named controls, keyboard activation, newest-first ordering, and portal rendering in `src/components/Notifications.test.tsx`

### Implementation for User Story 3

- [X] T012 [US3] Implement compact fixed-stack layout, wrapping, internal overflow, severity styling, focus states, dark-theme tokens, and reduced-motion behavior in `src/styles.css`
- [X] T013 [US3] Remove obsolete page-level notice spacing and success styles while retaining explicit inline failure styling in `src/styles.css` and `src/components/ErrorNotice.tsx`

**Checkpoint**: The shared feedback pattern meets accessibility and responsive acceptance criteria.

---

## Phase 5: Polish and Cross-Cutting Validation

**Purpose**: Verify the complete product, package, and native UI before delivery.

- [X] T014 Run formatting, lint, frontend tests, frontend build, end-to-end smoke tests, and Rust tests using `package.json`, `tests/e2e/`, and `src-tauri/Cargo.toml`
- [X] T015 Build the release bundle with `npm run tauri build`, install the macOS app locally, and confirm the installed version remains 0.2.2 using `src-tauri/tauri.conf.json`
- [X] T016 Perform native visual and keyboard review at the 720×520 main-window minimum and standard settings size, and verify the 420×360 compact result-window contract through focused component coverage; record evidence against `specs/003-unified-error-feedback/quickstart.md`
- [X] T017 Read back the final specification, contract, implementation diff, and completed checklist; mark all tasks complete in `specs/003-unified-error-feedback/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **User Story 1 (Phase 2)**: Depends on T001 and provides the provider/error foundation.
- **User Story 2 (Phase 3)**: Depends on T004 and T006; success lifecycle can then extend the provider.
- **User Story 3 (Phase 4)**: Depends on the complete notification markup from T004 and T009.
- **Validation (Phase 5)**: Depends on all user stories.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency beyond baseline; forms the MVP.
- **User Story 2 (P2)**: Reuses the provider created by US1 but remains independently testable through success actions.
- **User Story 3 (P3)**: Applies accessibility and responsive requirements to both prior variants.

### Within Each User Story

- Write focused tests before the behavior they cover.
- Implement shared lifecycle before migrating views.
- Preserve the existing sanitized error-message boundary.
- Complete story checks before moving to the next priority.

### Parallel Opportunities

- T003 and T005 touch separate files after T002 defines the contract.
- T008 can prepare representative view expectations while T007 defines provider timing behavior.
- Automated command groups in T014 may run independently when they do not compete for the same build output.

---

## Parallel Example: User Story 1

```text
Task T003: Update view error-publication regression expectations.
Task T005: Add shared error and close icons.
```

---

## Implementation Strategy

### MVP First

1. Confirm baseline and version.
2. Add failing provider and view tests.
3. Implement persistent viewport-fixed errors and migrate all operation errors.
4. Validate the compact scrolled-window error journey.

### Incremental Delivery

1. Error foundation and migration deliver the primary visibility fix.
2. Success lifecycle and action confirmations complete consistency.
3. Accessibility and responsive styling harden all variants.
4. Full build, package, installation, and native review establish release readiness.

## Notes

- No dependency, backend protocol, storage, or release-version change is planned.
- Field validation and stored failed-history content remain inline by design.
- Every task follows the required checkbox, ID, story-label, and file-path format.
