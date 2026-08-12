# Feature Specification: Result and History Configuration Display

**Feature Branch**: `master`

**Created**: 2026-08-12

**Status**: Draft

**Input**: User description: "请你在识别结果窗口增加使用的模型配置和提示词配置名字，另外在历史界面也增加模型配置名字。另外请把历史记录每一项的排版改成顶部为图片，下面为内容，因为图片基本上是很宽的文字。历史页图片使用原图、自适应高度并设置高度上限，不要为宽图留空；历史记录改为可选择每页条目数量的分页。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Identify Result Configuration (Priority: P1)

As a user viewing a recognition result, I can see the model configuration and prompt configuration used for that analysis so that I can understand and reproduce the output.

**Why this priority**: The result window is the immediate place where users evaluate output quality and need configuration context.

**Independent Test**: Start or reopen an analysis result and verify that both configuration names appear while submitting, streaming, and after a terminal outcome.

**Acceptance Scenarios**:

1. **Given** an analysis has selected configurations, **When** the result window attaches to the run, **Then** it shows the selected model configuration name and prompt configuration name.
2. **Given** an analysis is retried with different active configurations, **When** the retry starts, **Then** the displayed names update to the configurations used by the retry.
3. **Given** the result window has not attached yet, **When** configuration names are unavailable, **Then** it does not render misleading empty labels.

---

### User Story 2 - Scan History with Wide Screenshots (Priority: P2)

As a user reviewing history, I can see the model configuration used by each record and view each wide screenshot above its text so that screenshot content and metadata are easier to scan.

**Why this priority**: History already contains the configuration name, and a vertical card layout better matches wide text screenshots.

**Independent Test**: Load history records with and without images at desktop and narrow widths; verify model names appear, original images sit above content at their natural aspect within a height cap, and page controls request only the selected number of records.

**Acceptance Scenarios**:

1. **Given** a history record, **When** it is shown in the list, **Then** its metadata includes the saved model configuration name.
2. **Given** a record has a wide screenshot, **When** its card renders, **Then** the original screenshot appears above the content at its natural aspect with no fixed-height blank area.
3. **Given** a record has a tall screenshot, **When** its card renders, **Then** the image is reduced to the defined height cap without cropping or distortion.
4. **Given** multiple pages of records, **When** the user changes page or page size, **Then** only that page is requested and the page indicator and navigation controls update.
5. **Given** a record has no image or is viewed at a narrow width, **When** its card renders, **Then** no empty image region is reserved and the card has no horizontal overflow.

### Edge Cases

- The result window renders before the backend attachment returns configuration names.
- A retry uses configurations different from the original attempt.
- Historical configuration records have since been renamed or deleted; the saved history name remains visible.
- A screenshot is extremely wide, tall, missing, or still loading.
- Result and history metadata contain long configuration names.
- A page size change occurs while the user is on a later page.
- The current page is the first or final page.
- A history update or deletion occurs while viewing a later page.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The recognition result window MUST show the model configuration name used by the current analysis attempt.
- **FR-002**: The recognition result window MUST show the prompt configuration name used by the current analysis attempt.
- **FR-003**: Configuration names MUST remain available throughout submitting, streaming, completed, failed, and cancelled states.
- **FR-004**: Retrying an analysis MUST update the displayed configuration names to the configurations used for that retry.
- **FR-005**: Every history list item MUST show its saved model configuration name.
- **FR-006**: Every history list item with an image MUST place that image above its textual content and actions.
- **FR-007**: History list images MUST load the saved original image rather than the generated thumbnail.
- **FR-008**: History images MUST preserve their natural aspect, use no fixed height or forced aspect ratio, and respect a maximum displayed height.
- **FR-009**: Records without an image MUST NOT reserve an empty image area.
- **FR-010**: The history list MUST use page navigation instead of cumulative "load more" behavior.
- **FR-011**: Users MUST be able to select 10, 20, or 50 records per page, with 10 as the default.
- **FR-012**: Changing filters or page size MUST return to the first page.
- **FR-013**: Existing history filtering, detail navigation, deletion, and result controls MUST remain unchanged.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: Reuse existing metadata language, result status, history card, button, select, and notification patterns.
- **UX-002**: Missing transient configuration names MUST be omitted until available rather than shown as blank or placeholder values.
- **UI-001**: Reuse existing design tokens and components; no new visual dependency or component abstraction is permitted.
- **UI-002**: Long names and images MUST wrap or scale within their containers without horizontal page overflow or unused fixed-height image space.
- **UI-003**: Human visual review MUST cover the result window at 460×500 and 420×360 and history at 1094×768 and 540×800.

### Key Entities

- **Analysis Snapshot**: Transient run state including the exact model and prompt configuration names used by the current attempt.
- **History Page**: One cursor-bounded set of persisted summaries with a selected page size, current page number, previous-page cursor, and optional next-page cursor.
- **History List Item**: Persisted summary including saved model configuration name, prompt name, result preview, timestamps, status, and optional original image.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of focused result tests, both configuration names appear when supplied and remain associated with the run across state updates.
- **SC-002**: In 100% of focused retry tests, the result snapshot changes to the retry configuration names.
- **SC-003**: In 100% of focused history tests, the saved model configuration name appears, original images are requested, and cards use a one-column image-first layout without fixed-height image space.
- **SC-004**: In 100% of pagination tests, page navigation and page-size changes issue bounded queries with the expected cursor and limit.
- **SC-005**: At all four representative viewport sizes, configuration metadata and history cards show no horizontal page overflow and images are not cropped or distorted.
- **SC-006**: Existing result, history, frontend, and Rust test suites continue to pass.

## Assumptions

- The requested names are configuration display names, not provider protocol or raw model IDs.
- History uses the name saved with the record even if the live configuration is renamed or deleted later.
- Prompt names already shown in history remain in place; the model configuration name is added beside them.
- Cursor-based previous/next page navigation is sufficient; arbitrary page-number jumps and a total record count are out of scope.
- Original images are bounded by the selected page size and loaded only for visible page records.
