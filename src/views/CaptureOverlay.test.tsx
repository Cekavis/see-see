import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CaptureOverlay } from "./CaptureOverlay";

describe("CaptureOverlay", () => {
  it("captures the pointer, reports reverse drag selection, and cancels with Escape", () => {
    const onSelection = vi.fn();
    const onFinish = vi.fn();
    const onCancel = vi.fn();
    HTMLElement.prototype.setPointerCapture = vi.fn();
    render(
      <CaptureOverlay
        origin={{ x: -100, y: 0 }}
        scaleFactor={2}
        onSelection={onSelection}
        onFinish={onFinish}
        onCancel={onCancel}
      />,
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
});
