export type TokenUsageProps = {
  inputTokens: number | null | undefined;
  outputTokens: number | null | undefined;
  compact?: boolean;
};

function formatTokenCount(value: number | null | undefined) {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? new Intl.NumberFormat("en-US").format(value)
    : "—";
}

export function TokenUsage({
  inputTokens,
  outputTokens,
  compact = false,
}: TokenUsageProps) {
  return (
    <div className="token-usage" aria-label="模型调用 token 用量">
      <span>
        {compact ? "输入" : "输入 token"}：{formatTokenCount(inputTokens)}
      </span>
      <span>
        {compact ? "输出" : "输出 token"}：{formatTokenCount(outputTokens)}
      </span>
    </div>
  );
}
