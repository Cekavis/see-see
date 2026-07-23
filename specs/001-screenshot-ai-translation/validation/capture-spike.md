# Capture spike validation

Date: 2026-07-23

## Automated evidence

- `cargo test --manifest-path src-tauri/Cargo.toml -j 1`: passed 15 tests.
- Negative virtual-desktop coordinates, reverse dragging, zero-sized cancellation, cross-monitor composition, mixed monitor scale metadata, PNG dimension limits, and the 8 MiB base64 ceiling are covered by `src-tauri/tests/capture_flow.rs`.
- Pointer Capture, physical-coordinate conversion, selection feedback, and Escape cancellation are covered by `src/views/CaptureOverlay.test.tsx`.

## Implementation decision

The spike keeps xcap snapshots and the physical-pixel coordinate model. Each monitor receives a separate fixed overlay, while the initiating overlay uses Pointer Capture and broadcasts the normalized physical selection through Rust.

## Outstanding real-platform evidence

- Windows mixed-DPI cross-window dragging has not yet been exercised manually in this environment.
- macOS dual-display, negative-coordinate, and screen-recording permission behavior cannot be validated from this Windows host.

T027 remains open until those platform checks are performed; no platform pass is inferred from automated tests.
