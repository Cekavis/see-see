# Feature Specification: History Detail Navigation

**Feature Branch**: `master`

**Created**: 2026-07-26

**Status**: Draft

**Input**: User description: "Narrow the settings sidebar, replace the history detail card below the list with a dedicated detail view that can return while preserving search state and scroll position, and preserve line breaks and blank lines in history summaries."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open and Return from History Detail (Priority: P1)

As a user reviewing saved analyses, I can open one record in a dedicated history detail view and return to the same place in the history list so that the detail is easy to find without losing my browsing context.

**Why this priority**: The current detail card can appear below a long list and is easy to miss, interrupting the primary history-review workflow.

**Independent Test**: Search and scroll within history, open a record, verify that only its detail is shown, then return and verify that the filters, results, and prior scroll position are restored.

**Acceptance Scenarios**:

1. **Given** a populated history list, **When** the user selects "查看详情", **Then** the history list is replaced by a dedicated detail view for that record with a clearly labeled return action.
2. **Given** the user opened a detail after applying filters and scrolling the list, **When** the user returns, **Then** the same filter values, loaded results, and scroll position are restored without rerunning the initial query.
3. **Given** a detail request fails, **When** the failure is reported, **Then** the user remains on the history list and can retry opening a record.

---

### User Story 2 - Read Structured History Summaries (Priority: P2)

As a user scanning history, I can see the line breaks and blank lines present in each saved summary so that structured output remains readable.

**Why this priority**: Flattened summaries turn structured translations and explanations into dense text that is slower to scan.

**Independent Test**: Display a record whose summary contains both a newline and an empty line, then verify that both are visibly preserved in the list card.

**Acceptance Scenarios**:

1. **Given** a saved result summary containing line breaks and blank lines, **When** the history list renders it, **Then** the summary preserves those breaks while still wrapping long lines within the card.

---

### User Story 3 - Use a More Compact Sidebar (Priority: P3)

As a user, I can devote more window width to settings content because the desktop sidebar does not reserve unnecessary blank space.

**Why this priority**: A narrower sidebar improves content space across all settings screens without changing navigation behavior.

**Independent Test**: At a representative desktop viewport, compare the navigation width and confirm that every label and interaction remains clear and usable.

**Acceptance Scenarios**:

1. **Given** the desktop settings layout, **When** the window is wider than the compact navigation breakpoint, **Then** the sidebar is visibly narrower while the brand and all navigation labels remain unclipped.
2. **Given** a narrow viewport, **When** the responsive navigation layout activates, **Then** the existing horizontal navigation behavior remains usable.

### Edge Cases

- A record is deleted or becomes unavailable between list loading and detail selection.
- A user opens a detail after loading multiple result pages and then returns.
- The saved summary begins or ends with whitespace, contains multiple consecutive blank lines, or contains a single very long line.
- The window changes size while the detail view is open and the user subsequently returns.
- A record has no screenshot, no result text, or represents a failed analysis.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The history screen MUST present the list and a selected record's detail as separate, mutually exclusive views.
- **FR-002**: Each history record MUST provide an action that loads and opens that record in the detail view.
- **FR-003**: The detail view MUST provide a clearly labeled action that returns to the history list.
- **FR-004**: Returning from detail MUST preserve the current result-search value, prompt filter, status filter, loaded result set, pagination state, and scroll position.
- **FR-005**: Opening and returning from detail MUST NOT automatically issue a new history-list query.
- **FR-006**: If a detail cannot be loaded, the system MUST report the error and keep the list available.
- **FR-007**: History summaries MUST visually preserve stored line breaks and blank lines while wrapping content that exceeds the card width.
- **FR-008**: Existing detail actions for copying results and resubmitting screenshots MUST remain available with their existing enabled, success, and failure behavior.
- **FR-009**: The desktop settings sidebar MUST use less horizontal space than the current layout without clipping the brand or navigation labels.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: The list MUST retain its existing loading, empty, filtered-empty, pagination, error, deletion, and clear-history states; detail loading errors MUST be recoverable from the list.
- **UX-002**: Navigation, buttons, confirmations, notifications, and wording MUST reuse the application's established interaction and feedback patterns.
- **UI-001**: The implementation MUST reuse the existing settings header, buttons, cards, colors, spacing tokens, and responsive navigation rather than introduce a new visual system.
- **UI-002**: The return action MUST be keyboard reachable and have an accessible name; focus order MUST follow the visual order; long summaries MUST not create horizontal page overflow.
- **UI-003**: Human visual review MUST cover approximately 1094×768, 780×800, and 540×800 viewports in both list and detail states, checking hierarchy, spacing, wrapping, sidebar labels, and responsive behavior.

### Key Entities

- **History Browsing State**: The user's active search values, loaded results, pagination cursor, and scroll position while browsing history.
- **History Entry Detail**: The selected saved analysis, including result or failure information, model and prompt metadata, and optional screenshot.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of tested history records, selecting "查看详情" reveals the dedicated detail view without requiring the user to scroll farther down the list.
- **SC-002**: In 100% of return-flow tests, all three filter values and the list scroll position are restored to their prior values within one rendered frame.
- **SC-003**: Summaries containing one or more blank lines preserve the same visible line-break pattern in all representative viewport tests, with no horizontal page overflow.
- **SC-004**: At the 1094-pixel representative desktop width, the sidebar uses at least 15% less horizontal space while all five navigation labels and the brand remain fully visible.
- **SC-005**: All existing history actions and responsive navigation checks continue to pass with no regression.

## Assumptions

- "详情页面" means a dedicated view within the existing History settings section, not a separate native window or application-wide route.
- Returning restores the in-memory browsing state for the current visit to History; persisting it across app restarts or switching away from History is outside this feature's scope.
- Existing history data already retains newline characters; this feature changes their presentation rather than stored data.
- Browser-style forward navigation and deep links to a history record are outside scope.
