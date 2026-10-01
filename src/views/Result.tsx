import { useEffect, useRef, useState } from "react";
import { Button } from "../components/Button";
import { ModelRetryDialog } from "../components/ModelRetryDialog";
import { useNotifications } from "../components/Notifications";
import { RequestStatistics } from "../components/RequestStatistics";
import {
  getErrorMessage,
  type AppError,
  type ModelConfigSummary,
  type PerformanceMetrics as PerformanceMetricsValue,
} from "../ipc";

export type ResultSnapshot = {
  runId: string;
  modelConfigName: string;
  promptConfigName: string;
  state: "submitting" | "streaming" | "completed" | "failed" | "cancelled";
  thinking: string;
  text: string;
  inputTokens: number | null;
  outputTokens: number | null;
  savedToHistory: boolean;
  error: AppError | null;
  metrics: PerformanceMetricsValue;
};

type Props = {
  snapshot: ResultSnapshot;
  imageUrl?: string;
  alwaysOnTop?: boolean;
  onCancel?: () => void | Promise<unknown>;
  onRetry?: () => void | Promise<unknown>;
  onLoadModels?: () => Promise<ModelConfigSummary[]>;
  onRetryWithModel?: (modelConfigId: string) => Promise<unknown>;
  onModelChooserOpenChange?: (open: boolean) => Promise<void>;
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
  imageUrl,
  alwaysOnTop = false,
  onCancel,
  onRetry,
  onLoadModels,
  onRetryWithModel,
  onModelChooserOpenChange,
  onCopy,
  onOpenMain,
  onAlwaysOnTop,
}: Props) {
  const notifications = useNotifications();
  const [retrying, setRetrying] = useState(false);
  const [choosingModel, setChoosingModel] = useState(false);
  const [changingModelChooser, setChangingModelChooser] = useState(false);
  const [modelChooserError, setModelChooserError] = useState<string | null>(
    null,
  );
  const modelChooserState = useRef({
    active: true,
    open: false,
    pending: false,
    onOpenChange: onModelChooserOpenChange,
  });
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
    modelChooserState.current.onOpenChange = onModelChooserOpenChange;
  }, [onModelChooserOpenChange]);

  useEffect(() => {
    const chooser = modelChooserState.current;
    chooser.active = true;
    return () => {
      chooser.active = false;
      if (chooser.open && !chooser.pending) {
        void clearNativeModelChooser(chooser.onOpenChange);
      }
    };
  }, []);

  async function changeModelChooser(open: boolean) {
    const chooser = modelChooserState.current;
    if (!chooser.active || chooser.pending || chooser.open === open) return;
    chooser.pending = true;
    setChangingModelChooser(true);
    setModelChooserError(null);
    try {
      await chooser.onOpenChange?.(open);
      chooser.open = open;
      if (chooser.active) setChoosingModel(open);
    } catch (failure) {
      if (chooser.active) {
        const message = getErrorMessage(failure);
        if (chooser.open) setModelChooserError(message);
        else notifications.error(message);
      }
    } finally {
      chooser.pending = false;
      if (chooser.active) setChangingModelChooser(false);
      else if (chooser.open) {
        // An opening request may have completed after the result unmounted.
        void clearNativeModelChooser(chooser.onOpenChange);
      }
    }
  }

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
            {snapshot.state === "submitting" && "等待模型首字…"}
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
        <div className="result-view__header-actions">
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
          <RequestStatistics
            inputTokens={snapshot.inputTokens}
            outputTokens={snapshot.outputTokens}
            metrics={snapshot.metrics}
          />
        </div>
      </header>
      {imageUrl && (
        <div className="result-view__image-row">
          <img className="result-view__image" src={imageUrl} alt="原始截图" />
        </div>
      )}
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
        {onLoadModels && onRetryWithModel && (
          <Button
            aria-haspopup="dialog"
            aria-disabled={changingModelChooser}
            onClick={() => void changeModelChooser(true)}
          >
            换模型重试
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
      {onLoadModels && onRetryWithModel && (
        <ModelRetryDialog
          open={choosingModel}
          busy={changingModelChooser}
          error={modelChooserError}
          loadModels={onLoadModels}
          onSelect={onRetryWithModel}
          onClose={() => void changeModelChooser(false)}
        />
      )}
    </main>
  );
}

async function clearNativeModelChooser(
  onOpenChange: Props["onModelChooserOpenChange"],
) {
  try {
    await onOpenChange?.(false);
  } catch (failure) {
    console.error("无法清除结果窗口模型选择框状态", failure);
  }
}
