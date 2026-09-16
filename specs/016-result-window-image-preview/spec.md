# Feature Specification: Result Window Image Preview

**Feature Branch**: `master`

**Created**: 2026-09-16

**Status**: Ready for planning

**Input**: User description: "请你在结果窗口的上部也增加一行限制高度的图片方便对照，类似历史页。另外给结果窗口的高度增加大概50%。"

## User Scenarios & Testing

### User Story 1 - Compare the screenshot with the result (Priority: P1)

When a screenshot analysis opens its result window, the user can see the original captured image in a compact preview row above the analysis content and compare it with the streamed or completed result without opening the history page.

**Why this priority**: Direct visual comparison is the main value of the request and applies during both active analysis and completed-result review.

**Independent Test**: Open a result window for an analysis with a wide screenshot and with a tall screenshot. Confirm the original image appears above the result, keeps its proportions, stays within the preview height limit, and does not prevent result scrolling or footer actions.

**Acceptance Scenarios**:

1. **Given** an active or completed analysis has an original screenshot, **When** its result window is displayed, **Then** the screenshot appears above the thinking/result content with an accessible image label and a bounded height.
2. **Given** the screenshot is wider or taller than the result window, **When** the preview is rendered, **Then** it scales to fit the available width, preserves its aspect ratio, and does not create horizontal overflow or reserve blank space for an artificial aspect ratio.
3. **Given** the analysis is streaming, completed, failed, or cancelled, **When** the user uses the result window, **Then** the existing status, result scrolling, recovery actions, and footer actions remain available; a preview-loading or preview-retrieval failure does not remove the result content.

## Edge Cases

- If the current analysis image cannot be retrieved, the result window remains usable and reports the retrieval failure through the existing notification pattern.
- If the image is extremely tall, its displayed height is capped and the full result window remains usable.
- If the result window is resized to its minimum dimensions, the preview and result text do not cause horizontal overflow, and footer controls remain reachable.
- If the result window is reopened or attached after analysis has already completed, the same original screenshot remains available for that run.

## Requirements

### Functional Requirements

- **FR-001**: The result window MUST display the original screenshot associated with its current analysis run above the thinking and result content when the image is available.
- **FR-002**: The image preview MUST be identified as the original screenshot for screen-reader users and MUST preserve the source image's aspect ratio.
- **FR-003**: The image preview MUST adapt to the result window's available width, MUST have a fixed maximum display height, and MUST NOT use a fixed aspect-ratio placeholder that creates empty space for wide images.
- **FR-004**: The result window MUST continue to show streaming text, terminal results, errors, retry, copy, open-main-window, cancel, and always-on-top behavior independently of image retrieval.
- **FR-005**: The default result-window height MUST be increased by approximately 50% from the existing 500-pixel baseline, and the minimum height MUST be increased proportionally from the existing 360-pixel baseline while retaining the existing width limits.
- **FR-006**: The image preview MUST be loaded for the current run only and MUST NOT expose image bytes or data from another analysis run.

### User Experience and UI Requirements

- **UX-001**: Reuse the existing result-window layout, notification, typography, border, radius, and surface tokens; the new preview must read as the top comparison row rather than a separate management screen.
- **UX-002**: While the preview is loading or unavailable, the result content and footer remain usable without introducing a blocking empty state.
- **UI-001**: The preview must use an image element with meaningful alternative text, visible focus is not required because the image is non-interactive, and no new dependency or one-off control is introduced.
- **UI-002**: At the updated default size (460×750) and updated minimum size (420×540), the preview, result content, and footer must avoid horizontal clipping; the result text owns scrolling when content exceeds the available space.
- **UI-003**: Human visual review must cover wide and tall screenshots at the updated default and minimum result-window sizes, in streaming and completed states, including light and dark appearances when available.

## Key Entities

- **Active analysis image**: The original PNG captured for one in-memory analysis run, addressed by the run identifier and available for the lifetime of that run.
- **Result window viewport**: The resizable desktop window containing the image preview, analysis status/content, and existing actions.

## Success Criteria

### Measurable Outcomes

- **SC-001**: In every result-window state with a valid current-run screenshot, the preview is visible above the result content without requiring a history-page navigation.
- **SC-002**: At both 460×750 and 420×540, 100% of tested wide and tall screenshots preserve their aspect ratio, stay within the preview height limit, and produce no horizontal overflow.
- **SC-003**: At both updated window sizes, all existing result actions remain reachable and the result text remains scrollable when its content exceeds the viewport.
- **SC-004**: A preview retrieval failure leaves the result window functional and produces one user-visible failure notification without exposing raw image data or unrelated run content.

## Assumptions

- The current analysis run already owns the original screenshot in memory; no new persistence schema or history migration is required.
- "大概50%" is implemented as an exact 1.5× increase for the existing default and minimum heights: 500→750 and 360→540.
- The existing result-window width values (460 default and 420 minimum) remain unchanged.
- The feature applies to the result window only; the history page's existing image behavior remains unchanged.
