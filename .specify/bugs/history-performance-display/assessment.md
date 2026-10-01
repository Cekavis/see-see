# Assessment: history and performance metric presentation

## Reported behavior

The performance chart lays out each model row with its own grid column sizing. A row with a wider metric value such as `174.7 tps / 174.7 tps` therefore moves its bar start to the left, so bars and value columns do not align. The result window also still exposes the `TPS：` label, while the history list and detail view do not consistently reuse the compact statistics format.

## Evidence

- `src/components/Performance.tsx` renders each metric row as an independent grid row.
- `src/styles.css` defines `grid-template-columns` on `.performance-chart__row`, so intrinsic value width is resolved per row.
- `src/components/PerformanceMetrics.tsx` renders a visible `TPS：` prefix.
- `src/views/Result.tsx` and `src/views/History.tsx` compose token and performance metrics separately, with different compact/list presentation rules.
- `src-tauri/src/history.rs` already persists timing columns; history DTOs need to expose the same metrics used by the result view.

## Scope and remediation

1. Share a statistics presentation component between result and history views, with compact inline mode for history list rows.
2. Remove the visible TPS prefix while retaining accessible metric labeling.
3. Include persisted TTFT, generation duration, and TPS in history list/detail DTOs and map them through the existing IPC types.
4. Change performance chart layout to one shared grid for all rows, keeping labels, bars, and values aligned even when values have different widths.
5. Add focused frontend and Rust regression coverage for formatting, history round-tripping, and missing metric values.

## Risks

The history schema already contains nullable timing columns, so the change is a DTO/UI compatibility update. Legacy records must continue to render placeholders. The chart layout must remain usable at the existing compact breakpoints.
