import { Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useRef, useState } from "react";
import { useNotifications } from "./components/Notifications";
import {
  getErrorMessage,
  ipc,
  type AnalysisEvent,
  type AnalysisSnapshot,
} from "./ipc";
import { CaptureOverlay } from "./views/CaptureOverlay";
import { Result } from "./views/Result";
import { SettingsShell } from "./views/SettingsShell";
import { isResultWindowLabel } from "./windowLabels";

export const RESULT_ALWAYS_ON_TOP_CHANGED = "result-always-on-top-changed";

function MainView() {
  return <SettingsShell />;
}

function CaptureView() {
  const query = new URLSearchParams(window.location.search);
  const sessionId = query.get("session") ?? "";
  const monitorId = query.get("monitor") ?? "";
  return (
    <CaptureOverlay
      sessionId={sessionId}
      monitorId={monitorId}
      origin={{
        x: Number(query.get("x") ?? 0),
        y: Number(query.get("y") ?? 0),
      }}
      scaleFactor={Number(query.get("scale") ?? 1)}
    />
  );
}

function ResultView() {
  const notifications = useNotifications();
  const runId = new URLSearchParams(window.location.search).get("run") ?? "";
  const [snapshot, setSnapshot] = useState<AnalysisSnapshot>({
    runId,
    modelConfigName: "",
    promptConfigName: "",
    state: "submitting",
    thinking: "",
    text: "",
    inputTokens: null,
    outputTokens: null,
    savedToHistory: false,
    error: null,
  });
  const [imageUrl, setImageUrl] = useState<string>();
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);
  const alwaysOnTopEventReceived = useRef(false);

  useEffect(() => {
    if (!runId || typeof URL.createObjectURL !== "function") return;
    let active = true;
    let objectUrl: string | undefined;
    void ipc
      .getAnalysisImage(runId)
      .then((buffer) => {
        if (!active) return;
        objectUrl = URL.createObjectURL(
          new Blob([buffer], { type: "image/png" }),
        );
        setImageUrl(objectUrl);
      })
      .catch((value: unknown) => {
        if (active) notifications.error(getErrorMessage(value));
      });
    return () => {
      active = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [notifications, runId]);

  useEffect(() => {
    const channel = new Channel<AnalysisEvent>();
    channel.onmessage = (event) => {
      setSnapshot((current) => updateAnalysisSnapshot(current, event));
    };
    void ipc
      .attachAnalysis(runId, channel)
      .then((next) => {
        if (next.runId !== runId) {
          notifications.error("分析任务标识不匹配");
          return;
        }
        setSnapshot((current) =>
          mergeAttachedAnalysisSnapshot(current, next, runId),
        );
      })
      .catch((value: unknown) => notifications.error(getErrorMessage(value)));
  }, [notifications, runId]);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    void (async () => {
      try {
        const remove = await listen<boolean>(
          RESULT_ALWAYS_ON_TOP_CHANGED,
          (event) => {
            if (!active) return;
            alwaysOnTopEventReceived.current = true;
            setAlwaysOnTop(event.payload);
          },
        );
        if (!active) {
          remove();
          return;
        }
        unlisten = remove;

        const value = await ipc.getAppSnapshot();
        if (active && !alwaysOnTopEventReceived.current) {
          setAlwaysOnTop(value.settings.resultAlwaysOnTop);
        }
      } catch (value: unknown) {
        if (active) notifications.error(getErrorMessage(value));
      }
    })();

    return () => {
      active = false;
      unlisten?.();
    };
  }, [notifications]);

  return (
    <Result
      snapshot={snapshot}
      imageUrl={imageUrl}
      alwaysOnTop={alwaysOnTop}
      onCancel={() => ipc.cancelAnalysis(runId)}
      onRetry={() => ipc.retryAnalysis(runId)}
      onCopy={(text) => ipc.copyText(text)}
      onOpenMain={() => ipc.openMainWindow(runId)}
      onAlwaysOnTop={(value) => {
        setAlwaysOnTop(value);
        return ipc.setResultAlwaysOnTop(value);
      }}
    />
  );
}

export function updateAnalysisSnapshot(
  current: AnalysisSnapshot,
  event: AnalysisEvent,
): AnalysisSnapshot {
  if (event.runId !== current.runId) return current;
  if (event.type === "started")
    return {
      ...current,
      modelConfigName: event.modelConfigName,
      promptConfigName: event.promptConfigName,
      state: "submitting",
      thinking: "",
      text: "",
      inputTokens: null,
      outputTokens: null,
      savedToHistory: false,
      error: null,
    };
  if (event.type === "delta")
    return {
      ...current,
      state: "streaming",
      text: current.text + event.text,
    };
  if (event.type === "thinkingDelta")
    return {
      ...current,
      state: "streaming",
      thinking: current.thinking + event.text,
    };
  if (event.type === "usage")
    return {
      ...current,
      inputTokens: event.inputTokens ?? current.inputTokens,
      outputTokens: event.outputTokens ?? current.outputTokens,
    };
  if (event.type === "completed")
    return {
      ...current,
      state: "completed",
      thinking: event.thinking,
      text: event.text,
      inputTokens: event.inputTokens,
      outputTokens: event.outputTokens,
      savedToHistory: event.savedToHistory,
    };
  if (event.type === "failed")
    return {
      ...current,
      state: "failed",
      error: event.error,
      inputTokens: event.inputTokens,
      outputTokens: event.outputTokens,
      savedToHistory: event.savedToHistory,
    };
  return {
    ...current,
    state: "cancelled",
    thinking: "",
    text: "",
    error: null,
  };
}

export function mergeAttachedAnalysisSnapshot(
  current: AnalysisSnapshot,
  attached: AnalysisSnapshot,
  runId: string,
): AnalysisSnapshot {
  if (attached.runId !== runId || current.runId !== runId) return current;
  const hasLiveUpdate =
    current.state !== "submitting" ||
    Boolean(
      current.modelConfigName ||
      current.promptConfigName ||
      current.thinking ||
      current.text ||
      current.inputTokens !== null ||
      current.outputTokens !== null ||
      current.error,
    );
  return hasLiveUpdate ? current : attached;
}

type WindowShortcutEvent = Pick<
  KeyboardEvent,
  "key" | "code" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey"
>;

export function shouldCloseWindowOnKeydown(
  label: string,
  event: WindowShortcutEvent,
): boolean {
  const resultWindow = isResultWindowLabel(label);
  const escape =
    resultWindow &&
    (event.key === "Escape" || event.code === "Escape") &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.altKey &&
    !event.shiftKey;
  const closeShortcut =
    (label === "main" || resultWindow) &&
    (event.key.toLowerCase() === "w" || event.code === "KeyW") &&
    (event.ctrlKey || event.metaKey) &&
    !event.altKey &&
    !event.shiftKey;
  return escape || closeShortcut;
}

function PlaceholderView({ label }: { label: string }) {
  return (
    <main className="app-shell">
      <h1>{label}</h1>
    </main>
  );
}

export function App() {
  const currentWindow = getCurrentWebviewWindow();
  const label = currentWindow.label;

  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (!shouldCloseWindowOnKeydown(label, event)) return;
      event.preventDefault();
      event.stopPropagation();
      void currentWindow.close().catch((error: unknown) => {
        console.error("无法关闭窗口", error);
      });
    };
    window.addEventListener("keydown", keydown, true);
    return () => window.removeEventListener("keydown", keydown, true);
  }, [currentWindow, label]);

  if (label === "main") return <MainView />;
  if (label.startsWith("capture-")) return <CaptureView />;
  if (isResultWindowLabel(label)) return <ResultView />;
  return <PlaceholderView label={label} />;
}
