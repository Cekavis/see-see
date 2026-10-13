# Data Model: Anthropic Messages Reasoning Effort

| Field | Meaning | Validation |
|---|---|---|
| `reasoningEffort` | Protocol-specific model effort | OpenAI protocols accept the existing seven values; Anthropic accepts `low`, `medium`, `high`, `xhigh`, or `max`; unsupported protocols store `null` |

No database migration is required because the current model schema already permits the shared effort strings.
