# IPC Contract: Result Window Image Preview

## `get_analysis_image`

Returns the original PNG associated with one in-memory analysis run.

### Request

```text
runId: string
```

### Success response

Raw PNG bytes transported through the existing Tauri binary response mechanism.

### Errors

- `not_found`: the run does not exist in the current application runtime.
- Existing storage/IPC errors: the runtime state cannot be read.

### Contract rules

- The command MUST look up exactly the requested run identifier.
- The command MUST NOT read from or write to history storage.
- The command MUST NOT change analysis state, streaming, retry behavior, or result-window lifecycle.
- The frontend MUST treat retrieval failure as non-blocking and leave the result content/actions available.
