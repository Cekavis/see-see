# Tasks: Anthropic Messages Reasoning Effort

**Input**: Design documents from `/specs/020-anthropic-messages-reasoning/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Behavioral changes require focused frontend and Rust regression coverage.

## Phase 1: User Story 1 - Configure Anthropic effort (Priority: P1)

**Goal**: Show and persist Anthropic's official effort levels with a `low` default.

**Independent Test**: Select Anthropic in the settings editor, inspect the options/default, save, and reopen.

- [x] T001 [US1] Add frontend model-settings regressions for Anthropic visibility, five official options, low default, save, edit, and test draft payload in `src/views/Settings.model.test.tsx`.
- [x] T002 [US1] Extend protocol-aware reasoning option selection and save/test payloads for Anthropic in `src/views/Settings.tsx`.
- [x] T003 [US1] Accept Anthropic reasoning in protocol validation, default missing values to low, and reject unsupported enum values in `src-tauri/src/providers/mod.rs` and `src-tauri/src/settings.rs`.
- [x] T004 [US1] Update model persistence regressions for Anthropic default and supported values in `src-tauri/tests/model_config.rs`.

## Phase 2: User Story 2 - Send the selected effort (Priority: P1)

**Goal**: Forward the selected value using Anthropic's official Messages API field.

**Independent Test**: Prepare Anthropic requests for every supported effort and inspect the JSON body.

- [x] T005 [US2] Add Anthropic request-body contract assertions and invalid-value coverage in `src-tauri/tests/provider_contracts.rs`.
- [x] T006 [US2] Add conditional `output_config.effort` mapping to the Anthropic Messages adapter in `src-tauri/src/providers/anthropic.rs`.

## Phase 3: Polish & Cross-Cutting Concerns

- [x] T007 Synchronize the compatible feature version to 0.19.0 in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
- [x] T008 Run focused and full validation, update this task list and a validation record, build the release bundle, install locally, then commit and push `master`.

## Dependencies & Execution Order

- T001 and T005 are test-first tasks and may be prepared independently.
- T002 and T003 depend on the corresponding focused tests; T004 follows T003; T006 follows T005.
- T007 follows the implementation; T008 follows all code and test changes.
