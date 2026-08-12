# Research: Result and History Configuration Display

## Result configuration source

- **Decision**: Store model and prompt configuration names in the existing analysis snapshot when a run starts and replace them when a retry resets the run.
- **Rationale**: This represents the exact configuration used by the attempt, works before history persistence, and adds no extra IPC call.
- **Alternatives considered**: Looking up active settings in the result window can be wrong for history resubmission or retries and adds requests; reading history fails while streaming or when history saving is disabled.

## History configuration source

- **Decision**: Render the existing saved `modelConfigName` returned in each history list item.
- **Rationale**: The backend already persists and queries the historical display name, including legacy-safe behavior.
- **Alternatives considered**: Joining live model settings would change historical meaning and fail after configuration deletion.

## Wide screenshot layout

- **Decision**: Request the saved original image, render it at intrinsic aspect with `auto` dimensions constrained only by container width and a maximum height, and omit the image element when absent.
- **Rationale**: This shows text at original quality, lets wide captures naturally use little height, and caps tall captures without reserving an artificial aspect-ratio box.
- **Alternatives considered**: Generated thumbnails lose detail; fixed height/aspect ratio leaves blank space; cover cropping hides text.

## Pagination model

- **Decision**: Reuse the backend cursor and limit fields, retain visited page cursors in the History view, and expose previous/next buttons with 10, 20, and 50 item choices.
- **Rationale**: The current query already returns a next cursor and accepts bounded limits, so pagination needs no database or IPC change and never loads more than the selected page.
- **Alternatives considered**: Offset pagination needs a query contract change and becomes slower on deep pages; arbitrary page jumps require a total count and cursor indexing; cumulative loading keeps all original images resident.

## Result metadata placement

- **Decision**: Place compact configuration metadata below the existing result status inside the header.
- **Rationale**: It stays visible while result text scrolls and does not compete with result actions.
- **Alternatives considered**: Putting it in the scroll area can disappear during review; adding a separate panel consumes scarce compact-window space.
