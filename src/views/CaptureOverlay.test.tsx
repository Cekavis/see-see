import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { CaptureOverlay } from "./CaptureOverlay";

const mocks = vi.hoisted(() => ({
  getCaptureFrame: vi.fn(),
  showCaptureOverlay: vi.fn(),
  updateCaptureSelection: vi.fn(),
  finishCapture: vi.fn(),
  cancelCapture: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("../ipc", () => ({
  getErrorMessage: (value: unknown) => String(value),
  ipc: mocks,
}));

describe("CaptureOverlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("captures the pointer, reports reverse drag selection, and cancels with Escape", () => {
    const onSelection = vi.fn();
    const onFinish = vi.fn();
    const onCancel = vi.fn();
    HTMLElement.prototype.setPointerCapture = vi.fn();
    render(
      <NotificationProvider>
        <CaptureOverlay
          origin={{ x: -100, y: 0 }}
          scaleFactor={2}
          onSelection={onSelection}
          onFinish={onFinish}
          onCancel={onCancel}
        />
      </NotificationProvider>,
    );
    const overlay = screen.getByTestId("capture-overlay");
    fireEvent.pointerDown(overlay, { pointerId: 7, clientX: 50, clientY: 40 });
    fireEvent.pointerMove(overlay, { pointerId: 7, clientX: 10, clientY: 5 });
    fireEvent.pointerUp(overlay, { pointerId: 7, clientX: 10, clientY: 5 });
    expect(overlay.setPointerCapture).toHaveBeenCalledWith(7);
    expect(onSelection).toHaveBeenLastCalledWith({
      x: -80,
      y: 10,
      width: 80,
      height: 70,
    });
    expect(onFinish).toHaveBeenCalledWith({
      x: -80,
      y: 10,
      width: 80,
      height: 70,
    });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("shows the window only after the capture frame is decoded and committed", async () => {
    let finishDecode: (() => void) | undefined;
    const decode = new Promise<void>((resolve) => {
      finishDecode = resolve;
    });
    vi.stubGlobal(
      "Image",
      class {
        src = "";
        decode = vi.fn(() => decode);
      },
    );
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: vi.fn(() => "blob:capture-frame"),
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: vi.fn(),
    });
    mocks.getCaptureFrame.mockResolvedValue(new ArrayBuffer(1));
    mocks.showCaptureOverlay.mockResolvedValue(undefined);

    render(
      <NotificationProvider>
        <CaptureOverlay
          origin={{ x: 0, y: 0 }}
          scaleFactor={1}
          sessionId="session"
          monitorId="monitor"
        />
      </NotificationProvider>,
    );

    await waitFor(() =>
      expect(mocks.getCaptureFrame).toHaveBeenCalledWith("session", "monitor"),
    );
    expect(mocks.showCaptureOverlay).not.toHaveBeenCalled();

    finishDecode?.();

    await waitFor(() =>
      expect(screen.getByTestId("capture-overlay")).toHaveStyle({
        backgroundImage: 'url("blob:capture-frame")',
      }),
    );
    await waitFor(() =>
      expect(mocks.showCaptureOverlay).toHaveBeenCalledWith(
        "session",
        "monitor",
      ),
    );
  });
});
