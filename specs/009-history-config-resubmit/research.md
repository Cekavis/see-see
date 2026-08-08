# Research: History Configuration Resubmit

## Stable default identity

- **Decision**: Persist nullable model and prompt configuration IDs on each history entry while retaining the existing names and content/model snapshots.
- **Rationale**: Stable IDs allow the same configuration record to remain selected after its editable values or name change. Existing snapshots remain necessary to display what the historical run used.
- **Alternatives considered**: Matching only by name fails after renames and cannot reliably distinguish configuration identity; copying current configuration contents into a separate rerun record would violate the requirement to use later edits.

## Legacy history fallback

- **Decision**: Backfill identity columns by exact saved-name matches during migration, then use runtime fallback order of retained ID, exact name, active configuration, first available configuration.
- **Rationale**: This recovers original identities for most existing rows without inventing uncertain matches and keeps unmatched rows usable.
- **Alternatives considered**: Leaving all legacy IDs empty gives poorer defaults; fuzzy matching can silently select the wrong configuration.

## Global configuration isolation

- **Decision**: Pass selected IDs directly to the history resubmission command and load those configurations without calling global activation commands.
- **Rationale**: Local selection stays transient and the existing global capture configuration remains unchanged.
- **Alternatives considered**: Temporarily changing global settings creates race conditions and observable side effects; duplicating the analysis pipeline adds unnecessary code.

## UI interaction

- **Decision**: Add two labeled native selects in the existing history detail card and keep one primary action labeled "重新选择配置提交".
- **Rationale**: Native controls provide keyboard and accessibility behavior, fit existing form styling, and require no dialog or dependency.
- **Alternatives considered**: A modal adds interaction and state complexity without improving the two-field choice; custom comboboxes are unnecessary for the expected configuration count.
