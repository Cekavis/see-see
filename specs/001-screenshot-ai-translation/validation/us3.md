# US3 validation

Date: 2026-07-23

- Rust tests verify both built-in presets, input limits, unique duplicate names, active deletion, and immutable prompt snapshots.
- Frontend tests verify loading, empty state, editing, saving, duplication, deletion confirmation, and keyboard-focusable native controls.
- Screenshot submission loads the active prompt once and moves the owned name/body snapshot into the analysis run and history record.

Outstanding: processing the same real screenshot with both built-in prompts requires a configured live multimodal endpoint, so T047 remains open.
