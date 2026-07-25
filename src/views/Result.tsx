import { useEffect, useRef } from "react";
import { Button } from "../components/Button";
import { useNotifications } from "../components/Notifications";
import { getErrorMessage, type AppError } from "../ipc";

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
  onCancel?: () => void | Promise<unknown>;
  onCopy?: (text: string) => void | Promise<unknown>;
  onAlwaysOnTop?: (value: boolean) => void | Promise<unknown>;
};

export function Result({
  snapshot,
  alwaysOnTop = false,
  onCancel,
  onCopy,
  onAlwaysOnTop,
}: Props) {
  const notifications = useNotifications();
  const publishedError = useRef<string | undefined>(undefined);
  const active =
    snapshot.state === "submitting" || snapshot.state === "streaming";

  useEffect(() => {
    if (!snapshot.error) {
      publishedError.current = undefined;
      return;
    }
    const key = `${snapshot.runId}:${snapshot.error.code}:${snapshot.error.message}`;
    if (publishedError.current === key) return;
    publishedError.current = key;
    notifications.error(snapshot.error.message);
  }, [notifications, snapshot.error, snapshot.runId]);

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
            onChange={(event) => {
              if (!onAlwaysOnTop) return;
              try {
                const result = onAlwaysOnTop(event.target.checked);
                if (result instanceof Promise) {
                  void result.catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  );
                }
              } catch (value) {
                notifications.error(getErrorMessage(value));
              }
            }}
          />
          窗口置顶
        </label>
      </header>
      <pre className="result-view__text" aria-live="polite">
        {snapshot.text || (active ? "等待模型返回文字…" : "暂无结果")}
      </pre>
      <footer className="button-row">
        {active && (
          <Button
            variant="danger"
            onClick={() => {
              if (!onCancel) return;
              try {
                const result = onCancel();
                if (result instanceof Promise) {
                  void result.catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  );
                }
              } catch (value) {
                notifications.error(getErrorMessage(value));
              }
            }}
          >
            取消分析
          </Button>
        )}
        <Button
          disabled={!snapshot.text}
          onClick={() => {
            if (!onCopy) return;
            notifications.clear();
            try {
              const result = onCopy(snapshot.text);
              if (result instanceof Promise) {
                void result
                  .then(() => notifications.success("结果已复制"))
                  .catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  );
              } else {
                notifications.success("结果已复制");
              }
            } catch (value) {
              notifications.error(getErrorMessage(value));
            }
          }}
        >
          复制全文
        </Button>
      </footer>
    </main>
  );
}
