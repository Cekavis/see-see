# Data Model: Result and History Configuration Display

## Analysis Snapshot

Existing transient run snapshot with two added display fields:

- `modelConfigName`: exact model configuration display name used by the current attempt.
- `promptConfigName`: exact prompt configuration display name used by the current attempt.

Both values are set when the run starts. A retry replaces both values with the configuration names selected for that retry while retaining the run identity and source image.

## History List Item

Existing persisted summary; no schema change:

- `modelConfigName`: saved model configuration display name at execution time.
- `promptName`: saved prompt configuration display name at execution time.
- `hasImage`: controls the existing thumbnail load/empty region.

The list presentation uses these saved names and does not resolve live configuration records. For visible items with images, it requests the existing original image variant.

## History Page State

- `pageSize`: selected bounded limit; one of 10, 20, or 50.
- `pageIndex`: zero-based current page used to render the human-readable page number.
- `pageCursors`: cursors for visited pages; the first page cursor is absent.
- `nextCursor`: backend cursor for the next page, or absent on the final page.

Changing filters or page size resets all page state to the first page. Moving backward reuses the previously stored cursor without adding a total-count query.
