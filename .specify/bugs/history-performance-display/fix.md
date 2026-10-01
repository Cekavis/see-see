# Fix: history and performance metric presentation

## Changes

- Added `RequestStatistics` so result and history views share the compact `输入/输出`, `TTFT`, and speed presentation. The speed value is shown as `tps` without a visible `TPS：` prefix.
- Extended history list/detail IPC payloads with persisted TTFT, generation duration, and derived TPS. Missing or partial values remain nullable and render as `—`.
- Centralized TPS calculation for live analysis and history reconstruction.
- Changed performance chart rows to `display: contents` within one shared grid. The label, track, and value columns now use the same tracks for every model, so long values cannot shift an individual bar.
- Added frontend and Rust coverage for the shared layout, compact history display, metric round-tripping, and legacy/partial records.

## Files

- `src/components/PerformanceMetrics.tsx`
- `src/components/RequestStatistics.tsx`
- `src/views/Result.tsx`
- `src/views/History.tsx`
- `src/views/Performance.test.tsx`
- `src/styles.css`
- `src/ipc.ts`
- `src-tauri/src/analysis.rs`
- `src-tauri/src/history.rs`
- `src-tauri/tests/history_integration.rs`

## Compatibility

No schema change was required. Existing nullable timing columns and records without metrics remain readable.
