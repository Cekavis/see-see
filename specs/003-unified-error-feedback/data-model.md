# Data Model: Unified App Feedback

This feature adds ephemeral UI state only. It does not change persisted application data.

## Notification

Represents one operation outcome within a single application window.

| Field | Type | Rules |
|-------|------|-------|
| `id` | number | Unique, monotonically increasing within the provider instance; not derived from message text |
| `variant` | `error` or `success` | Determines semantics, visual token set, and default lifetime |
| `message` | string | Required, non-empty, already normalized for user display |
| `action` | optional action | Contains an accessible label and callback, normally for recovery |
| `durationMs` | number or persistent | Success defaults to 4000 ms; errors default to persistent |

## Notification Action

| Field | Type | Rules |
|-------|------|-------|
| `label` | string | Required accessible and visible action name |
| `onClick` | callback | Invoked once per activation; may publish a subsequent outcome |

## Provider State

- `notifications`: newest-first ordered collection of active notifications.
- `nextId`: provider-local sequence used to distinguish repeated identical outcomes.

## State Transitions

```text
published -> visible -> dismissed -> removed
                    \-> timeout -> removed    (success only)
                    \-> action invoked -> removed, then action proceeds
```

- Publishing never changes document layout or scroll position.
- Manual dismissal removes only the selected item.
- An action invocation removes its error before invoking recovery to prevent a stale error from contradicting a successful retry.
- Provider unmount removes all notification state for that window.
