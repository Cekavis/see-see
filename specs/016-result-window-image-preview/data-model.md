# Data Model: Result Window Image Preview

## Active Analysis Image

An existing in-memory value associated with exactly one active analysis run.

| Field | Type | Required | Description |
|---|---|---:|---|
| `runId` | string | yes | Stable identifier supplied by the result window and used for isolation |
| `imagePng` | byte sequence | yes | Original captured PNG retained by the active analysis |

### Lifecycle

1. The backend creates the value when a new analysis run is created.
2. The result window requests it using its own `runId`.
3. The backend returns bytes only when that run is present in runtime state.
4. The frontend creates a temporary object URL for display.
5. The frontend revokes the object URL when the run changes or the result window unmounts.
6. Existing run cleanup removes the in-memory value when the result is closed; no persistent field changes.

### Validation and isolation

- A missing or unknown `runId` returns the existing not-found error pattern.
- A request for run A cannot return run B's image because lookup is keyed by the supplied run identifier.
- The command does not expose API keys, provider request metadata, or image data for another run.

## Result Window Viewport

Existing native window configuration with updated height values.

| Property | Value |
|---|---:|
| Default width | 460 px |
| Default height | 750 px |
| Minimum width | 420 px |
| Minimum height | 540 px |
| Preview maximum display height | 220 px |

The preview width remains responsive to the available content width, and its height is content-driven up to the cap rather than forced by an aspect-ratio placeholder.
