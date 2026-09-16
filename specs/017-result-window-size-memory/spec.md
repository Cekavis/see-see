# Feature Specification: Result Window Size Memory

**Feature Branch**: `master`

**Created**: 2026-09-16

**Status**: Ready for planning

**Input**: User description: "1. 结果窗口改成默认 460x540 吧，然后用户如果调整了结果窗口，要持久化地记住，下次跳出时需要沿用用户的调整。重启也要记住 2. 截图预览最大高度改成 60px，历史页的截图也一样"

## User Scenarios & Testing

### User Story 1 - Reuse a preferred result-window size (Priority: P1)

After resizing a result window, a user sees later result windows at that same size. The preference remains after the application restarts, while the window continues to honor the supported minimum size.

**Why this priority**: The user explicitly requests durable control over the result window's dimensions, and it must work for every future capture rather than only while the app remains open.

**Independent Test**: Resize a result window above its minimum dimensions, close it, open another result, restart the application, and open a third result. Each later result opens at the chosen dimensions.

**Acceptance Scenarios**:

1. **Given** no result-window size has been saved, **When** a result window opens, **Then** its content area starts at 460 by 540 pixels and cannot be resized below the existing supported minimum dimensions.
2. **Given** the user resizes a result window, **When** another result window opens during the same application session, **Then** it uses the most recently chosen size.
3. **Given** the user has resized a result window and restarted the application, **When** another result window opens, **Then** it uses the saved size.
4. **Given** an old installation has no saved size, **When** it upgrades, **Then** it opens result windows at the 460 by 540 default without losing existing settings.

---

### User Story 2 - Keep screenshot previews compact (Priority: P2)

When comparing a result or reviewing history, the user sees the original screenshot at a maximum display height of 60 pixels so the text and actions remain prominent.

**Why this priority**: A compact, consistent preview improves comparison while preserving vertical space in the small result window and the history page.

**Independent Test**: Open a result and a history entry with tall and wide screenshots. Confirm each preview preserves its aspect ratio, stays within 60 pixels high, and does not cause horizontal overflow.

**Acceptance Scenarios**:

1. **Given** a result window has an original screenshot, **When** it displays the preview, **Then** the preview is no taller than 60 pixels and keeps its aspect ratio.
2. **Given** a history list or detail view has an original screenshot, **When** it displays the preview, **Then** the preview is no taller than 60 pixels and keeps its aspect ratio.
3. **Given** a screenshot is wider than its container, **When** it is displayed in either location, **Then** it scales to fit without horizontal overflow.

## Edge Cases

- A resize event that reports a size below the result window's minimum must not replace a valid saved preference with an unusable size.
- Failure to write the size preference must not prevent resizing, displaying, or closing a result window; it is recorded through the existing application logging pattern.
- A saved size is applied only to result windows and does not change main, capture, settings, or history-window dimensions.
- Image retrieval remains non-blocking: an unavailable screenshot does not prevent result or history content from displaying.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST open a result window at 460 by 540 pixels when no user-selected result-window size exists.
- **FR-002**: The system MUST record the most recently user-selected result-window dimensions after a result-window resize.
- **FR-003**: The system MUST use the recorded dimensions for each subsequently created result window in the same session and after an application restart.
- **FR-004**: The system MUST store the recorded result-window dimensions alongside existing application settings without exposing them as a user-editable settings-page control.
- **FR-005**: The system MUST preserve the existing result-window minimum dimensions and MUST fall back to the 460 by 540 default when no valid persisted dimensions are available.
- **FR-006**: The result-window screenshot preview MUST have a maximum display height of 60 pixels while preserving source aspect ratio and fitting its available width.
- **FR-007**: Both the history list screenshot preview and history detail screenshot preview MUST have a maximum display height of 60 pixels while preserving source aspect ratio and fitting their available widths.
- **FR-008**: Existing saved settings and historical data MUST remain valid after the application is upgraded.

### User Experience and UI Requirements

- **UX-001**: The remembered dimensions apply silently; no confirmation, settings form, or new user-facing copy is added.
- **UX-002**: Result-window actions, result scrolling, error states, and always-on-top behavior remain available at both the 460 by 540 default and the existing minimum dimensions.
- **UI-001**: Reuse the existing result and history image styling patterns; only the common height limit changes.
- **UI-002**: Previews remain non-interactive, use their existing accessible labels, and do not introduce keyboard focus targets.
- **UI-003**: Human visual review covers the default result-window size, the minimum result-window size, a remembered larger size, and tall/wide images in result, history list, and history detail views.

## Key Entities

- **Result-window dimensions**: The latest valid width and height selected by a user for result windows, stored with application settings.
- **Result-window size preference**: An optional persisted value; absence means the default size is used.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A new installation opens result windows at exactly 460 by 540 pixels in 100% of tested launches.
- **SC-002**: After a user resizes a result window, 100% of tested later result windows in the same session and after restart use those dimensions.
- **SC-003**: In all tested result, history-list, and history-detail states, screenshot previews never exceed 60 pixels in height and never cause horizontal overflow.
- **SC-004**: All existing result actions and scrolling remain reachable at the default and minimum result-window dimensions in 100% of tested scenarios.

## Assumptions

- The existing 420 by 540 result-window minimum remains unchanged; the 460 by 540 default is deliberately close to that minimum.
- The user means the latest manually resized dimensions should become the shared preference for all future result windows, rather than a separate size per analysis run or monitor.
- The existing local application-settings store is the appropriate durable storage location and needs no new settings-page control.
- The requested 60-pixel limit applies to the result preview and both existing history image presentations.
