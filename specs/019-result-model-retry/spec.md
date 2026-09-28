# Feature Specification: Result Model Retry

**Feature Branch**: `master`

**Created**: 2026-09-28

**Status**: Implemented; validation and platform limitations recorded

**Input**: 在结果窗口底部增加使用其他模型重试的按钮；点击选择模型，选完直接弹出新的结果窗口，并考虑模型较多的情况。

## User Scenarios & Testing

### User Story 1 - Retry in another result window (Priority: P1)

A user opens the model chooser from a result window footer, chooses a saved model configuration, and immediately gets an independent result window for the same screenshot and prompt.

**Independent Test**: Choose another model from a completed or failed result; observe a new result while the source result stays available.

**Acceptance Scenarios**:

1. Given a result in any analysis state, when a model is selected, a separate result starts without an extra confirmation step.
2. Given the original prompt was edited or deleted after submission, the new run still uses the original prompt text and image.
3. Given multiple result windows, retrying or closing one does not reset, cancel, or overwrite another.
4. Given a selected model was deleted or the new window cannot open, the chooser reports the failure and allows another attempt.

### User Story 2 - Find a model in a long list (Priority: P2)

A user can select a configuration by its saved name without scrolling the result window itself.

**Independent Test**: Load 500 configurations, navigate to an item near the end, and select it using the keyboard.

**Acceptance Scenarios**:

1. Given 500 saved configurations, the list scrolls locally without additional network requests or extra search/count UI.
2. Given long names or a small window, rows remain readable and the bounded list scrolls within the chooser.
3. Given no configurations or a loading failure, the chooser gives explicit feedback and an available recovery action; loading and creation have no progress copy.
4. Given keyboard navigation, focus moves to the first loaded model, stays inside the open modal, and returns to its trigger when dismissed.

### Edge Cases

- Loading completes after the chooser closes or reopens.
- A user selects repeatedly while the new run is being created.
- The source is still streaming, failed, cancelled, or not saved to history.
- Configurations share a model identifier but have different names or endpoints.
- The source configuration has been renamed or deleted.

## Requirements

### Functional Requirements

- **FR-001**: A footer action opens a chooser of saved model configurations.
- **FR-002**: Selection immediately starts a new result window; no extra submit action is required.
- **FR-003**: The new run preserves the source screenshot, original prompt, and history-saving preference, and uses the selected model configuration with its current credentials.
- **FR-004**: The original result, any in-flight request, and the globally selected model remain unchanged.
- **FR-005**: Show saved configuration names in the existing order, without a search box or model count.
- **FR-006**: The chooser bounds its height and scrolls the model list, supporting at least 500 configurations.
- **FR-007**: Empty and failure states are explicit; loading and submission show no progress copy. One pending selection starts at most one new run.
- **FR-008**: The existing same-model failure retry continues to work as before.
- **FR-009**: Selecting a model on Windows creates a responsive result window without hanging.
- **FR-010**: Development mode must not duplicate streamed text or thinking content; legitimate repeated content is preserved.

### User Experience and UI Requirements

- **UX-001**: Use concise Chinese labels, existing error feedback, and dismissible modal behavior.
- **UI-001**: Reuse existing Button, dialog surface, spacing, color, and focus patterns; no new dependency.
- **UI-002**: The model list has an accessible label; buttons support keyboard activation; Escape dismisses and restores focus; list navigation supports arrows and Home/End.
- **UI-003**: Review screenshots at 460×500 and 420×360 with long names, many models, loading/error/empty states, and a wrapping footer.
- **UI-004**: Each compact model row shows only the user-defined configuration name. Keep one line with ellipsis for overflow and expose the full name on hover.

### Key Entities

- **Source analysis**: Original screenshot, frozen prompt, model metadata, history policy, and independent run state.
- **Model configuration**: Saved identity, display name, model identifier, protocol, and backend-only credentials.
- **New analysis**: Unique result identity with its own output, cancellation, and history record.

## Success Criteria

- **SC-001**: A new result starts after exactly two selections: the footer action and a model row.
- **SC-002**: A target among 500 configurations can be reached by scrolling or keyboard navigation and activated with the keyboard.
- **SC-003**: Both representative window sizes keep dismissal controls visible without horizontal page overflow.
- **SC-004**: Regression checks demonstrate source isolation, prompt preservation, and duplicate-submit prevention.

## Assumptions

- “Models” means saved model configurations; adding new configurations remains in Settings.
- All saved configurations remain selectable, including the original one if desired; identifiers, not display names, select the target.
- No history record is required because open result windows retain their original request.
- This compatible feature increments 0.16.1 to 0.17.0 once, with a local build/install and atomic commit/push after validation.
