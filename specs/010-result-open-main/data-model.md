# Data Model: Result Window Main Navigation

## Analysis Lifecycle State

Existing transient state used to decide whether the result window remains open:

- `submitting`: active; keep the result window open.
- `streaming`: active; keep the result window open.
- `completed`: terminal; close the result window after opening main.
- `failed`: terminal; close the result window after opening main.
- `cancelled`: terminal; close the result window after opening main.

No persisted entity, schema, or setting changes are required.

## Window Relationship

- The existing `main` window is restored, shown, and focused.
- The result window is identified by the analysis run identity in its existing window label.
- A terminal result closes through the existing close lifecycle, which releases its matching runtime analysis.
- An active result remains independently visible and subscribed to its ongoing analysis.
