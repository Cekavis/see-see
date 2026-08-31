# Data Model: Concurrent Analysis Requests

## RuntimeState

| Field | Type | Meaning |
|---|---|---|
| `capture` | optional capture session | The one active screen-selection session, unchanged by this feature |
| `capture_reservation` | optional ID | Native capture reservation, unchanged |
| `analysis` | map keyed by `run_id` | Every retained analysis, including active and terminal runs whose result windows remain open |

## Analysis request snapshot

The run retains the image, model snapshot, prompt snapshot, API key, history-save setting, and request timing context used for the original attempt. Retry clones these values and updates only the new attempt timestamp.

## Lifecycle

1. Start: generate a UUID, create an `ActiveAnalysis`, insert it under that UUID, then create the matching result window.
2. Stream: events are emitted with the same UUID and consumed only by listeners attached to that run.
3. Retry: only a failed run may reset its state; its stored snapshot is reused under the same UUID.
4. Close/cancel: resolve and remove only the requested UUID.
5. Exit: signal cancellation to every map value.
