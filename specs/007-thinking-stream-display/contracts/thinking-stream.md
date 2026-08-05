# Contract: Thinking Stream and Result Disclosure

## Provider normalization

Adapters emit only these provider-independent events:

```text
ThinkingDelta(text)
TextDelta(text)
Completed
```

- OpenAI-compatible adapters recognize non-empty `reasoning_content`, `reasoning`, text entries in `reasoning_details`, and ordinary `content`.
- Anthropic recognizes `thinking_delta.thinking` and `text_delta.text` inside content block deltas.
- Gemini treats text parts marked `thought: true` as thinking and other text parts as final answer.
- A leading `<think>` block found in ordinary text is converted to thinking deltas by the shared stream parser.

## Analysis IPC

Snapshots include:

```json
{
  "runId": "run-id",
  "state": "streaming",
  "thinking": "provider reasoning summary",
  "text": "final answer so far",
  "savedToHistory": false,
  "error": null
}
```

Streaming events add:

```json
{"type":"thinkingDelta","runId":"run-id","text":"..."}
```

Completion includes both accumulated fields so the terminal frontend state is authoritative.

## Result disclosure

- No thinking text: render no disclosure.
- Active run with thinking and empty final answer: render disclosure open.
- First non-empty final-answer text: render disclosure closed.
- Completed, failed, or cancelled run: disclosure defaults closed when it exists.
- The native summary remains activatable by keyboard and pointer after automatic collapse.
- `aria-live` applies to the visible thinking content while streaming and to the answer content as it arrives.
- "复制全文" receives only `snapshot.text`.

## History contract

- Saved success and failure entries persist accumulated thinking in a nullable `thinking_text` field.
- History detail exposes it as optional `thinkingText`.
- History lists, result previews, result search, and copy continue to use only `resultText`.
- History disclosures are closed by default and omitted when `thinkingText` is empty or null.
- Older entries without thinking remain valid after migration.
