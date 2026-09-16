# Data Model: Result Window Size Memory

## ResultWindowDimensions

Represents the latest valid logical dimensions selected by the user for a result window.

| Field | Type | Validation | Meaning |
|-------|------|------------|---------|
| width | positive integer | At least the result window's supported minimum width | Visible content width to reuse for later result windows |
| height | positive integer | At least the result window's supported minimum height | Visible content height to reuse for later result windows |

## Persistence

The existing singleton application-settings record gains two nullable fields.

| Field | State | Meaning |
|-------|-------|---------|
| result window width | absent or valid integer | Absent means use the 460-pixel default width; a valid value overrides it. |
| result window height | absent or valid integer | Absent means use the 540-pixel default height; a valid value overrides it. |

Both fields must be present and valid before a saved preference is applied. A partial, missing, or undersized value falls back to the default dimensions.

## State Transitions

1. **No saved preference** → a new result opens at 460×540.
2. **Result resized** → the converted logical dimensions become the in-session preference.
3. **Result closes or app quits** → the latest in-session dimensions are stored.
4. **Application starts again** → stored valid dimensions seed the in-session preference.
5. **Invalid stored values** → the application uses the default safely.
