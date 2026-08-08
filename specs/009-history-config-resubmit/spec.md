# Feature Specification: History Configuration Resubmit

**Feature Branch**: `master`

**Created**: 2026-08-08

**Status**: Draft

**Input**: User description: "在历史详情界面，将‘使用当前配置再次提交’改为‘重新选择配置提交’，允许选择模型和提示词配置，默认选中原配置；原配置被修改后使用修改后的内容；选择不影响全局配置。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Choose Configurations for Resubmission (Priority: P1)

As a user reviewing a history entry, I can choose a model configuration and a prompt configuration before resubmitting the saved screenshot so that I can intentionally rerun it with a different combination.

**Why this priority**: Choosing both configurations is the primary requested capability and removes the current dependency on whichever global configuration happens to be active.

**Independent Test**: Open a history detail, choose a non-default model and prompt, submit, and verify that the new analysis uses those selections while the global active model and prompt remain unchanged.

**Acceptance Scenarios**:

1. **Given** a history entry with an image and at least one available model and prompt, **When** the user opens its detail, **Then** the detail presents keyboard-accessible model and prompt selectors and an action labeled "重新选择配置提交".
2. **Given** the user selects a model and prompt, **When** the user activates "重新选择配置提交", **Then** the saved image is submitted with the selected configurations' current values.
3. **Given** a resubmission succeeds or fails, **When** the operation finishes, **Then** the existing notification pattern reports the outcome and the global active configuration values are unchanged.

---

### User Story 2 - Default to the Original Configuration Identities (Priority: P2)

As a user rerunning a history entry, I see its original model and prompt selected by default so that repeating the prior intent requires no extra selection, while edits made to those configurations since the original run are respected.

**Why this priority**: A useful default makes the flow fast and ensures configuration maintenance applies to future reruns without retaining obsolete configuration contents.

**Independent Test**: Save a history entry, edit the same model or prompt configuration, open the entry, and verify that the same configuration identities are selected and their updated values are used for resubmission.

**Acceptance Scenarios**:

1. **Given** the original model and prompt still exist, **When** the history detail loads, **Then** both original configuration identities are selected by default regardless of edits to their names or contents.
2. **Given** a legacy history entry can be matched to current configurations by its saved names, **When** the detail loads, **Then** those matching configurations are selected by default.
3. **Given** an original configuration no longer exists or cannot be matched, **When** the detail loads, **Then** the active configuration of that type is selected, falling back to the first available option when necessary.

### Edge Cases

- The history entry has no saved image.
- No model configurations or no prompt configurations are available.
- The original configuration was renamed, edited, or deleted after the history entry was created.
- Loading the configuration choices fails while the history detail itself remains available.
- A selected configuration is deleted between loading the detail and submitting it.
- Another analysis or screenshot capture is already running.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The history detail action MUST be labeled "重新选择配置提交".
- **FR-002**: The history detail MUST allow the user to select one available model configuration and one available prompt configuration before resubmission.
- **FR-003**: New history entries MUST retain the identities of the model and prompt used for the run in addition to the existing display snapshots.
- **FR-004**: If the retained configuration identities still exist, the selectors MUST default to those identities.
- **FR-005**: Resubmission MUST load the selected configurations' current values at submission time rather than reuse the historical snapshot contents.
- **FR-006**: Selecting configurations for history resubmission MUST NOT modify either global active configuration.
- **FR-007**: Existing history entries without retained identities SHOULD be matched to current configurations by saved names; unmatched entries MUST fall back to the active configuration and then the first available option.
- **FR-008**: Resubmission MUST be disabled when the history image is unavailable, configuration choices are loading, or either required configuration type has no selection.
- **FR-009**: Configuration-loading and submission failures MUST use the existing error notification and recovery patterns.
- **FR-010**: A resubmission that is saved to history MUST retain the identities and display snapshots of the configurations selected for that resubmission.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: The selectors MUST define loading, ready, empty, disabled, success, and error behavior without hiding the existing history result or failure details.
- **UX-002**: The feature MUST reuse existing form controls, button styling, notifications, and history-detail navigation patterns.
- **UI-001**: Existing design tokens and shared button components MUST be reused; no new visual system or dependency is permitted.
- **UI-002**: Both selectors and the submit action MUST have visible labels, keyboard access, clear disabled states, and a logical focus order.
- **UI-003**: Human visual review MUST cover approximately 1094×768, 780×800, and 540×800 detail views, checking selector wrapping, labels, spacing, button states, and absence of horizontal overflow.

### Key Entities

- **History Entry Configuration Reference**: The model configuration identity and prompt configuration identity used by a saved analysis, alongside the existing historical display snapshots.
- **Resubmission Selection**: The model and prompt identities selected locally in one history detail view; it is transient and does not change global settings.
- **Current Configuration**: The latest saved values belonging to a selected model or prompt identity at resubmission time.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of automated selection-flow tests, a user can choose any available model/prompt pair and the submitted request contains exactly those two identities.
- **SC-002**: In 100% of tests where original configurations still exist, the selectors default to their identities after those configurations' contents or names have changed.
- **SC-003**: In 100% of resubmission tests, global active configuration identities are identical before and after submission.
- **SC-004**: All representative viewport reviews show readable selector labels and options with no horizontal page overflow.
- **SC-005**: Existing history navigation, copy, deletion, filtering, and result-display tests continue to pass.

## Assumptions

- A configuration's stable identity, not its historical content snapshot or name, defines "原配置".
- Historical prompt/model snapshots remain for display and audit readability; they are not used to execute a new submission.
- For older rows that predate configuration identities, exact name matching is an acceptable best-effort migration and runtime fallback.
- If the original configuration is unavailable, choosing the current global active option as a default is a convenience only and does not modify global settings.
- Editing configurations from the history detail is outside scope; users choose from configurations managed in the existing settings screens.
