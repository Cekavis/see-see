# Research: History Detail Navigation

## Decision 1: Keep list and detail as internal History views

- **Decision**: Use mutually exclusive list/detail rendering inside the mounted History component.
- **Rationale**: This directly removes the hard-to-find below-list detail card while retaining filter values, loaded pages, and selected data in existing component state. It avoids introducing a router for a single local navigation flow.
- **Alternatives considered**: A modal would constrain long results and screenshots; a separate application route would add navigation infrastructure and state transfer beyond this feature; keeping the responsive stacked card would not solve discoverability.

## Decision 2: Restore the existing scroll container explicitly

- **Decision**: Capture the nearest settings content container's vertical scroll offset before loading detail, then restore it in a layout effect when returning to the list.
- **Rationale**: The settings shell, not the window, owns desktop scrolling. A layout effect restores the position immediately after the list DOM is committed, before the user sees a jump.
- **Alternatives considered**: Keeping the list hidden would retain layout and scroll but leave a large hidden tree and complicate accessibility; window scrolling does not match the desktop shell; session persistence is unnecessary for an in-view return flow.

## Decision 3: Preserve summary whitespace at presentation time

- **Decision**: Apply pre-wrapped whitespace behavior to the summary element while retaining anywhere wrapping.
- **Rationale**: SQLite's preview substring preserves newline characters; the current paragraph rendering collapses them. Presentation styling fixes the defect without changing stored data or IPC contracts.
- **Alternatives considered**: Splitting strings into explicit break elements adds rendering logic; transforming data in Rust risks changing search/preview semantics.

## Decision 4: Reduce only the desktop sidebar track

- **Decision**: Change the desktop grid track from 220 px to 184 px and leave existing compact breakpoint behavior intact.
- **Rationale**: This reduces sidebar width by about 16%, meeting the requested space savings while retaining 12 px side padding, navigation icons, gaps, and Chinese labels without clipping.
- **Alternatives considered**: Icon-only navigation would reduce discoverability; changing shared padding or font size would reduce comfortable target sizing and readability.

## Decision 5: Focus automated coverage on observable navigation behavior

- **Decision**: Extend `History.test.tsx` to cover dedicated detail rendering, retained filters and result pages, no return query, scroll restoration, whitespace styling, and existing copy/resubmit behavior.
- **Rationale**: These are the behavioral boundaries most likely to regress and can be verified at the component level without brittle pixel assertions.
- **Alternatives considered**: A new end-to-end suite would duplicate component coverage and add runtime; stylesheet pixel values will instead be checked by build plus representative visual review.
