# Feature Specification: Thinking Stream Display

**Feature Branch**: `master`

**Created**: 2026-08-05

**Status**: Draft

**Input**: User description: "适配各家模型的 thinking 输出；thinking 默认折叠，流式 thinking 时展开，正式回答出现后折叠"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Separate Thinking from the Answer (Priority: P1)

As a user, I can use reasoning-capable models through supported native and OpenAI-compatible endpoints without seeing provider-specific thinking fields or `<think>` tags mixed into the final answer.

**Why this priority**: Correctly separating the two streams is required before the interface can present thinking without corrupting the answer, copy action, or saved result.

**Independent Test**: Feed representative structured-thinking streams and split `<think>` tags through each supported protocol and verify that thinking and final-answer text are emitted separately and in order.

**Acceptance Scenarios**:

1. **Given** an endpoint streams thinking in a dedicated field or thought block, **When** chunks arrive, **Then** the system classifies them as thinking rather than final-answer text.
2. **Given** an OpenAI-compatible endpoint streams a leading `<think>...</think>` section across arbitrary chunk boundaries, **When** the stream is processed, **Then** the tags are removed and their contents are classified as thinking.
3. **Given** a model returns only ordinary answer text, **When** the stream is processed, **Then** its content remains unchanged and no thinking section is created.

---

### User Story 2 - Follow Streaming Thinking without Losing the Answer (Priority: P2)

As a user, I can watch thinking while it is actively streaming, and the interface automatically gets it out of the way once the final answer begins.

**Why this priority**: It provides visibility during model latency while keeping the completed result focused on the useful answer.

**Independent Test**: Render a result with thinking-only streaming, append the first answer chunk, and complete the response; verify the thinking section starts open, closes when answer text appears, and remains user-expandable.

**Acceptance Scenarios**:

1. **Given** thinking is streaming and no answer text exists, **When** the result is displayed, **Then** the thinking section is visible and expanded.
2. **Given** thinking has been displayed, **When** the first final-answer text arrives, **Then** the thinking section automatically collapses.
3. **Given** a response has completed with thinking, **When** the user activates the thinking section, **Then** they can expand and collapse it using pointer or keyboard controls.
4. **Given** a response contains no thinking, **When** the result is displayed, **Then** no empty thinking section consumes space.

---

### User Story 3 - Revisit Saved Thinking (Priority: P3)

As a user who saves analysis history, I can revisit the thinking associated with a prior result without mixing it into the saved answer or result preview.

**Why this priority**: Persistence makes thinking useful after the result window closes while preserving the existing answer-focused list and copy behavior.

**Independent Test**: Save a successful analysis with thinking, reopen its history detail, and verify the thinking is present in a collapsed disclosure while the list preview, answer text, and copy action remain answer-only.

**Acceptance Scenarios**:

1. **Given** history saving is enabled and a run contains thinking, **When** the run succeeds or fails, **Then** its accumulated thinking is stored with the history entry.
2. **Given** a saved history entry contains thinking, **When** the user opens its detail, **Then** a collapsed thinking disclosure is available above the answer or error.
3. **Given** a saved entry contains thinking and a final answer, **When** the history list and copy action are used, **Then** previews and copied text contain only the final answer.
4. **Given** an older history entry has no thinking, **When** it is opened after upgrade, **Then** it remains readable and no empty disclosure appears.

### Edge Cases

- Opening and closing `<think>` tags may be split at any character boundary across streaming events.
- A stream may expose structured thinking and ordinary answer text in adjacent events.
- A response may contain thinking but fail, cancel, or end without a final answer.
- Empty thinking deltas and metadata-only stream events must not create visible content.
- The result window may attach after some or all thinking and answer text has already arrived.
- Unclosed leading `<think>` content must remain available as thinking rather than being discarded.
- Existing databases and history entries have no thinking field before upgrade.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST maintain separate accumulated thinking and final-answer text for an active analysis.
- **FR-002**: The system MUST recognize displayable thinking from supported structured stream fields and blocks used by OpenAI-compatible, Anthropic, and Gemini protocols.
- **FR-003**: The system MUST recognize a leading `<think>...</think>` section even when either tag spans multiple stream chunks.
- **FR-004**: The system MUST remove recognized thinking tags from displayed and copied final-answer text.
- **FR-005**: The system MUST preserve ordinary answer-only responses without modification.
- **FR-006**: The system MUST expose accumulated thinking when a result window attaches during or after streaming.
- **FR-007**: The system MUST keep the existing final answer as the value copied by "复制全文" and stored as the history result.
- **FR-008**: The system MUST request displayable thinking summaries where the selected native protocol has a stable request option, while preserving endpoint compatibility where no common option exists.
- **FR-009**: Empty or metadata-only thinking events MUST NOT count as answer output or satisfy the non-empty-response requirement.
- **FR-010**: When history saving is enabled, the system MUST persist accumulated thinking separately for successful and failed analyses.
- **FR-011**: History detail MUST expose persisted thinking in a collapsed disclosure without adding it to list previews, result search, or copy output.
- **FR-012**: Existing databases and history records MUST remain readable after the thinking field is added.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: During thinking-only streaming, the status remains the existing streaming state and the thinking section is expanded; submitting, answer streaming, completed, failed, cancelled, and retry states retain their existing controls and feedback.
- **UX-002**: The thinking section MUST use the platform's familiar disclosure interaction, the existing Chinese interface language, and the current result-window visual hierarchy.
- **UI-001**: The implementation MUST reuse existing color, border, radius, spacing, and typography tokens without adding a UI dependency.
- **UI-002**: The disclosure MUST be keyboard operable, expose its expanded state to assistive technology, keep live output readable, and remain usable in the compact result-window viewport.
- **UI-003**: Human visual review MUST cover a representative desktop result window and the compact `420px`-wide or `300px`-high layout in thinking-only, answer-streaming, and completed states.

### Key Entities

- **Analysis Snapshot**: The current run identifier, lifecycle state, accumulated thinking, accumulated final answer, history status, and error.
- **Normalized Provider Event**: A provider-independent thinking delta, answer delta, or completion signal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All representative provider fixtures classify 100% of supplied thinking and answer characters into the expected stream without exposing recognized `<think>` tags.
- **SC-002**: The thinking section is expanded for 100% of thinking-only streaming test states and collapsed after the first answer chunk in 100% of transition tests.
- **SC-003**: Answer-only responses, copying, and history storage retain their prior visible output in all regression tests.
- **SC-004**: The result interface remains keyboard operable and usable without clipped controls at the existing desktop and compact review sizes.
- **SC-005**: Persisted thinking is restored in 100% of success, failure, and database-upgrade history fixtures without changing answer previews or copied text.

## Assumptions

- Reasoning-capable OpenAI-compatible models may enable thinking by model choice, provider defaults, or vendor-specific request options; there is no single safe request parameter for every compatible endpoint.
- Displayable thinking may be a provider-supplied summary rather than raw hidden chain of thought.
- Thinking is stored separately when history saving is enabled and is never included in result previews, result search, or "复制全文".
- Existing plain-text rendering remains the security boundary; thinking is never interpreted as HTML.
