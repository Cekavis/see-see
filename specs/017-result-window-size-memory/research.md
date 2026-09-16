# Research: Result Window Size Memory

## Decision: Store a single optional logical size in the existing application-settings row

**Rationale**: Result-window placement already uses native state and the application already persists durable preferences in its local settings row. Width and height belong to that same profile-level preference set, and null values provide a clean default for existing users.

**Alternatives considered**:

- A new settings table: rejected because a single pair of values does not need a new entity or relationship.
- A frontend-only preference: rejected because native result windows are created before their frontend view is available and the preference must survive restart.
- Per-analysis or per-monitor dimensions: rejected because the user asked for the latest user adjustment to be reused globally.

## Decision: Persist logical dimensions and translate native resize events using the window scale factor

**Rationale**: Native resize events report physical pixels, while the result-window builder accepts logical dimensions. Persisting the logical values keeps an equivalent visible size when a user moves between displays with different DPI scales.

**Alternatives considered**:

- Persist physical pixels: rejected because the same numeric value produces a different apparent size on a display with another scale factor.
- Keep fractional logical values: rejected because integer logical pixels are sufficient for user-selected window geometry and match the existing integer database preference pattern.

## Decision: Keep the current resize in runtime state and flush it when a result closes or the app quits

**Rationale**: This gives every later result window in the same session the latest size immediately without synchronous disk writes for every intermediate resize event. Closing a result or quitting through the existing action commits the latest value for the next launch.

**Alternatives considered**:

- Write to SQLite for every resize event: rejected because continuous native resize events can cause excessive synchronous local writes and reduce resize responsiveness.
- Persist only after application restart: rejected because the next result in the same session must also use the new size.

## Decision: Change all three existing screenshot preview rules to 60 pixels

**Rationale**: The result preview, history-list preview, and history-detail preview already preserve aspect ratio and fit their available width. Reducing their shared maximum height is the smallest consistent change.

**Alternatives considered**:

- Change only the history-list preview: rejected because the request covers the history page's screenshot presentations and would leave detail inconsistent.
- Add a new image component: rejected because existing markup and accessibility labels already meet the need.
