# Feature Specification: Per-prompt screenshot shortcuts

**Feature Branch**: `master`
**Created**: 2026-09-06
**Status**: Draft

## User Scenarios & Testing

### User Story 1 - Configure prompts and shortcuts (Priority: P1)
Users can add or edit a prompt on a dedicated page and assign one screenshot shortcut per prompt.

**Acceptance Scenarios**:
1. Given the prompt list, when the user clicks add or edit, then a separate editor page opens with Save and Cancel.
2. Given a prompt without a shortcut, when the shortcut box is clicked and a valid combination is pressed, then it is saved and shown on that prompt.
3. Given a duplicate shortcut, when saving, then the change is rejected with an error and the previous shortcut remains.

### User Story 2 - Capture with the selected prompt (Priority: P1)
When a configured prompt shortcut is pressed, the app starts capture using that prompt; no global shortcut, current prompt, or tray capture action is required.

**Acceptance Scenarios**:
1. Given a prompt shortcut, when it is pressed, then capture starts with that prompt's text snapshot.
2. Given the tray menu, then it contains only opening the app and quitting.

## Requirements

- **FR-001**: Each prompt MUST persist zero or one screenshot shortcut.
- **FR-002**: The prompt list MUST show Edit, Clone, Delete on the first row and shortcut setting on the second row with aligned widths.
- **FR-003**: The shortcut control MUST show a light “点击设置快捷键” placeholder when unset.
- **FR-004**: The app MUST reject shortcut conflicts and preserve the prior value.
- **FR-005**: The app MUST remove the global screenshot shortcut, current prompt concept, and tray “开始截图” entry.
- **FR-006**: Capture MUST use the prompt associated with the pressed shortcut.

## User Experience and UI Requirements

- Reuse existing Button, Field, notification, and shortcut recording patterns.
- Support Escape to cancel recording and accessible labels for controls.
- Provide loading, empty, saving, error, and recovery states.

## Success Criteria

- **SC-001**: A user can create a prompt and assign its shortcut without leaving the prompt settings flow.
- **SC-002**: Pressing a prompt shortcut starts capture with the matching prompt in one attempt.
- **SC-003**: No global shortcut or tray capture entry is visible or registered.

## Assumptions

- Existing shortcut format and conflict handling remain valid.
- Cloned prompts start without a shortcut.
