# Feature Specification: Remember Result Window Position

**Feature Branch**: `master`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "如果一个结果窗口被拖动位置，之后新的结果窗口都采用这个新位置"

## User Scenarios & Testing

### User Story 1 - Reuse the latest result window position (Priority: P1)

Users can move a result window to a convenient location and have subsequently created result windows open at that same location during the current application session.

**Why this priority**: Reusing a user's chosen location removes repeated window rearrangement and is the only behavior required to deliver this feature's value.

**Independent Test**: Open a result window, drag it to a known location, start another analysis, and verify that the new result window opens at the moved window's top-left location while retaining its existing size and content behavior.

**Acceptance Scenarios**:

1. **Given** no result window position has been remembered, **When** a result window is first shown, **Then** it opens using the existing centered placement.
2. **Given** a result window is moved to a new location, **When** a later result window is created, **Then** the later window opens at the most recently moved location.
3. **Given** multiple result windows are open, **When** any one of them is moved, **Then** only the most recently reported result-window location is used for the next newly created result window; existing windows are not repositioned.

### Edge Cases

- Closing the result window that established the remembered location does not clear that location for later windows in the same session.
- If the operating system adjusts a remembered location because a display or work area is unavailable, the next move event updates the remembered location to the adjusted position.
- Creating additional result windows without manually moving any window preserves the existing initial centering behavior when no remembered location exists.
- Window size, always-on-top state, focus behavior, and result-window content remain unchanged.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST record the latest location reported for any result window when that window changes position.
- **FR-002**: The system MUST use the recorded location when creating each subsequent result window in the current application session.
- **FR-003**: When no result window location has been recorded, the system MUST retain the existing centered placement.
- **FR-004**: Updating the remembered location MUST NOT move or resize result windows that are already open.
- **FR-005**: The feature MUST preserve the existing result-window size, minimum size, focus, always-on-top, close, and analysis behaviors.

### User Experience and UI Requirements

- **UX-001**: No new control or setting is required; moving a result window is the only user action needed to change the default location for later result windows.
- **UX-002**: Existing result-window loading, streaming, completed, failed, cancelled, retry, close, and keyboard interaction states MUST remain unchanged.
- **UI-001**: The feature MUST reuse the existing compact result-window layout and native window behavior; no new visual pattern is introduced.
- **UI-002**: Existing keyboard and accessibility behavior MUST remain available after a result window opens at a remembered location.
- **UI-003**: Human visual review MUST cover a representative compact result window at the existing 420×360 minimum and 460×500 default sizes on supported desktop platforms, including first-open centered placement and a later-open remembered placement.

### Key Entities

- **Remembered result-window location**: The latest screen location associated with any result window during the current application session; it is shared by subsequently created result windows and is not persisted across application restarts in this feature.

## Success Criteria

### Measurable Outcomes

- **SC-001**: After a user moves a result window, 100% of subsequently created result windows in the same session open at the latest recorded location, subject to operating-system work-area adjustments.
- **SC-002**: When no result window has been moved, the first result window retains its existing centered placement in 100% of tested launches.
- **SC-003**: Creating or moving a result window does not change the position, size, content, or interaction state of any other already open result window in the regression test scenario.
- **SC-004**: The feature requires no additional user configuration or repeated repositioning after the first manual move in a session.

## Assumptions

- The latest position is shared across all result windows, regardless of which analysis run created them.
- The remembered position is scoped to the current application session; restoring it after an application restart is out of scope for this feature.
- The operating system remains responsible for clamping or adjusting positions that are outside an available display work area.
- Existing result-window creation paths, including fresh captures and history resubmissions, use the same placement behavior.
