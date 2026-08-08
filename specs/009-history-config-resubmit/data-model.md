# Data Model: History Configuration Resubmit

## History Entry Configuration Reference

Extends each saved history entry with:

- `prompt_config_id`: optional stable identity of the prompt configuration used by the run.
- `model_config_id`: optional stable identity of the model configuration used by the run.
- Existing prompt name/body and model name/protocol/model ID fields remain immutable display snapshots.

### Relationships

- `prompt_config_id` references one current prompt configuration and becomes empty if that configuration is deleted.
- `model_config_id` references one current model configuration and becomes empty if that configuration is deleted.
- A history entry remains readable and deletable when either referenced configuration no longer exists.

### Validation and transitions

- New successful and failed analysis records store both selected configuration identities.
- Retrying the same analysis run replaces both identities and snapshots together with the latest attempt.
- Existing rows are backfilled only when their saved names exactly match a current configuration.
- Deleting a configuration clears only the reference; it does not delete or alter history.

## Resubmission Selection

Transient detail-view state containing:

- `selected_model_config_id`: one available model configuration identity.
- `selected_prompt_config_id`: one available prompt configuration identity.

### Default transition

For each configuration type, select the first available match in this order:

1. Retained history configuration identity.
2. Exact match to the saved historical configuration name.
3. Current global active configuration.
4. First available configuration.
5. Empty selection when no configuration exists.

Changing either selection does not persist settings. Submitting resolves both identities to their current saved values.
