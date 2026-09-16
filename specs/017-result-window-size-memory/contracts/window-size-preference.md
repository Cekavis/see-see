# Window Size Preference Contract

## Native result-window lifecycle

### Input: native resize event

| Property | Requirement |
|----------|-------------|
| Scope | Only windows whose label identifies a result window may update the preference. |
| Source size | The event's physical content size is converted using the current window scale factor. |
| Validation | Dimensions below the existing minimum or otherwise invalid are ignored. |
| Session behavior | A valid resize updates the shared in-session preference immediately. |

### Persistence trigger

| Trigger | Requirement |
|---------|-------------|
| Result-window close | The latest valid in-session dimensions are saved before the corresponding analysis cleanup completes. |
| Explicit app quit | The latest valid in-session dimensions are saved before the process exits. |
| Save failure | The result window and quit action continue; the failure is logged through the existing native logging path. |

### Input: new result window

| Condition | Result |
|-----------|--------|
| Valid in-session or persisted dimensions exist | Open at those dimensions. |
| No valid dimensions exist | Open at 460×540. |
| Persisted dimensions are invalid or partial | Ignore them and open at 460×540. |

## Image-preview style contract

The result preview, history-list preview, and history-detail preview retain existing width fitting and aspect-ratio preservation. Each has a maximum display height of exactly 60 logical CSS pixels.
