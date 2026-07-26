# Data Model: History Detail Navigation

## History Browsing State

Transient state owned by the History screen for the duration of its mounted visit.

| Field | Meaning | Validation |
|-------|---------|------------|
| result search | Current free-text result query | String, including empty |
| prompt filter | Current prompt-name filter | String, including empty |
| status filter | All, success, or failed | One of the existing filter values |
| loaded items | Current ordered history results, including appended pages | Existing history list item contract |
| next cursor | Cursor for the next page | Existing cursor or null |
| list scroll offset | Vertical position before detail navigation | Non-negative number |

## History View State

| State | Visible content | Transition |
|-------|-----------------|------------|
| list | Header actions, filters, results, pagination | Selecting a record begins detail loading; success transitions to detail |
| detail | Return action, record metadata, screenshot/result or failure, detail actions | Return transitions to list and restores its scroll offset |

If detail loading fails, the state remains `list` and existing error feedback is shown.

## History Entry Detail

No stored fields or validation rules change. The detail continues to use the existing record identity, status, result/failure fields, model metadata, prompt metadata, timestamps, and optional image flag.

## Relationships

- One History Browsing State contains zero or more loaded history list items.
- The detail view represents exactly one item selected from the current browsing state.
- Returning removes the active detail selection but retains the browsing state.
