import { Channel } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
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
    savedToHistory: false,
    error: null,
  });
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);

  useEffect(() => {
    const channel = new Channel<AnalysisEvent>();
    channel.onmessage = (event) => {
      setSnapshot((current) => updateAnalysisSnapshot(current, event));
    };
    void ipc
      .attachAnalysis(runId, channel)
      .then(setSnapshot)
      .catch((value: unknown) => notifications.error(getErrorMessage(value)));
    void ipc
      .getAppSnapshot()
      .then((value) => setAlwaysOnTop(value.settings.resultAlwaysOnTop))
      .catch((value: unknown) => notifications.error(getErrorMessage(value)));
  }, [notifications, runId]);

  return (
    <Result
      snapshot={snapshot}
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
  if (event.type === "completed")
    return {
      ...current,
      state: "completed",
      thinking: event.thinking,
      text: event.text,
      savedToHistory: event.savedToHistory,
    };
  if (event.type === "failed")
    return {
      ...current,
      state: "failed",
      error: event.error,
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
      void currentWindow.close();
    };
    window.addEventListener("keydown", keydown, true);
    return () => window.removeEventListener("keydown", keydown, true);
  }, [currentWindow, label]);

  if (label === "main") return <MainView />;
  if (label.startsWith("capture-")) return <CaptureView />;
  if (isResultWindowLabel(label)) return <ResultView />;
  return <PlaceholderView label={label} />;
}
