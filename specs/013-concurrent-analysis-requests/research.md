# Research: Concurrent Analysis Requests

## Decisions

1. **Key runtime analyses by run ID**: Result-window labels and all IPC commands already carry `runId`; a map makes lookup/removal exact and permits multiple unfinished runs.
2. **Keep retry in the same run**: The frontend binds a result window to its original URL run ID, so retrying in place avoids rebinding and preserves existing UI behavior.
3. **Snapshot the complete `AnalysisInput`**: Store image, prompt/model snapshots, API key, history setting, and initial timestamp context when the run is created. A retry clones this snapshot and refreshes only the new attempt's start time.
4. **Keep capture sessions single-owner**: Removing the analysis guard does not change capture overlay reservation rules.

## Rejected Alternatives

- Replace the run ID on retry: would require frontend window reattachment and risks losing the failed window's identity.
- Re-read active model/prompt settings on retry: produces a retry for the newest request rather than the failed request.
- Add a global request queue: would preserve serialization and violate the requested concurrent behavior.
