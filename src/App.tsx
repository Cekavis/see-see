import { Channel } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { ipc, type AnalysisEvent, type AnalysisSnapshot } from "./ipc";
import { CaptureOverlay } from "./views/CaptureOverlay";
import { Result } from "./views/Result";
import { SettingsShell } from "./views/SettingsShell";

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
  const runId = new URLSearchParams(window.location.search).get("run") ?? "";
  const [snapshot, setSnapshot] = useState<AnalysisSnapshot>({
    runId,
    state: "submitting",
    text: "",
    savedToHistory: false,
    error: null,
  });
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);

  useEffect(() => {
    const channel = new Channel<AnalysisEvent>();
    channel.onmessage = (event) => {
      setSnapshot((current) => {
        if (event.runId !== current.runId) return current;
        if (event.type === "started")
          return { ...current, state: "submitting" };
        if (event.type === "delta")
          return {
            ...current,
            state: "streaming",
            text: current.text + event.text,
          };
        if (event.type === "completed")
          return {
            ...current,
            state: "completed",
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
        return { ...current, state: "cancelled", text: "" };
      });
    };
    void ipc.attachAnalysis(runId, channel).then(setSnapshot);
    void ipc
      .getAppSnapshot()
      .then((value) => setAlwaysOnTop(value.settings.resultAlwaysOnTop));
  }, [runId]);

  return (
    <Result
      snapshot={snapshot}
      alwaysOnTop={alwaysOnTop}
      onCancel={() => void ipc.cancelAnalysis(runId)}
      onCopy={(text) => void ipc.copyText(text)}
      onAlwaysOnTop={(value) => {
        setAlwaysOnTop(value);
        void ipc.setResultAlwaysOnTop(value);
      }}
    />
  );
}

function PlaceholderView({ label }: { label: string }) {
  return (
    <main className="app-shell">
      <h1>{label}</h1>
    </main>
  );
}

export function App() {
  const label = getCurrentWebviewWindow().label;
  if (label === "main") return <MainView />;
  if (label.startsWith("capture-")) return <CaptureView />;
  if (label === "result") return <ResultView />;
  return <PlaceholderView label={label} />;
}
