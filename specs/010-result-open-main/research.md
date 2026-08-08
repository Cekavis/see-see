# Research: Result Window Main Navigation

## Navigation ownership

- **Decision**: Handle the action in a Rust IPC command and reuse the existing main-window restore/show/focus sequence.
- **Rationale**: The backend already owns window lifecycle and authoritative analysis state, so one command avoids frontend permission changes and state races.
- **Alternatives considered**: Calling window APIs directly from the result webview would split lifecycle logic and rely on frontend capabilities; launching another application instance is unnecessary because the main window already exists.

## Result close boundary

- **Decision**: Close only when the backend snapshot is terminal: completed, failed, or cancelled.
- **Rationale**: Submitting and streaming analyses must continue delivering output. Using the backend snapshot at click time avoids acting on a delayed frontend render.
- **Alternatives considered**: Closing based on visible answer text can terminate a stream after its first token; closing only completed results leaves failed and cancelled terminal windows open contrary to the finished-state intent.

## Cleanup path

- **Decision**: Close the matching result window through its normal window-close event so the existing runtime cleanup remains the single cleanup path.
- **Rationale**: The close handler already removes the matching analysis and safely treats terminal cancellation as a no-op.
- **Alternatives considered**: Duplicating state removal inside the new command risks cleanup drift and inconsistent failure ordering.

## UI placement

- **Decision**: Add a shared Button to the existing result footer and keep it available in every analysis state.
- **Rationale**: The footer is already persistent at compact sizes and contains the result actions users expect.
- **Alternatives considered**: Adding navigation to the header creates a second action region and provides no usability advantage.
