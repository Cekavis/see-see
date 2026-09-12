# Data Model: Remember Result Window Position

## Remembered Result-Window Location

An in-memory value shared by all result windows in the current application session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `x` | signed 32-bit integer | yes | Latest result-window left coordinate in physical screen pixels |
| `y` | signed 32-bit integer | yes | Latest result-window top coordinate in physical screen pixels |

### Lifecycle

1. The value starts empty when the application runtime is initialized.
2. A movement event from a result window replaces the value with the event's coordinates.
3. Result-window creation reads the current value before presentation.
4. The value is not changed when a result window closes and is not persisted across application restarts.

### Validation and Invariants

- The value is present only as a complete `(x, y)` pair.
- Coordinates may be negative because desktop work areas can exist to the left or above the primary display.
- Only result-window labels may update the value.
- Updating the value has no side effects on windows that are already open.
