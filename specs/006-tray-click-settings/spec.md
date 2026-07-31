# Feature Specification: Tray Click Settings

**Feature Branch**: `master`

**Created**: 2026-07-31

**Status**: Draft

**Input**: User description: "左键托盘图标时直接打开设置窗口，右键时才显示菜单"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open Settings from Tray (Priority: P1)

As a user, I can left-click the tray icon to open the existing settings window immediately, while right-clicking remains the way to open the tray menu.

**Why this priority**: It makes the most common tray action direct without removing access to capture, open, and quit menu actions.

**Independent Test**: With the app running in the tray and the settings window hidden, left-click the tray icon and verify the settings window appears without a menu; then right-click and verify the menu appears without changing the window behavior.

**Acceptance Scenarios**:

1. **Given** the app is running and the settings window is hidden, **When** the user left-clicks the tray icon, **Then** the settings window is restored, shown, and focused without displaying the tray menu.
2. **Given** the app is running, **When** the user right-clicks the tray icon, **Then** the existing tray menu is displayed.
3. **Given** the settings window is already visible or minimized, **When** the user left-clicks the tray icon, **Then** the same settings window is restored and focused rather than duplicated.

### Edge Cases

- A press and release sequence must open the settings window only once.
- Unsupported tray events, including movement and double-click events, must not open the settings window.
- Existing menu actions for capture, opening See See, and quitting must continue to work.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST open the existing settings window when the tray icon receives a completed primary-button click.
- **FR-002**: The system MUST NOT display the tray menu for a primary-button click.
- **FR-003**: The system MUST continue to display the tray menu for a secondary-button click.
- **FR-004**: Opening settings from the tray MUST restore, show, and focus the existing settings window without creating another window.
- **FR-005**: Existing tray menu actions MUST remain available and unchanged.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: The left-click action must provide immediate success by bringing the settings window to the foreground; no new loading, empty, disabled, error, or recovery state is required.
- **UX-002**: The change must reuse the existing settings-window opening behavior and existing tray menu labels.
- **UI-001**: No new visual components or styles may be introduced.
- **UI-002**: Existing keyboard behavior and accessibility of the settings window and tray menu must remain unchanged.
- **UI-003**: Human review must confirm the interaction on a representative desktop environment; no viewport-specific visual change is expected.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A single left-click opens and focuses the settings window in 100% of manual verification attempts.
- **SC-002**: A left-click displays the tray menu in 0% of manual verification attempts.
- **SC-003**: A right-click displays the existing tray menu in 100% of manual verification attempts.
- **SC-004**: All existing automated tests and the new tray-click regression test pass.

## Assumptions

- The existing main window is the settings window requested by the user.
- Platform tray implementations retain their standard secondary-click menu behavior when primary-click menu display is disabled.
- Double-click behavior is outside this change and remains unchanged.
