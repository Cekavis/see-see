# Tasks: Thinking Stream Display

**Input**: Design documents from `/specs/007-thinking-stream-display/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This behavioral change requires focused Rust provider and lifecycle tests plus frontend stream-state and disclosure tests.

**Organization**: Tasks are grouped into provider normalization, result presentation, and history persistence stories; each is independently testable.

## Phase 1: Setup

**Purpose**: Confirm the existing single stream and result paths used by the feature.

- [x] T001 Confirm provider normalization, analysis lifecycle, IPC state, and result rendering remain centralized in `src-tauri/src/providers/`, `src-tauri/src/analysis.rs`, `src/ipc.ts`, `src/App.tsx`, and `src/views/Result.tsx`

---

## Phase 2: Foundational

**Purpose**: No new infrastructure or dependency is required; reuse existing stream, snapshot, and design-token patterns.

- [x] T002 Confirm the locked serde/eventsource and native HTML disclosure capabilities cover the implementation in `src-tauri/Cargo.lock` and `package.json`

---

## Phase 3: User Story 1 - Separate Thinking from the Answer (Priority: P1) 🎯 MVP

**Goal**: Normalize structured and tagged provider thinking separately from the final answer.

**Independent Test**: Run provider and analysis-flow tests with OpenAI-compatible, Anthropic, Gemini, split-tag, and answer-only fixtures.

### Tests for User Story 1

- [x] T003 [US1] Add provider fixtures for structured thinking fields, thought parts, and split leading `<think>` tags in `src-tauri/tests/provider_contracts.rs`
- [x] T004 [US1] Add analysis accumulation, completion, retry, and cancellation assertions for separate thinking and answer text in `src-tauri/tests/analysis_flow.rs`

### Implementation for User Story 1

- [x] T005 [US1] Add normalized thinking events, shared cross-chunk tag parsing, answer-only completion output, and Gemini thought-summary requests in `src-tauri/src/providers/mod.rs`, `src-tauri/src/providers/openai.rs`, `src-tauri/src/providers/anthropic.rs`, and `src-tauri/src/providers/gemini.rs`
- [x] T006 [US1] Carry thinking through analysis snapshots and IPC events while persisting only final-answer text in `src-tauri/src/analysis.rs`

**Checkpoint**: Provider thinking and final-answer text are separated before reaching the frontend, including across arbitrary tag boundaries.

---

## Phase 4: User Story 2 - Follow Streaming Thinking without Losing the Answer (Priority: P2)

**Goal**: Show thinking expanded while it streams and collapse it when the answer begins.

**Independent Test**: Render thinking-only, first-answer, completed, failed, and answer-only snapshots; verify native disclosure state, keyboard accessibility, controls, and copy payload.

### Tests for User Story 2

- [x] T007 [US2] Add frontend state-reducer assertions for thinking deltas and terminal snapshots in `src/App.test.ts`
- [x] T008 [US2] Add disclosure transition, answer-only, safe plain-text, copy, and compact-layout assertions in `src/views/Result.test.tsx`

### Implementation for User Story 2

- [x] T009 [US2] Extend frontend analysis types and state reduction with thinking in `src/ipc.ts` and `src/App.tsx`
- [x] T010 [US2] Render a native thinking disclosure and preserve scrolling/footer behavior in `src/views/Result.tsx` and `src/styles.css`

**Checkpoint**: Thinking is visible during its stream, collapses at the first answer text, remains user-expandable, and never pollutes copy output.

---

## Phase 5: User Story 3 - Revisit Saved Thinking (Priority: P3)

**Goal**: Persist thinking separately and show it in history detail without changing previews, search, or copy.

**Independent Test**: Upgrade an existing database, save success and failure entries with thinking, open history detail, and verify the collapsed disclosure and answer-only copy behavior.

### Tests for User Story 3

- [x] T011 [US3] Add migration, success/failure persistence, and detail restoration assertions in `src-tauri/src/database.rs` and `src-tauri/tests/history_integration.rs`
- [x] T012 [US3] Add history detail disclosure and answer-only copy assertions in `src/views/History.test.tsx`

### Implementation for User Story 3

- [x] T013 [US3] Add the nullable thinking history migration and persistence contract in `src-tauri/migrations/0003_history_thinking.sql`, `src-tauri/src/database.rs`, `src-tauri/src/history.rs`, and `src-tauri/src/analysis.rs`
- [x] T014 [US3] Expose history thinking over IPC and reuse the disclosure in `src/ipc.ts`, `src/views/Result.tsx`, and `src/views/History.tsx`

**Checkpoint**: Saved thinking survives restart and appears only in history detail.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T015 Bump the synchronized compatible-feature version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- [x] T016 Run focused tests, full Rust/frontend tests, lint, formatting, frontend build, and release bundle commands from `specs/007-thinking-stream-display/quickstart.md`
- [x] T017 Install the Windows release bundle and visually review result/history thinking layouts, answer-only behavior, keyboard, copy, normal-size, and compact-size states using `specs/007-thinking-stream-display/quickstart.md`
- [x] T018 Record the unavailable macOS visual-review gap in `specs/007-thinking-stream-display/quickstart.md` if macOS is unavailable

---

## Dependencies & Execution Order

- T001 and T002 confirm the existing path and constraints.
- T003 and T004 are written before T005 and T006 and must fail until the provider and analysis changes exist.
- T005 precedes T006 because analysis consumes normalized provider events.
- T007 and T008 are written before T009 and T010 and must fail until frontend state and presentation change.
- T009 precedes T010 because the result component consumes the extended snapshot.
- T011 and T012 are written before T013 and T014.
- T013 precedes T014 because history detail consumes the persisted field.
- T015 follows behavior implementation; T016 through T018 complete verification.

## Parallel Opportunities

- T003 and T004 touch separate test files but target dependent layers; keeping them sequential makes the smallest TDD loop clearer.
- T007 and T008 touch separate test files but share the extended snapshot contract; keeping them sequential avoids temporary duplicate types.
- T011 and T012 cover separate Rust and frontend history layers but share the new detail contract; keeping them sequential keeps the migration loop explicit.
- No subagent or additional coordination is warranted for this concentrated change.

## Implementation Strategy

Complete P1 normalization, add the P2 native disclosure, then persist the same thinking field for P3 history detail. Reuse existing structures and omit arbitrary request JSON and custom accordion code.
