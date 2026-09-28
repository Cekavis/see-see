# Data Model

No schema migration or new persistent setting.

- Source: retained `AnalysisInput` contains PNG bytes, frozen `PromptSnapshot`, `ModelSnapshot`, credential, history preference, and timestamp.
- Derived request: copies image/prompt/history preference, replaces model and credential from the selected saved ID, refreshes `started_at`.
- Run: `start_analysis` creates a new UUID and `ActiveAnalysis`; its stream, terminal state, window, and history are independent.
- Chooser: open state, loading state, model summaries, pending selection guard, and recoverable error. Closed requests cannot update a later opening.

Invalid run IDs, unavailable original inputs, removed model configurations, and window failures return existing `AppError` values. Credentials stay entirely in Rust. Source states submitting/streaming/completed/failed/cancelled all permit creating a separate run.
