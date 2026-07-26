# UI Contract: History Navigation

## List view

- Accessible section name: `历史记录`.
- Contains the clear-history action, all three filters, search action, list/empty state, record actions, and pagination when available.
- A summary renders stored newline and blank-line characters and wraps long content without horizontal page overflow.
- Activating `查看详情` requests that record's detail. Until the request succeeds, the list remains available.

## Detail view

- Replaces the list view after a detail request succeeds; list filters and cards are not exposed to assistive technology while detail is active.
- Provides a keyboard-reachable `返回历史记录` action before the detail heading/content.
- Shows the selected record's existing metadata, optional original screenshot, success result or failure notice, copy action, and resubmit action.

## Return behavior

- Activating `返回历史记录` restores the previously rendered list.
- Filter field values, loaded items, next-page cursor, and scroll offset match their values at the moment detail navigation began.
- Returning does not call the history query interface.

## Failure behavior

- If record detail loading fails, show the existing notification feedback and retain the list view and its browsing state.
