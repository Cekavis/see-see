export type TokenUsageProps = {
  inputTokens: number | null | undefined;
  outputTokens: number | null | undefined;
};

function formatTokenCount(value: number | null | undefined) {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? new Intl.NumberFormat("en-US").format(value)
    : "—";
}

export function TokenUsage({ inputTokens, outputTokens }: TokenUsageProps) {
  return (
    <div className="token-usage" aria-label="模型调用 token 用量">
      <span>输入 token：{formatTokenCount(inputTokens)}</span>
      <span>输出 token：{formatTokenCount(outputTokens)}</span>
    </div>
  );
}
