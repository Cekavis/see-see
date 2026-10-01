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
  return `${value.toFixed(1)} tps`;
}

export function PerformanceMetrics({
  metrics,
}: {
  metrics: PerformanceMetricsValue;
}) {
  const tps = formatTps(metrics.tps);
  return (
    <div className="performance-metrics" aria-label="模型性能">
      <span>TTFT：{formatDurationMs(metrics.ttftMs)}</span>
      <span aria-label={`速度：${tps}`}>{tps}</span>
    </div>
  );
}
