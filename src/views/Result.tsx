import { Button } from "../components/Button";
import { ErrorNotice } from "../components/ErrorNotice";
import type { AppError } from "../ipc";

export type ResultSnapshot = {
  runId: string;
  state: "submitting" | "streaming" | "completed" | "failed" | "cancelled";
  text: string;
  savedToHistory: boolean;
  error: AppError | null;
};

type Props = {
  snapshot: ResultSnapshot;
  alwaysOnTop?: boolean;
  onCancel?: () => void;
  onCopy?: (text: string) => void;
  onAlwaysOnTop?: (value: boolean) => void;
};

export function Result({
  snapshot,
  alwaysOnTop = false,
  onCancel,
  onCopy,
  onAlwaysOnTop,
}: Props) {
  const active =
    snapshot.state === "submitting" || snapshot.state === "streaming";
  return (
    <main className="result-view">
      <header className="result-view__header">
        <div>
          <h1>识别结果</h1>
          <p aria-live="polite">
            {snapshot.state === "submitting" && "正在提交图片…"}
            {snapshot.state === "streaming" && "模型正在输出…"}
            {snapshot.state === "completed" &&
              (snapshot.savedToHistory ? "已完成并保存到历史" : "已完成")}
            {snapshot.state === "failed" && "分析失败"}
            {snapshot.state === "cancelled" && "已取消"}
          </p>
        </div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={alwaysOnTop}
            onChange={(event) => onAlwaysOnTop?.(event.target.checked)}
          />
          窗口置顶
        </label>
      </header>
      {snapshot.error && <ErrorNotice message={snapshot.error.message} />}
      <pre className="result-view__text" aria-live="polite">
        {snapshot.text || (active ? "等待模型返回文字…" : "暂无结果")}
      </pre>
      <footer className="button-row">
        {active && (
          <Button variant="danger" onClick={onCancel}>
            取消分析
          </Button>
        )}
        <Button
          disabled={!snapshot.text}
          onClick={() => onCopy?.(snapshot.text)}
        >
          复制全文
        </Button>
      </footer>
    </main>
  );
}
