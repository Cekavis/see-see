# Feature Specification: Concurrent Analysis Requests

**Feature Branch**: `master`

**Created**: 2026-08-31

**Status**: Implemented

**Input**: User description: "去掉同时只能有一个llm请求的限制。允许多个请求/未完成结果窗口同时存在。但是要区分好每个结果窗口对应的请求，如果失败重试要按原配置重试，不要错误地变成重试最新的请求"

## User Scenarios & Testing

### User Story 1 - Run multiple analyses concurrently (Priority: P1)

Users can start another screenshot analysis while an earlier model request is still submitting or streaming. Each request keeps its own result window and output.

**Why this priority**: Parallel work is the core requested capability and removes the current single-request bottleneck.

**Independent Test**: Start two analyses before either finishes, then verify both result windows remain open and each displays only its own stream and terminal state.

**Acceptance Scenarios**:

1. **Given** request A is still active, **When** the user starts request B, **Then** request B starts instead of being rejected as already running.
2. **Given** requests A and B are active, **When** output events arrive interleaved, **Then** each result window displays only events carrying its own run identity.
3. **Given** request A is terminal and its window remains open, **When** the user starts request B, **Then** A's completed or failed result remains available and B starts normally.

### User Story 2 - Retry the failed request with its original configuration (Priority: P1)

When a request fails, users can retry from that result window and the retry uses the exact request snapshot captured when that run started, even if the active model or prompt changed afterward.

**Why this priority**: A retry that silently targets the newest configuration can produce a result unrelated to the failed request and is unsafe for reproducibility.

**Independent Test**: Fail request A, change active model/prompt settings, retry A, and verify the retry still uses A's model, prompt, endpoint, credentials, image, and history-setting snapshot.

**Acceptance Scenarios**:

1. **Given** request A failed, **When** its result window's retry action is used, **Then** the same run identity is reset and restarted.
2. **Given** active settings changed after request A started, **When** A is retried, **Then** the retry does not read or use the newer active request configuration.
3. **Given** request B is newer than failed request A, **When** A is retried, **Then** B remains unaffected and continues displaying its own state.

### User Story 3 - Manage each result window independently (Priority: P2)

Users can cancel, close, navigate from, or attach to one result window without changing another run.

**Why this priority**: Independent lifecycle actions are required for concurrent windows to be trustworthy.

**Independent Test**: Keep two result windows open, close or cancel one by its run ID, and verify the other remains in the runtime and continues streaming.

**Acceptance Scenarios**:

1. **Given** A and B have separate result windows, **When** A is closed, **Then** only A is removed or cancelled.
2. **Given** A is terminal and B is active, **When** the user opens the main window from A, **Then** A's window may close while B remains active.

## Edge Cases

- Starting a capture while analyses are active is allowed, but two capture sessions cannot overlap.
- A stale or unknown result window run ID must not remove another run.
- If a retry is requested for a non-failed run, the command must reject it without changing the run.
- If result-window creation fails, the newly inserted run must be removed so no orphaned submitting task remains.
- Closing the application cancels every stored analysis, not only the newest one.

## Requirements

### Functional Requirements

- **FR-001**: The runtime MUST retain each analysis independently under its unique run ID.
- **FR-002**: The system MUST allow a new analysis to start while one or more other analyses are submitting, streaming, completed, failed, or cancelled.
- **FR-003**: Every result window action and event subscription MUST resolve the analysis by the requesting window's run ID.
- **FR-004**: A failed run MUST retain an immutable request snapshot containing its image, model configuration, prompt configuration, endpoint, credential, history setting, and request timestamp context.
- **FR-005**: Retrying a failed run MUST reuse that run's request snapshot and retain the same run ID; it MUST NOT load the latest active model or prompt configuration.
- **FR-006**: Removing or cancelling one run MUST NOT remove, cancel, or overwrite any other run.
- **FR-007**: Application exit MUST signal cancellation to every retained analysis.

### User Experience and UI Requirements

- **UX-001**: Each result window MUST show the model and prompt names belonging to its own run.
- **UX-002**: Loading, streaming, completed, failed, cancelled, and retrying states MUST remain available per window using existing controls and language patterns.
- **UI-001**: Result windows MUST continue using the existing compact result layout and shared button/toggle components.
- **UI-002**: Keyboard close actions MUST apply to the focused result window only.
- **UI-003**: Verification MUST include representative compact result-window sizes and a manual check that two windows remain distinguishable.

### Key Entities

- **Analysis run**: One model request, its run ID, state snapshot, stream listeners, cancellation signal, image, and original request configuration.
- **Result window**: A window labeled with exactly one analysis run ID and bound to that run for attach, cancel, retry, navigation, and close actions.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Two analyses can be started before either completes, with both result windows receiving independent output and no single-request rejection.
- **SC-002**: In an interleaved two-run test, zero events are rendered in the wrong result window.
- **SC-003**: A failed-run retry uses 100% of the original request snapshot fields in automated verification, regardless of later active-setting changes.
- **SC-004**: Closing or cancelling one of two runs leaves the other run addressable and unchanged in automated verification.

## Assumptions

- The existing provider client and network layer are safe to use concurrently; this feature removes application-level singleton state only.
- Capture UI remains single-session because overlapping screen selection overlays are a separate interaction concern.
- A retry keeps the existing result window and run ID so current frontend bindings remain valid.
