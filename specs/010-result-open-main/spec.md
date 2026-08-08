# Feature Specification: Result Window Main Navigation

**Feature Branch**: `master`

**Created**: 2026-08-08

**Status**: Draft

**Input**: User description: "请你在识别结果窗口增加一个打开主窗口按钮，点击之后打开主窗口，如果模型回答已经结束则顺便关闭结果窗口"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open the Main Window from Results (Priority: P1)

As a user viewing a recognition result, I can open the main window directly so that I can return to settings or history without using the tray or taskbar.

**Why this priority**: Direct navigation removes an unnecessary desktop detour while preserving the result when the model is still working.

**Independent Test**: Open a result window during streaming and after a terminal outcome, activate the new button, and verify that the main window opens in both cases while only the terminal result window closes.

**Acceptance Scenarios**:

1. **Given** a result window in any analysis state, **When** the user activates "打开主窗口", **Then** the existing main window is shown, restored if minimized, and focused.
2. **Given** the model is still submitting or streaming, **When** the user opens the main window, **Then** the result window remains open and continues receiving output.
3. **Given** the analysis is completed, failed, or cancelled, **When** the user opens the main window, **Then** the result window closes and its finished runtime state is cleaned up.
4. **Given** the action cannot complete, **When** the operation fails, **Then** the result window remains usable and the existing notification pattern reports the error.

### Edge Cases

- The main window is hidden, minimized, already visible, or already focused.
- The analysis changes from active to terminal while the button action is being handled.
- The result window belongs to a stale or missing analysis run.
- The result window is configured to stay always on top.
- The result window is at its compact minimum size.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The recognition result footer MUST include a button labeled "打开主窗口".
- **FR-002**: Activating the button MUST show, restore, and focus the existing main window.
- **FR-003**: An active submitting or streaming result MUST remain open after the main window opens.
- **FR-004**: A completed, failed, or cancelled result MUST close after the main window opens.
- **FR-005**: The close decision MUST use the authoritative analysis state at action time so a stale frontend snapshot cannot cancel active work.
- **FR-006**: Closing a terminal result through this action MUST release its retained runtime analysis state.
- **FR-007**: Action failures MUST use the existing result-window notification pattern without closing the result window.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: The action MUST remain available during loading, streaming, success, failure, and cancellation states.
- **UX-002**: The feature MUST reuse the existing result footer, shared Button component, main-window behavior, and notification language patterns.
- **UI-001**: Existing design tokens and responsive footer behavior MUST be reused; no new visual system or dependency is permitted.
- **UI-002**: The button MUST be keyboard accessible, expose its visible label as its accessible name, and retain a visible focus state.
- **UI-003**: Human visual review MUST cover the normal result-window size and the 420×360 minimum, checking footer reachability, wrapping, spacing, and absence of horizontal overflow.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of automated result-control tests, activating "打开主窗口" invokes the navigation action once.
- **SC-002**: In 100% of state-decision tests, submitting and streaming results remain open while completed, failed, and cancelled results are eligible to close.
- **SC-003**: Existing cancel, retry, copy, always-on-top, and streaming-state tests continue to pass.
- **SC-004**: At both the normal result-window size and 420×360 minimum, every footer action remains keyboard reachable with no horizontal page overflow.

## Assumptions

- "模型回答已经结束" means any terminal analysis outcome: completed, failed, or cancelled.
- The existing `main` window is the requested main window.
- Opening the main window does not change the active settings section.
- The result window remains the owner of ongoing output and must not be closed while the analysis is active.
