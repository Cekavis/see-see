# Research: Result Window Image Preview

## Decision: Reuse the in-memory active analysis image

- **Decision**: Add a run-scoped command that returns the original PNG already stored by `ActiveAnalysis`.
- **Rationale**: The image is already retained for provider requests, retries, and history saving. This avoids a database migration, temporary files, duplicated serialization in `AnalysisSnapshot`, and exposing image bytes through the URL query string.
- **Alternatives considered**: Persisting/reloading through history would fail when history saving is disabled; adding base64 to every snapshot would increase event payloads and duplicate data; passing a data URL in the window URL would expose large data in navigation state.

## Decision: Use a frontend object URL with lifecycle cleanup

- **Decision**: Fetch the bytes after the result window mounts, create a blob URL, render it, and revoke it on run change/unmount.
- **Rationale**: This matches the existing history image pattern, avoids keeping large base64 strings in React state, and keeps image loading independent from streaming text updates.
- **Alternatives considered**: Inline base64 would be simpler to render but larger in memory and inconsistent with the existing image loading path; a shared global image cache is unnecessary for one image per result window.

## Decision: Use a proportional 1.5× native height increase

- **Decision**: Change the existing 500-pixel default height to 750 pixels and 360-pixel minimum height to 540 pixels; retain 460/420 widths.
- **Rationale**: It implements the user's approximate 50% request deterministically and provides enough vertical room for the bounded preview plus the existing result and footer areas.
- **Alternatives considered**: Increasing only the default height would make the minimum window disproportionately cramped; changing width would not address the requested comparison space.

## Decision: Put the optional preview in its own grid row

- **Decision**: Render the preview after the header and before the scrollable result content, and add a dedicated `auto` row to the result grid.
- **Rationale**: This makes the image consistently top-of-window, prevents it from competing with result scrolling, and lets the preview disappear cleanly if retrieval fails.
- **Alternatives considered**: Putting the image inside the scrolling result panel would make comparison less stable; putting it above the header would separate it from the result-window title and controls.
