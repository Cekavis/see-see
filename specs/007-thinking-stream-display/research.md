# Research: Thinking Stream Display

## Decision 1: Normalize thinking before analysis state

**Decision**: Add a provider-independent thinking delta beside the existing text delta and completion event.

**Rationale**: OpenAI-compatible APIs commonly expose `reasoning_content` or reasoning detail records, Anthropic streams `thinking_delta`, and Gemini marks thought-summary parts. Classifying those shapes in their existing adapters keeps provider JSON out of UI and lifecycle code.

**Alternatives considered**:

- Parse provider JSON in the frontend: rejected because the frontend never sees raw provider events and this would duplicate protocol knowledge.
- Store tags inside answer text and split only for display: rejected because copy, history, completion validation, and late attachment would still receive polluted text.

## Decision 2: Parse `<think>` once in the shared stream path

**Decision**: Apply a small stateful parser to answer deltas before they reach analysis state. It recognizes only a leading `<think>` block and buffers only possible partial tag suffixes.

**Rationale**: MiniMax and compatible gateways may place thinking inside `content`, and either tag can be split across SSE events. The shared stream path is the single point all text deltas traverse, so one parser covers every adapter without caller-specific guards.

**Alternatives considered**:

- Regex per chunk: rejected because it fails when tags cross chunk boundaries.
- General XML parser: rejected because the stream is not an XML document and a dependency would add no value.
- Parse every `<think>` occurrence: rejected because a legitimate final answer may discuss or print the tag; provider convention places the reasoning block at the start.

## Decision 3: Preserve endpoint compatibility instead of forcing one OpenAI extension

**Decision**: Parse all common response representations but do not add a universal OpenAI-compatible thinking request field. Preserve provider defaults and model selection. For native Gemini, request thought summaries with its existing `includeThoughts` option.

**Rationale**: MiniMax uses `thinking.type=adaptive`, DeepSeek supports `thinking.type=enabled`, and Qwen-compatible services use `enable_thinking`; sending any one of these to unrelated endpoints can be rejected. Gemini's native protocol has a stable request option tied to the adapter already in use.

**Alternatives considered**:

- Send every known option: rejected because strict endpoints may reject unknown fields.
- Detect provider by base URL: rejected because gateways and custom domains hide the upstream and make hostname heuristics unreliable.
- Add arbitrary JSON request settings: rejected as unnecessary configuration and validation surface for this feature.

## Decision 4: Use native disclosure and persist thinking separately

**Decision**: Render thinking in `<details>` with `<summary>思考过程</summary>`. Open it only while the run is active and no answer text exists. Add one nullable history column and expose it only in history detail.

**Rationale**: Native disclosure provides keyboard and accessibility semantics with minimal code. A separate nullable field preserves answer previews, search, copy, and old rows while allowing users to revisit thinking.

**Alternatives considered**:

- Custom accordion state and button: rejected because the browser already provides the required behavior.
- Store thinking inside result text: rejected because it would pollute previews, search, copy, and existing answer contracts.

## Sources

- MiniMax OpenAI-compatible thinking and `reasoning_split`: https://platform.minimax.io/docs/api-reference/text-openai-api
- DeepSeek reasoning content: https://api-docs.deepseek.com/guides/thinking_mode/
- Anthropic thinking streaming: https://docs.anthropic.com/en/api/messages-streaming
- Gemini thought summaries: https://ai.google.dev/gemini-api/docs/generate-content/thinking
