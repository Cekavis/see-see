# Data Model: Thinking Stream Display

## Normalized Provider Event

Represents one provider-independent streaming event.

| Variant | Data | Rule |
|---------|------|------|
| Thinking delta | Non-empty text | Appended only to analysis thinking |
| Text delta | Non-empty text | Appended only to the final answer |
| Completed | None | Ends provider streaming |

Structured provider thinking is classified in its adapter. Text deltas additionally pass through the leading-tag parser before becoming normalized events.

## Leading Think-Tag Parser

Transient state owned by one provider request.

| State | Meaning | Transition |
|-------|---------|------------|
| Start | The stream may begin with `<think>` | Opening tag → Thinking; any other text → Answer |
| Thinking | Content belongs to thinking | Closing tag → Answer |
| Answer | All later text belongs to the final answer | Terminal |

The parser buffers only text that may be a partial opening or closing tag. On end-of-stream, buffered start text becomes answer text and buffered thinking text remains thinking.

## Analysis Snapshot

| Field | Type | Behavior |
|-------|------|----------|
| runId | string | Existing analysis identifier |
| state | lifecycle state | Existing submitting/streaming/terminal state |
| thinking | string | Accumulated session-only thinking text |
| text | string | Accumulated final answer; copied and persisted as before |
| savedToHistory | boolean | Existing history result |
| error | optional error | Existing failure details |

### State transitions

- Started/retry clears both `thinking` and `text`.
- Thinking delta changes the run to streaming and appends only to `thinking`.
- Text delta changes the run to streaming and appends only to `text`.
- Completion preserves both fields in the in-memory snapshot and emits both to an attached result view.
- Cancellation clears both fields, matching existing cancellation behavior.
- Failure preserves partial thinking and answer in the active snapshot while retaining existing error handling.

## History Entry

| Field | Type | Behavior |
|-------|------|----------|
| thinkingText | nullable text | Accumulated thinking for successful or failed saved analyses |
| resultText | nullable text | Existing final answer; success entries still require it |

The database upgrade adds `thinking_text` as nullable, so existing rows remain valid. History list previews, result search, and copy continue to use only `result_text`. History detail shows `thinking_text` in the same disclosure used by the live result.
