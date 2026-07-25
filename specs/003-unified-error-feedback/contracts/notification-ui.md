# UI Contract: Window Notification Service

## Provider boundary

- A notification provider MUST wrap the application root in every webview window.
- Provider state MUST be isolated to that React root and MUST NOT persist or cross window boundaries.
- The notification viewport MUST render outside normal page flow and above application scroll containers.

## Publishing contract

Views can publish:

- `error(message, options?)`
  - persistent by default;
  - announced assertively;
  - may include one recovery action.
- `success(message, options?)`
  - visible for 4000 ms by default;
  - announced politely;
  - manually dismissible.
- `dismiss(id)` and `clear()` for explicit lifecycle control.

Publishing returns the new notification identifier. Message identity does not determine notification identity.

## Presentation contract

Each notification contains:

- a non-color severity cue;
- complete wrapping message text;
- at most one optional action;
- a dismiss button named for assistive technology.

The viewport uses newest-first order, remains inside compact-window insets, clips neither horizontally nor behind scrolling content, and becomes internally scrollable when its content exceeds available height.

## Integration boundary

- Operation outcomes from model settings, desktop preferences, onboarding, prompts, history, and result actions use this contract.
- Field validation remains in `Field` and does not use this contract.
- Stored analysis failure details remain inline and do not use this contract.
- Caught values pass through the existing error-message normalization boundary before publication.
