# Research: Anthropic Messages Reasoning Effort

- The official Messages API reference defines `output_config.effort` values `low`, `medium`, `high`, `xhigh`, and `max`.
- The existing `ReasoningEffort` enum already represents all five values, so the storage and IPC shape can remain unchanged.
- The existing adapter already parses Anthropic `thinking_delta` events; this change only adds the request-side effort field.
- The separate `thinking: { type: "adaptive" }` option is intentionally omitted from this focused change so existing model compatibility and current request behavior remain stable.

Reference: https://platform.claude.com/docs/en/api/messages/create
