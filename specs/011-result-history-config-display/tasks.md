# Tasks: Result and History Configuration Display

**Input**: Design documents from `/specs/011-result-history-config-display/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Behavioral snapshot, original-image selection, pagination, and user-facing layout require focused regressions before implementation.

**Organization**: User Story 1 owns result configuration identity; User Story 2 owns history metadata, original images, layout, and bounded paging.

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Trace analysis snapshot creation/retry, result rendering, history cursor/limit queries, image variants, card styling, and ignore coverage in src-tauri/src/analysis.rs, src-tauri/src/commands.rs, src-tauri/src/history.rs, src/App.tsx, src/views/Result.tsx, src/views/History.tsx, src/styles.css, .gitignore, eslint.config.js, and .prettierignore

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T002 Add failing snapshot and retry configuration-name tests in src/App.test.ts and src-tauri/tests/analysis_flow.rs
- [x] T003 Add failing result metadata and history metadata/layout tests in src/views/Result.test.tsx and src/views/History.test.tsx
- [x] T004 Add failing original-image, no-empty-region, cursor pagination, and page-size tests in src/views/History.test.tsx

## Phase 3: User Story 1 - Identify Result Configuration (Priority: P1) 🎯 MVP

**Goal**: Show the exact model and prompt configuration names used by the current analysis attempt.

**Independent Test**: Attach and retry an analysis and verify both names remain correct throughout the result lifecycle.

- [x] T005 [US1] Extend analysis snapshot creation, started events, and retry reset with configuration names in src-tauri/src/analysis.rs and src-tauri/src/commands.rs
- [x] T006 [US1] Extend frontend snapshot typing/state defaults and render compact result metadata in src/ipc.ts, src/App.tsx, src/views/Result.tsx, and src/styles.css

## Phase 4: User Story 2 - Scan Paged History with Original Screenshots (Priority: P2)

**Goal**: Show saved model names, render each original screenshot at natural capped height above content, and limit records/images to the selected page.

**Independent Test**: Render wide, tall, and missing images and navigate cursor pages at 10, 20, and 50 items per page.

- [x] T007 [US2] Add the saved model configuration name to history list metadata in src/views/History.tsx
- [x] T008 [US2] Request original images, omit absent image regions, and apply natural aspect plus a maximum height in src/views/History.tsx and src/styles.css
- [x] T009 [US2] Replace cumulative loading with cursor-backed previous/next paging and a 10/20/50 page-size selector in src/views/History.tsx and src/styles.css

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T010 Synchronize version 0.10.0 in package.json, package-lock.json, src-tauri/Cargo.toml, src-tauri/Cargo.lock, and src-tauri/tauri.conf.json
- [x] T011 Run focused tests, full tests, lint, formatting, frontend build, E2E smoke flow, release checks, and cargo test using specs/011-result-history-config-display/quickstart.md
- [x] T012 Run signed npm run tauri build and install the generated Windows bundle locally
- [x] T013 Perform human visual review at 460×500, 420×360, 1094×768, and 540×800 and record evidence in specs/011-result-history-config-display/validation.md
- [x] T014 Inspect final status/diff, mark all tasks complete, commit the atomic feature, and push origin/master

## Dependencies & Execution Order

- Setup precedes focused regression contracts.
- User Story 1 depends on snapshot tests; User Story 2 depends on history rendering and pagination tests.
- The two stories share the stylesheet, so implementation remains sequential.
- Validation, signed release build, installation, visual review, commit, and push follow both stories.

## Implementation Strategy

1. Extend the existing snapshot instead of adding lookup IPC.
2. Render the history field already present in list data.
3. Reuse the existing original-image variant and cursor/limit query contract.
4. Keep page navigation previous/next only; skip total counts and random page jumps.

## Notes

- No migration, dependency, live configuration lookup, total-count query, or new component abstraction is required.
- Preserve saved historical names even when configurations are renamed or deleted.
- Original images are bounded by the visible page size rather than accumulated in memory.
