import { useEffect, useRef, useState } from "react";
import { Button } from "../components/Button";
import { useNotifications } from "../components/Notifications";
import { getErrorMessage, type AppError } from "../ipc";

export type ResultSnapshot = {
  runId: string;
  modelConfigName: string;
  promptConfigName: string;
  state: "submitting" | "streaming" | "completed" | "failed" | "cancelled";
  thinking: string;
  text: string;
  savedToHistory: boolean;
  error: AppError | null;
};

type Props = {
  snapshot: ResultSnapshot;
  alwaysOnTop?: boolean;
  onCancel?: () => void | Promise<unknown>;
  onRetry?: () => void | Promise<unknown>;
  onCopy?: (text: string) => void | Promise<unknown>;
  onOpenMain?: () => void | Promise<unknown>;
  onAlwaysOnTop?: (value: boolean) => void | Promise<unknown>;
};

export function ThinkingDisclosure({
  text,
  initialOpen = false,
  live = false,
}: {
  text: string | null | undefined;
  initialOpen?: boolean;
  live?: boolean;
}) {
  if (!text) return null;
  return (
    <details className="result-view__thinking" open={initialOpen || undefined}>
      <summary>思考过程</summary>
      <pre aria-live={live ? "polite" : undefined}>{text}</pre>
    </details>
  );
}

export function Result({
  snapshot,
  alwaysOnTop = false,
  onCancel,
  onRetry,
  onCopy,
  onOpenMain,
  onAlwaysOnTop,
}: Props) {
  const notifications = useNotifications();
  const [retrying, setRetrying] = useState(false);
  const publishedError = useRef<string | undefined>(undefined);
  const active =
    snapshot.state === "submitting" || snapshot.state === "streaming";
  const hasAnswer = Boolean(snapshot.text);
  const displayText =
    snapshot.text ||
    (snapshot.state === "failed" && snapshot.error
      ? `${snapshot.error.message}${snapshot.error.details ? `\n\n错误详情\n${snapshot.error.details}` : ""}`
      : active
        ? snapshot.thinking
          ? "等待正式回答…"
          : "等待模型返回文字…"
        : "暂无结果");

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
          {snapshot.modelConfigName && snapshot.promptConfigName && (
            <div className="result-view__configuration">
              <span>模型配置：{snapshot.modelConfigName}</span>
              <span>提示词配置：{snapshot.promptConfigName}</span>
            </div>
          )}
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
      <div className="result-view__content">
        <ThinkingDisclosure
          key={`${snapshot.runId}:${active && !hasAnswer ? "thinking" : "answer"}`}
          text={snapshot.thinking}
          initialOpen={active && !hasAnswer}
          live={active && !snapshot.text}
        />
        <pre className="result-view__text" aria-live="polite">
          {displayText}
        </pre>
      </div>
      <footer className="button-row">
        <Button
          onClick={() => {
            if (!onOpenMain) return;
            try {
              const result = onOpenMain();
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
          打开主窗口
        </Button>
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
        {snapshot.state === "failed" && onRetry && (
          <Button
            variant="primary"
            disabled={retrying}
            onClick={() => {
              if (!onRetry) return;
              setRetrying(true);
              try {
                const result = onRetry();
                if (result instanceof Promise) {
                  void result
                    .catch((value: unknown) =>
                      notifications.error(getErrorMessage(value)),
                    )
                    .finally(() => setRetrying(false));
                } else {
                  setRetrying(false);
                }
              } catch (value) {
                setRetrying(false);
                notifications.error(getErrorMessage(value));
              }
            }}
          >
            {retrying ? "正在重试…" : "重试"}
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
