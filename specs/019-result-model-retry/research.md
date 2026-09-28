# Research: Result Model Retry

## Retained request

Decision: Clone `ActiveAnalysis` original `AnalysisInput`, replace model and API key, refresh the timestamp, and use `start_analysis`.
Rationale: Source input is already retained even without history; prompt edits/deletion must not affect a retry. `start_analysis` creates a UUID, registers a distinct run, opens the result before launching network output, and rolls back failed window creation.
Alternatives: In-place `retry_analysis` only accepts failed states and replaces source output. History resubmission requires persisted history and a currently existing prompt. Neither fulfills the requested independent result.

## Model selection

Decision: Load saved model summaries each time the chooser opens and bound the list height. Display only the user-defined name in compact, single-line rows; preserve the full name in a title for truncated rows. Do not show search, model counts, or progress copy, as explicitly requested.
Rationale: Existing summaries omit credentials and distinguish configurations. Scrolling and arrow/Home/End navigation support hundreds of entries without a dependency. All configured IDs are selectable, avoiding unsafe name-based exclusion when the source model was renamed or deleted.
Alternatives: Remote model discovery would require changing saved configurations and expand scope.

## Accessibility

Decision: Use a native modal dialog with an accessible title, labeled list, native buttons, keyboard list navigation, Escape dismissal, and focus restoration, reusing the confirmation-dialog pattern.
Rationale: Native modal behavior handles focus containment; normal buttons keep model choice an immediate action.

## Concurrency and recovery

Decision: A synchronous pending guard prevents rapid duplicate selection; load generation/cleanup prevents stale completion after dismiss/reopen. Keep chooser open with feedback if creating the new run fails.
Rationale: New windows already isolate execution/cancellation; only the chooser's async lifecycle needs additional protection.

## Native dispatch and development subscriptions

Decision: Declare `retry_analysis_with_model` async, following `finish_capture` and `resubmit_history`. In ResultView, use an effect-local active flag to ignore callbacks and attach responses after cleanup.
Rationale: Locked Tauri 2.11.5 documents a Windows deadlock when creating WebViews from synchronous commands. StrictMode replays effect setup/cleanup in development; without invalidating the previous channel, each emitted delta is appended twice. Compile-time command constraints and real-App StrictMode tests cover these regressions. Keep the original repeated content and StrictMode itself.
