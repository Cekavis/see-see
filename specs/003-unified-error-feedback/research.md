# Research: Unified App Feedback

## Decision 1: Use one provider per webview window

- **Decision**: Wrap the React root with a notification provider that owns a window-local queue and exposes a hook to views.
- **Rationale**: It guarantees consistent behavior across settings and result views while keeping separate Tauri windows independent. It also prevents each page from duplicating positioning, timing, and accessibility logic.
- **Alternatives considered**: Page-local notice components remain vulnerable to scroll and overflow; a process-wide native notification service would be disproportionate and would mix state across windows.

## Decision 2: Render the notification viewport through a document-body portal

- **Decision**: Render a fixed notification viewport into `document.body` and position it above app content.
- **Rationale**: A portal avoids clipping by scroll containers, sticky regions, and component overflow while producing zero document reflow.
- **Alternatives considered**: Rendering inside each screen's content tree can still be clipped; reserving space near headers recreates the empty-gap problem the user rejected.

## Decision 3: Use a newest-first scrollable stack

- **Decision**: Keep distinct notifications in newest-first order in one bounded stack that can scroll vertically when compact height is exhausted.
- **Rationale**: Outcomes remain deterministic and do not overlap. Repeated identical messages remain independently dismissible because identity is generated per event rather than derived from message text.
- **Alternatives considered**: A single replace-only banner can hide a prior actionable error; independent fixed notices can overlap; an unbounded non-scrollable stack can cover the window.

## Decision 4: Persist errors and time-limit success

- **Decision**: Errors remain until dismissal; successes dismiss after four seconds by default. Both expose a manual dismiss control, and successful recovery removes the related error before publishing success.
- **Rationale**: Errors often require reading or recovery, while success confirmations should not become clutter. Four seconds matches the product specification and is testable with fake timers.
- **Alternatives considered**: Auto-dismissing errors risks hiding necessary details; permanent success notices recreate stale messages and visual noise.

## Decision 5: Keep validation and persistent domain content inline

- **Decision**: Field errors remain with `Field`; a stored failed analysis shown as history detail remains inline. Only operation outcomes use the notification provider.
- **Rationale**: Validation and historical content must remain available in context and should not disappear or move away from their subject.
- **Alternatives considered**: Converting every red message to a toast would reduce context and make persisted failure details transient.

## Decision 6: Use semantic roles without moving focus

- **Decision**: Errors use `role="alert"`; successes use `role="status"`; notification controls are keyboard focusable and clearly named. Notifications do not take focus automatically.
- **Rationale**: Severity is announced at the appropriate urgency without interrupting typing or changing a user's position.
- **Alternatives considered**: Forcing focus is disruptive; color alone does not communicate severity; a single assertive live region makes routine successes unnecessarily intrusive.

## Decision 7: Preserve sanitized message boundaries

- **Decision**: Views continue to use existing `AppError.message` and `getErrorMessage` normalization before publishing notifications.
- **Rationale**: The visual redesign must not expose raw provider responses, credentials, or internal payloads.
- **Alternatives considered**: Publishing arbitrary caught values could regress the existing security boundary.
