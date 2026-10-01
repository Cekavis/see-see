import type { PerformanceMetrics } from "../ipc";
import { PerformanceMetrics as PerformanceMetricsView } from "./PerformanceMetrics";
import { TokenUsage } from "./TokenUsage";

export type RequestStatisticsProps = {
  inputTokens: number | null | undefined;
  outputTokens: number | null | undefined;
  metrics: PerformanceMetrics;
  inline?: boolean;
};

export function RequestStatistics({
  inputTokens,
  outputTokens,
  metrics,
  inline = false,
}: RequestStatisticsProps) {
  return (
    <div
      className={`request-statistics${inline ? " request-statistics--inline" : ""}`}
      aria-label="请求统计"
    >
      <TokenUsage
        inputTokens={inputTokens}
        outputTokens={outputTokens}
        compact
      />
      <PerformanceMetricsView metrics={metrics} />
    </div>
  );
}
