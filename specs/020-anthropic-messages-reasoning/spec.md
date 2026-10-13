# Feature Specification: Anthropic Messages Reasoning Effort

**Feature Branch**: `master`

**Created**: 2026-10-13

**Status**: Implemented

**Input**: User description: “对于 anthropic messages，也支持配置官方的几档 reasoning，默认 low”

## User Scenarios & Testing

### User Story 1 - Configure Anthropic effort (Priority: P1)

When creating or editing an Anthropic Messages model configuration, a user can choose one of Anthropic's official effort levels and the editor starts at `low`.

**Why this priority**: The setting must be discoverable and persist before requests can use it.

**Independent Test**: Open the model editor, select Anthropic Messages, verify the effort field defaults to `low`, choose each supported value, save, and reopen the configuration.

**Acceptance Scenarios**:

1. **Given** a new model editor, **When** the protocol is Anthropic Messages, **Then** the effort field is visible and defaults to `low`.
2. **Given** an Anthropic configuration, **When** the user selects `medium`, `high`, `xhigh`, or `max` and saves, **Then** the selected value is shown after reload and edit.
3. **Given** a non-Anthropic configuration, **When** the user edits it, **Then** its existing protocol-specific effort controls remain unchanged.

### User Story 2 - Send the selected effort (Priority: P1)

When an Anthropic Messages request runs, the selected effort is sent in the official `output_config.effort` field.

**Why this priority**: A saved value has no user value unless it changes the provider request.

**Independent Test**: Build an Anthropic request for every supported effort and inspect its JSON body without exposing the API key.

**Acceptance Scenarios**:

1. **Given** an Anthropic request with `low`, **When** the request is prepared, **Then** its body contains `output_config.effort = "low"`.
2. **Given** any supported Anthropic effort, **When** the request is prepared, **Then** the same value is forwarded without translation or omission.
3. **Given** an existing Anthropic configuration with no stored effort, **When** it is loaded, **Then** it behaves as `low`.

### Edge Cases

- `none` and `minimal` remain valid for OpenAI protocols but are not offered for Anthropic Messages because Anthropic's official effort values are `low`, `medium`, `high`, `xhigh`, and `max`.
- A model or compatible endpoint that does not support the selected effort may reject the request; the existing provider error path remains responsible for surfacing that failure.
- API keys remain in headers and must not appear in the serialized request body.

## Requirements

### Functional Requirements

- **FR-001**: The model configuration editor MUST treat Anthropic Messages as a reasoning-capable protocol.
- **FR-002**: The Anthropic effort selector MUST offer exactly `low`, `medium`, `high`, `xhigh`, and `max`, with `low` as the default for new and legacy configurations.
- **FR-003**: Saving, loading, duplicating, syncing, and retrying an Anthropic configuration MUST preserve its selected effort.
- **FR-004**: Anthropic Messages requests MUST map the selected effort to `output_config.effort`.
- **FR-005**: OpenAI and OpenAI Responses effort behavior MUST remain unchanged, including their existing `none` and `minimal` values.
- **FR-006**: Invalid Anthropic-only values MUST be rejected at the configuration boundary instead of being sent as an Anthropic effort.

### User Experience and UI Requirements

- **UX-001**: Reuse the existing “思考强度” field, select control, hints, notifications, and configuration card patterns.
- **UX-002**: The hint MUST identify that the selected level must be supported by the concrete model or endpoint.
- **UI-001**: The selector MUST remain keyboard accessible and fit the existing 720×520 settings layout without new styling.
- **UI-002**: Focus, save, edit, and error recovery behavior MUST match the existing model editor.

### Key Entities

- **Reasoning effort**: A persisted protocol-specific level represented by the existing shared model configuration field.
- **Anthropic Messages request**: A provider request whose output configuration carries the selected effort.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100% of supported Anthropic effort values can be selected, saved, reloaded, and displayed without manual database edits.
- **SC-002**: 100% of prepared Anthropic requests with a selected effort contain the same value under `output_config.effort`.
- **SC-003**: Existing OpenAI effort tests and configuration flows continue to pass unchanged.
- **SC-004**: The focused frontend and Rust regression suites pass with no new lint, formatting, or type-check failures.

## Assumptions

- Anthropic's current official Messages API values are `low`, `medium`, `high`, `xhigh`, and `max`.
- Existing local database schema already permits the shared effort strings, so no migration is required.
- The request keeps the existing `max_tokens` and streaming behavior; this feature only wires effort selection.
