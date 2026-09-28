# Tasks: Result Model Retry

## Phase 1: Setup

- [x] T001 Inspect result, retained request, launch path, shared components, and repository rules; document `specs/019-result-model-retry/spec.md` and design artifacts.

## Phase 2: Foundation

- [x] T002 Define independent-run IPC and chooser props in `specs/019-result-model-retry/contracts/result-model-retry.md`.

## Phase 3: US1 — Retry in a new result window

Independent test: Selecting a model creates a distinct run using the original screenshot/prompt without mutating the source or active model.

- [x] T003 [P] [US1] Add immutable retry-input regressions in `src-tauri/tests/analysis_flow.rs` before backend changes.
- [x] T004 [US1] Add selected-model input cloning and new command in `src-tauri/src/analysis.rs`, `src-tauri/src/commands.rs`, and `src-tauri/src/lib.rs`.
- [x] T005 [P] [US1] Add selected-model IPC and footer interaction regression tests in `src/ipc.test.ts` and `src/views/Result.test.tsx`.
- [x] T006 [US1] Wire new command and chooser into `src/ipc.ts`, `src/App.tsx`, and `src/views/Result.tsx`.

## Phase 4: US2 — Select among many models

Independent test: With 500 models, scroll or navigate to a target using the keyboard; verify recovery and bounded layout.

- [x] T007 [P] [US2] Add name-only list, keyboard, lifecycle, failure, and duplicate-submit tests in `src/components/ModelRetryDialog.test.tsx`.
- [x] T008 [US2] Implement `src/components/ModelRetryDialog.tsx` using shared Button and dialog patterns.
- [x] T009 [US2] Style the bounded responsive list in `src/styles.css`; use compact 32px rows showing only saved names, with ellipsis and full-name titles for overflow in `src/components/ModelRetryDialog.tsx`.

## Phase 5: Validation and delivery

- [x] T010 Synchronize 0.17.0 in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
- [x] T011 Run repository validation gates and record evidence in `specs/019-result-model-retry/validation.md`.
- [x] T012 Review chooser and footer at 460×500 and 420×360; save screenshots under `specs/019-result-model-retry/`.
- [x] T013 Build and install locally; record verification and limitations in `specs/019-result-model-retry/validation.md`.
- [x] T014 Inspect status/diff and prepare scoped implementation plus `specs/019-result-model-retry/` for atomic delivery; commit/push publication is recorded in Git history.
- [x] T015 Remove search/count/progress UI as requested; preserve compact rows, bounded scrolling, keyboard navigation, and recovery tests.
- [x] T016 Fix Windows retry-window deadlock using async command dispatch and extend the existing compile-time regression.
- [x] T017 Fix duplicate development text/thinking output with effect-local subscription cleanup and failing-before/passing-after StrictMode tests.

## Dependencies and parallel execution

T001 → T002 → backend track (T003–T004) and standalone chooser track (T007–T009). Main agent performs T005–T006 while both independent tracks run. All converge before T010–T014. Within each track, tests precede implementation. US1 delivers the new window; US2 makes its chooser practical for large configuration lists.

## Implementation strategy

Preserve existing retry and independent-window behavior. Implement backend and standalone chooser in separate file sets; integrate once, run focused regressions, then repository checks and visual review. Keep deployment and git operations in the main agent after validation.
