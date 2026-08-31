# Runtime Analysis Contract

## Run identity

- Every analysis has a unique `runId`.
- Result windows use `result-<runId>` labels and `?run=<runId>` URLs.
- `attach_analysis`, `cancel_analysis`, `retry_analysis`, `close_result`, and `open_main_window` operate only on the supplied run ID.

## Concurrency

- Starting a new analysis MUST NOT reject solely because another analysis exists.
- Capture reservation rules remain unchanged: two capture sessions cannot overlap.

## Retry

- `retry_analysis(runId)` is valid only when that run is failed.
- The retry keeps `runId` and uses the stored original request snapshot.
- The command MUST NOT load the latest active model or prompt configuration for a failed-run retry.
