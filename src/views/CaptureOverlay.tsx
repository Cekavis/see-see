import { listen } from "@tauri-apps/api/event";
import {
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { ipc, type PhysicalRect } from "../ipc";

type Point = { x: number; y: number };

type Props = {
  origin: Point;
  scaleFactor: number;
  sessionId?: string;
  monitorId?: string;
  onSelection?: (selection: PhysicalRect) => void;
  onFinish?: (selection: PhysicalRect) => void;
  onCancel?: () => void;
};

function rect(
  start: Point,
  end: Point,
  origin: Point,
  scaleFactor: number,
): PhysicalRect | null {
  const left = Math.min(start.x, end.x);
  const top = Math.min(start.y, end.y);
  const width = Math.abs(start.x - end.x);
  const height = Math.abs(start.y - end.y);
  if (width === 0 || height === 0) return null;
  return {
    x: Math.round(origin.x + left * scaleFactor),
    y: Math.round(origin.y + top * scaleFactor),
    width: Math.round(width * scaleFactor),
    height: Math.round(height * scaleFactor),
  };
}

export function CaptureOverlay({
  origin,
  scaleFactor,
  sessionId,
  monitorId,
  onSelection,
  onFinish,
  onCancel,
}: Props) {
  const start = useRef<Point | null>(null);
  const [localSelection, setLocalSelection] = useState<PhysicalRect | null>(
    null,
  );
  const [remoteSelection, setRemoteSelection] = useState<PhysicalRect | null>(
    null,
  );
  const [frameUrl, setFrameUrl] = useState<string>();

  useEffect(() => {
    if (!sessionId || !monitorId) return;
    let url: string | undefined;
    void ipc.getCaptureFrame(sessionId, monitorId).then((buffer) => {
      url = URL.createObjectURL(new Blob([buffer], { type: "image/png" }));
      setFrameUrl(url);
    });
    const unlisten = listen<PhysicalRect>("capture-selection", (event) =>
      setRemoteSelection(event.payload),
    );
    return () => {
      void unlisten.then((stop) => stop());
      if (url) URL.revokeObjectURL(url);
    };
  }, [monitorId, sessionId]);

  useEffect(() => {
    const cancel = () => {
      if (onCancel) onCancel();
      else if (sessionId) void ipc.cancelCapture(sessionId);
    };
    const keydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") cancel();
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, [onCancel, sessionId]);

  const selection = localSelection ?? remoteSelection;
  const handleMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!start.current) return;
    const next = { x: event.clientX, y: event.clientY };
    const value = rect(start.current, next, origin, scaleFactor);
    setLocalSelection(value);
    if (value) {
      if (onSelection) onSelection(value);
      else if (sessionId) void ipc.updateCaptureSelection(sessionId, value);
    }
  };

  return (
    <div
      className="capture-overlay"
      data-testid="capture-overlay"
      style={frameUrl ? { backgroundImage: `url(${frameUrl})` } : undefined}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        start.current = { x: event.clientX, y: event.clientY };
        setLocalSelection(null);
      }}
      onPointerMove={handleMove}
      onPointerUp={(event) => {
        if (!start.current) return;
        const value = rect(
          start.current,
          { x: event.clientX, y: event.clientY },
          origin,
          scaleFactor,
        );
        start.current = null;
        setLocalSelection(null);
        if (value) {
          if (onFinish) onFinish(value);
          else if (sessionId) void ipc.finishCapture(sessionId, value);
        } else if (onCancel) onCancel();
        else if (sessionId) void ipc.cancelCapture(sessionId);
      }}
    >
      <div className="capture-overlay__hint">拖动选择区域 · Esc 取消</div>
      {selection && (
        <div
          className="capture-overlay__selection"
          style={{
            left: (selection.x - origin.x) / scaleFactor,
            top: (selection.y - origin.y) / scaleFactor,
            width: selection.width / scaleFactor,
            height: selection.height / scaleFactor,
          }}
        />
      )}
    </div>
  );
}
