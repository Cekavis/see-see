import type { PerformanceMetrics as PerformanceMetricsValue } from "../ipc";

export function formatDurationMs(value: number | null | undefined) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    return "—";
  }
  if (value < 1000) return `${Math.round(value)} ms`;
  return `${(value / 1000).toFixed(2)} s`;
}

export function formatTps(value: number | null | undefined) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    return "—";
  }
  return `${value.toFixed(1)} token/s`;
}

export function PerformanceMetrics({
  metrics,
}: {
  metrics: PerformanceMetricsValue;
}) {
  return (
    <div className="performance-metrics" aria-label="模型性能">
      <span>首 token TTFT：{formatDurationMs(metrics.ttftMs)}</span>
      <span>TPS：{formatTps(metrics.tps)}</span>
    </div>
  );
}
