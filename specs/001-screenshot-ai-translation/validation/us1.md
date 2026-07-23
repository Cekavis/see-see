# US1 validation

Date: 2026-07-23

## Automated evidence

- `cargo test --manifest-path src-tauri/Cargo.toml -j 1`: passed. Capture tests cover reverse drag, zero-size cancellation, negative coordinates, cross-monitor composition, and PNG normalization. Analysis tests cover one active request, one terminal event, cancellation, storage failure, and no automatic retry.
- `npm test -- --run`: passed. Capture overlay and result tests cover selection completion/cancellation, streaming, copy, always-on-top control, failure state, and script-shaped output rendered as text.
- Regression test `state::tests::wrong_capture_id_does_not_discard_active_session`: passed. A stale finish/cancel request no longer removes the current capture session.
- `npm run test:e2e`: passed in Chrome 150. The IPC smoke records capture start, result copy, and history access after a model configuration call.

## Windows desktop startup

`npm run tauri dev` compiled the dev profile in 28.71 seconds and launched `src-tauri/target/debug/see-see.exe`. The task-owned Tauri CLI, Vite, and application processes were then stopped explicitly.

## Outstanding independent test

No live provider credential was supplied, so a real screenshot-to-model response was not executed. A physical mixed-DPI/negative-coordinate drag and macOS dual-display capture were also not available. T027 and T034 therefore remain open.
