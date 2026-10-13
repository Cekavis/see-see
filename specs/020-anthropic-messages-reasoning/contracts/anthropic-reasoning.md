# Contract: Anthropic Messages Reasoning Effort

For an Anthropic `ProviderRequest` with `reasoning_effort: Some(ReasoningEffort::Low)`, the prepared JSON body MUST include:

```json
{
  "output_config": {
    "effort": "low"
  }
}
```

The same direct mapping applies to `medium`, `high`, `xhigh`, and `max`. `none` and `minimal` are invalid for Anthropic configuration input and are never emitted by the adapter.
