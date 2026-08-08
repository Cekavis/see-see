# Contract: History Configuration Resubmit

## History detail response

The history detail response adds two nullable fields:

- `promptConfigId`: identity of the prompt configuration used by the saved run.
- `modelConfigId`: identity of the model configuration used by the saved run.

All existing history detail fields remain unchanged.

## Configuration choices

The detail view uses the existing configuration-list operations:

- List model configurations, including identity, display name, model ID, and active state.
- List prompt configurations, including identity, display name, body, and active state.

Loading either list does not mutate active settings.

## Resubmit request

`resubmit_history` receives:

- `id`: history entry identity.
- `modelConfigId`: selected model configuration identity.
- `promptConfigId`: selected prompt configuration identity.

## Resubmit behavior

1. Reject the request if capture or another analysis prevents a new run under existing concurrency rules.
2. Load the saved original image.
3. Resolve both selected identities to their current configuration values.
4. Resolve the selected model's current API credential.
5. Start the existing analysis flow without changing global active configuration fields.
6. If history saving is enabled, save the new run with the selected identities and current display snapshots.

Missing or deleted selected identities return the existing not-found/error feedback pattern and do not mutate settings.
