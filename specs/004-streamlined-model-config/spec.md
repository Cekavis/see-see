# Feature Specification: Streamlined Model Configuration

**Feature Branch**: `master`

**Created**: 2026-07-25

**Status**: Draft

**Input**: User description: "Reduce excessive subtitles throughout the app; store API keys in plain text with endpoints; separate connection testing from saving; hide the model form until adding or editing; and allow copying an existing model configuration."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Manage configurations without form clutter (Priority: P1)

A user opens the model page and sees saved configurations first. The configuration form appears only after the user chooses to add or edit a configuration, and disappears again after a successful save or cancellation.

**Why this priority**: This directly removes the persistent form that dominates the page and makes the most common task—choosing an existing configuration—easier to scan.

**Independent Test**: Open the model page with and without saved configurations, add a configuration, edit one, cancel, and save; verify the form is visible only during an explicit add or edit flow.

**Acceptance Scenarios**:

1. **Given** the model page has loaded, **When** the user has not selected add or edit, **Then** no configuration input form is displayed and a clear “新增” action is available.
2. **Given** the user clicks “新增”, **When** the form opens, **Then** it contains empty/default values and supports saving without a prior connection test.
3. **Given** the user edits a saved configuration, **When** saving succeeds or the user cancels, **Then** the form closes and the saved list remains visible.
4. **Given** a save fails, **When** the error is shown, **Then** the form remains open with the user's input intact so they can recover.

---

### User Story 2 - Save complete reusable connection details (Priority: P1)

A user saves an endpoint and its API key as one local model configuration. When they later edit, copy, test, or use that configuration, the saved key remains available without being exposed outside the configuration boundary.

**Why this priority**: The requested storage model removes dependence on a separate credential record and makes the configuration self-contained.

**Independent Test**: Save, reload, edit, copy, test, activate, and use a configuration containing a key; verify all operations use the same saved endpoint and key while logs and history continue to omit the key.

**Acceptance Scenarios**:

1. **Given** a user enters an endpoint and API key, **When** the configuration is saved, **Then** both values are retained together in local application storage and leaving the key field blank during a later edit preserves the saved key.
2. **Given** an existing configuration previously used separate credential storage, **When** the upgraded app starts, **Then** it migrates any readable key into the configuration without losing the endpoint.
3. **Given** a configuration contains a key, **When** logs, notifications, history, or other non-configuration views are produced, **Then** the key is not included.

---

### User Story 3 - Test independently from saving (Priority: P1)

A user can test the values currently shown in the form to detect incorrect connection information. Testing does not save, activate, or attach a status to the configuration, and saving never requires testing.

**Why this priority**: Connection testing is a diagnostic tool, not a lifecycle gate or persistent property of a model configuration.

**Independent Test**: Test unsaved and edited draft values for success and failure, then verify no configuration, activation, or test-result data changed; separately save and activate an untested configuration.

**Acceptance Scenarios**:

1. **Given** the user has entered unsaved connection values, **When** they click “测试连接”, **Then** those draft values are tested directly and no configuration is created or changed.
2. **Given** a test succeeds or fails, **When** the result is presented, **Then** it is transient feedback only and is absent after the page reloads.
3. **Given** a saved configuration has never been tested, **When** the user chooses “设为当前”, **Then** activation succeeds.
4. **Given** the user does not run a test, **When** they save valid fields, **Then** saving succeeds.

---

### User Story 4 - Copy an existing configuration (Priority: P2)

A user can create a saved copy of an existing model configuration, including its endpoint, model selection, protocol, and API key, while leaving the original and current selection unchanged.

**Why this priority**: Copying reduces repetitive data entry when creating variants for nearby models or endpoints.

**Independent Test**: Copy a configuration several times and verify each copy has a unique name, identical connection values, no active status, and can be edited independently.

**Acceptance Scenarios**:

1. **Given** a saved configuration, **When** the user clicks “复制”, **Then** a new saved configuration appears with a unique copy name and identical connection fields.
2. **Given** the original is current, **When** it is copied, **Then** the original remains current and the copy is not automatically activated.

---

### User Story 5 - Reduce redundant page subtitles (Priority: P2)

A user navigates through the entire app without seeing repeated explanatory subtitle copy beneath self-explanatory page titles. Necessary instructions remain next to the control or state they clarify.

**Why this priority**: A quieter visual hierarchy improves scanning and reduces the sense of dense, repetitive interface copy.

**Independent Test**: Review every app screen at representative desktop and narrow widths; verify page headers contain concise titles, redundant subtitles are removed, and task-critical guidance remains contextual.

**Acceptance Scenarios**:

1. **Given** any primary settings page, **When** it is displayed, **Then** its top header does not include a generic explanatory subtitle.
2. **Given** guidance is required to avoid an error, fee, privacy misunderstanding, or irreversible action, **When** the related control or state appears, **Then** that guidance remains adjacent to it.
3. **Given** the app is displayed at 1024×720 or 720×520, **When** the user navigates every section, **Then** titles and controls remain visually clear without unused subtitle space.

### Edge Cases

- The user starts adding or editing, then cancels after entering values; no draft values are persisted.
- A connection test is in progress while the user attempts to save, cancel, edit, or start another test; conflicting actions remain disabled until the test finishes.
- Copying a configuration repeatedly continues to produce a valid unique name within the name-length limit.
- A legacy credential reference exists but the system credential is unavailable; the configuration remains usable as a keyless configuration and can be updated normally.
- A key contains leading, trailing, or internal whitespace; its exact value is preserved rather than normalized.
- A saved configuration is deleted while it is currently being edited; the form closes after successful deletion.
- Empty-state guidance remains useful after removal of page-level subtitles.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The model page MUST hide its configuration form until the user explicitly starts adding or editing.
- **FR-002**: The model page MUST provide a prominent “新增” action in both empty and populated states.
- **FR-003**: A successful save or explicit cancel MUST close the configuration form; a failed save MUST keep it open with entered values intact.
- **FR-004**: Saving a valid model configuration MUST be allowed without running a connection test.
- **FR-005**: Connection testing MUST operate on the current draft values without saving, activating, or otherwise mutating any model configuration.
- **FR-006**: Connection test results MUST be transient and MUST NOT be recorded as configuration data.
- **FR-007**: Any saved configuration MUST be eligible to become current regardless of whether it has been tested.
- **FR-008**: The API key MUST be stored as plain text in the same local configuration record as the endpoint; editing without a replacement MUST preserve it, and users MUST be able to explicitly clear it.
- **FR-009**: Existing separately stored API keys MUST be migrated when they are readable, and unavailable legacy keys MUST NOT prevent startup or access to other configuration fields.
- **FR-010**: API keys MUST remain excluded from logs, notifications, history records, and non-configuration views.
- **FR-011**: Users MUST be able to copy a saved model configuration; the copy MUST include all connection values, have a unique name, remain inactive, and be independently editable.
- **FR-012**: Deleting a configuration MUST delete its locally stored plain-text API key with the configuration record.
- **FR-013**: Primary page headers throughout the app MUST omit generic explanatory subtitles that repeat the page purpose.
- **FR-014**: Safety-, cost-, privacy-, recovery-, and input-specific guidance MUST remain available beside the relevant control or state.
- **FR-015**: Existing endpoint validation, model-list retrieval, deletion confirmation, notification feedback, and key redaction behavior MUST continue to work.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: Loading, empty, add, edit, testing, saving, deleting, success, error, disabled, cancel, and recovery states MUST be defined and consistently surfaced.
- **UX-002**: Buttons, cards, fields, confirmations, and notifications MUST reuse the app's existing interaction, language, and feedback patterns.
- **UI-001**: The design MUST reuse existing spacing, color, typography, card, field, and button tokens; no new visual system or dependency is required.
- **UI-002**: Add, edit, copy, test, save, cancel, activate, and delete actions MUST be keyboard accessible, have clear accessible names, and remain usable at the app's 720×520 minimum size.
- **UI-003**: Human visual review MUST cover the model page and every other primary page at 1024×720 and 720×520, checking reduced subtitle density, hierarchy, spacing, wrapping, focus visibility, empty state, and edit state.

### Key Entities *(include if feature involves data)*

- **Model Configuration**: A named local configuration containing protocol, endpoint, model ID, optional plain-text API key, timestamps, and whether it is current; list summaries disclose only whether a key exists.
- **Connection Test Result**: A transient success or failure response, including latency and an optional categorized error; it has no persisted relationship to a model configuration.
- **Configuration Draft**: Unsaved form values used for add, edit, model listing, and connection testing; it exists only while the form is open.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On initial model-page load, zero configuration input fields are visible until the user selects “新增” or “编辑”.
- **SC-002**: Users can save and activate a valid configuration without testing in one add flow, and can test a draft without increasing or changing the saved-configuration count.
- **SC-003**: Across save, reload, edit, copy, and use flows, 100% of configured endpoints and API keys retain their entered values.
- **SC-004**: A user can create an editable copy of a saved configuration with one action, and ten consecutive copies receive unique valid names.
- **SC-005**: Every primary page passes visual review at 1024×720 and 720×520 with no generic page subtitle remaining and no loss of task-critical guidance.
- **SC-006**: All model-configuration, activation, migration, redaction, and interface regression tests pass with no new lint, formatting, build, or accessibility failures.

## Assumptions

- The user explicitly accepts the reduced security of storing API keys unencrypted in the app's local database; the app will not imply that the keys are protected by the operating-system credential store.
- “Together with the endpoint” means the API key is a field of the same local model-configuration record, not a separate referenced secret. It does not require returning the saved key to the webview after save.
- “Copy” means immediately creating a saved sibling configuration with a generated unique name, matching the existing prompt-copy interaction.
- Subtitle cleanup applies to generic descriptive copy directly beneath primary page titles; contextual field hints, state descriptions, privacy notes, test cost warnings, and confirmation text remain when useful.
- The existing local database remains the source of truth, and no cloud synchronization is introduced.
