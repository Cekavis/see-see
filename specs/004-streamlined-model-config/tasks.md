# Tasks: Streamlined Model Configuration

**Input**: Design documents from `/specs/004-streamlined-model-config/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Behavioral changes require Rust and frontend regression coverage; the conditional editor flow also receives browser smoke coverage and native visual review.

**Organization**: Tasks are grouped by user story. Stories 1–3 share P1; storage is implemented first because the other model flows consume its contract.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it affects different files and has no dependency on incomplete work
- **[Story]**: Maps the task to a specification user story
- Every task names the exact files it changes or validates

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the release metadata and project safeguards for a compatible feature change

- [x] T001 Synchronize the minor version to 0.3.0 in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- [x] T002 [P] Verify secret, dependency, log, build-output, formatter, and linter exclusions in `.gitignore`, `.prettierignore`, and `eslint.config.js`

---

## Phase 2: User Story 2 - Save Complete Reusable Connection Details (Priority: P1)

**Goal**: Store each API key as plain text in the endpoint's SQLite row, migrate legacy credentials without data loss, and keep saved keys redacted from the webview and logs

**Independent Test**: Save, edit without replacement, explicitly clear, migrate, reload, and use keyed configurations; summaries expose only `hasApiKey`, and credential-store errors do not block other configuration access.

### Tests for User Story 2

- [x] T003 [US2] Add failing same-row key persistence, preserve/clear, exact-value, redacted-summary, and copy-safe fixtures in `src-tauri/tests/model_config.rs`
- [x] T004 [US2] Add failing idempotent legacy credential migration and schema-version checks in `src-tauri/tests/model_config.rs` and `src-tauri/tests/storage_foundation.rs`

### Implementation for User Story 2

- [x] T005 [US2] Add the version-3 `api_key` schema migration and clear obsolete test values in `src-tauri/migrations/0001_init.sql`, `src-tauri/migrations/0002_plaintext_model_keys.sql`, and `src-tauri/src/database.rs`
- [x] T006 [US2] Persist, preserve, clear, list, load, and best-effort migrate same-row model keys while redacting summaries in `src-tauri/src/settings.rs`
- [x] T007 [US2] Switch analysis/provider authentication to SQLite keys and constrain the keyring store to startup migration in `src-tauri/src/commands.rs`, `src-tauri/src/state.rs`, and `src-tauri/src/lib.rs`

**Checkpoint**: Keyed configurations survive reload and legacy migration without returning stored key values to the webview.

---

## Phase 3: User Story 3 - Test Independently from Saving (Priority: P1)

**Goal**: Make connection testing a transient draft diagnostic and permit saving/activation without testing

**Independent Test**: Test successful and failing drafts without changing the database, then save and activate a configuration that has never been tested.

### Tests for User Story 3

- [x] T008 [US3] Replace persisted-test and activation-gate expectations with no-write test and untested-activation regressions in `src-tauri/tests/model_config.rs` and `src-tauri/tests/provider_contracts.rs`
- [x] T009 [US3] Add frontend regressions proving testing never saves or activates and untested cards can become current in `src/views/Settings.model.test.tsx`

### Implementation for User Story 3

- [x] T010 [US3] Remove model test-state serialization/writes and the passed-test activation gate in `src-tauri/src/settings.rs` and `src-tauri/src/commands.rs`
- [x] T011 [US3] Remove persisted test fields from IPC and make the test action call the current draft directly in `src/ipc.ts` and `src/views/Settings.tsx`
- [x] T012 [US3] Update onboarding readiness language so current-model selection no longer implies testing in `src/views/Onboarding.tsx` and `src/views/Onboarding.test.tsx`

**Checkpoint**: Testing has no persistent effect, while saving and activation remain available independently.

---

## Phase 4: User Story 1 - Manage Configurations without Form Clutter (Priority: P1) 🎯 MVP

**Goal**: Default to a list-only model page and render the editor only during explicit add or edit flows

**Independent Test**: Load the page, add, cancel, edit, fail a save, and successfully save; verify the editor DOM appears and closes only at the specified transitions.

### Tests for User Story 1

- [x] T013 [US1] Add failing editor visibility, add/edit/cancel, failed-save retention, successful-save closure, focus, busy, and delete-while-editing tests in `src/views/Settings.model.test.tsx`
- [x] T014 [P] [US1] Add conditional model-editor smoke coverage in `tests/e2e/primary-flow.mjs`

### Implementation for User Story 1

- [x] T015 [US1] Implement the closed/add/edit form state machine, list-first layout, header add action, contextual key warning, cancel behavior, and conflicting-action disabling in `src/views/Settings.tsx`
- [x] T016 [US1] Add conditional editor, action-header, compact list, focus, and responsive styling with existing tokens in `src/styles.css`

**Checkpoint**: No fields are rendered on initial load; add/edit is keyboard accessible and save/cancel returns to the list.

---

## Phase 5: User Story 4 - Copy an Existing Configuration (Priority: P2)

**Goal**: Copy every connection value, including the non-disclosed key, into a uniquely named inactive saved configuration with one action

**Independent Test**: Copy a current keyed configuration repeatedly; each sibling has identical connection data, a unique valid name, and does not replace the current selection.

### Tests for User Story 4

- [x] T017 [US4] Add failing backend value-preservation/name-boundary/current-selection and frontend copy-action regressions in `src-tauri/tests/model_config.rs` and `src/views/Settings.model.test.tsx`

### Implementation for User Story 4

- [x] T018 [US4] Implement bounded unique-name model duplication and expose the Tauri command in `src-tauri/src/settings.rs`, `src-tauri/src/commands.rs`, and `src-tauri/src/lib.rs`
- [x] T019 [US4] Add the duplicate IPC wrapper and configuration-card copy interaction in `src/ipc.ts` and `src/views/Settings.tsx`

**Checkpoint**: Copy is one click, never sends the stored key to the webview, and leaves the source/current selection unchanged.

---

## Phase 6: User Story 5 - Reduce Redundant Page Subtitles (Priority: P2)

**Goal**: Remove generic page-title descriptions throughout the app while retaining contextual guidance needed for action, safety, privacy, or recovery

**Independent Test**: Navigate every primary page at 1024×720 and 720×520; no generic title subtitle remains, and all necessary contextual/dynamic guidance is still present.

### Tests for User Story 5

- [x] T020 [US5] Add page-header copy and necessary-context regression assertions in `src/views/SettingsShell.test.tsx`, `src/views/Settings.model.test.tsx`, and `src/views/Prompts.test.tsx`

### Implementation for User Story 5

- [x] T021 [US5] Remove generic header subtitles and relocate prompt/model privacy guidance contextually in `src/views/SettingsShell.tsx`, `src/views/Settings.tsx`, `src/views/Prompts.tsx`, `src/views/History.tsx`, and `src/views/Onboarding.tsx`
- [x] T022 [US5] Update About privacy copy for same-row plain-text keys and tighten single-line header spacing in `src/views/SettingsShell.tsx` and `src/styles.css`

**Checkpoint**: The interface has a quieter title hierarchy without losing validation, fees, privacy, permission, state, or recovery information.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Complete security, accessibility, validation, release, and documentation evidence

- [x] T023 [P] Update redaction/storage security regressions and confirm no key appears in history or diagnostics in `src-tauri/tests/model_config.rs`, `src-tauri/tests/storage_foundation.rs`, and `src-tauri/tests/foundation.rs`
- [x] T024 Run Prettier, ESLint, Vitest, frontend build, Rust tests, and browser smoke validation from `specs/004-streamlined-model-config/quickstart.md`
- [x] T025 Run `npm run tauri build`, install the macOS bundle locally, and record native 1024×720 and 720×520 model/add/edit/empty plus all-page subtitle review in `specs/004-streamlined-model-config/validation.md`
- [x] T026 Mark all completed tasks and record final verification evidence in `specs/004-streamlined-model-config/tasks.md` and `specs/004-streamlined-model-config/validation.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **US2 storage (Phase 2)**: Depends on setup and establishes the key contract.
- **US3 transient tests (Phase 3)**: Depends on US2 for draft-ID key resolution.
- **US1 conditional editor (Phase 4)**: Depends on the US2/US3 IPC behavior.
- **US4 copy (Phase 5)**: Depends on US2 storage; its UI integrates with the US1 list-first page.
- **US5 subtitle cleanup (Phase 6)**: Independent after setup, but is sequenced after model UI changes to avoid overlapping edits.
- **Polish (Phase 7)**: Depends on all user stories.

### Within Each User Story

- Regression tests are written before the implementation they specify and must fail for the missing behavior.
- Schema precedes storage services; storage precedes command wiring; backend contracts precede frontend integration.
- A story reaches its checkpoint only after its focused tests pass.

### Parallel Opportunities

- T002 can run independently from version synchronization.
- T014 affects the browser test while T013 affects the component test.
- T023 can be prepared independently after the storage contract stabilizes.
- US5 view-audit assertions can be drafted while Rust-only US2/US3 work is in progress, but final edits touching `Settings.tsx` remain sequential.

## Parallel Example: User Story 1

```text
Task T013: Add component-level editor state-machine regressions in src/views/Settings.model.test.tsx
Task T014: Add browser smoke coverage in tests/e2e/primary-flow.mjs
```

## Implementation Strategy

### MVP First

1. Complete setup and same-row key storage/migration.
2. Remove test persistence and activation gating.
3. Deliver the list-first conditional editor.
4. Validate save-without-test and draft-test-without-save end to end.

### Incremental Delivery

1. Storage and migration make existing configurations reliable.
2. Transient testing removes the lifecycle coupling.
3. Conditional editing removes default form clutter.
4. Copying speeds variant creation.
5. App-wide subtitle cleanup completes the visual hierarchy pass.

## Notes

- No new dependency is authorized or required.
- Saved API keys must never be returned in summaries, notifications, logs, history, accessibility labels, or copy UI payloads.
- The keyring dependency remains for one-way migration compatibility only.
- Manual Windows review may be recorded as pending if no Windows host is available; macOS review and installation are required in this environment.
