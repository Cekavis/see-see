import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Result, type ResultSnapshot } from "./Result";

const snapshot = (overrides: Partial<ResultSnapshot> = {}): ResultSnapshot => ({
  runId: "run-1",
  state: "streaming",
  text: "逐步输出",
  savedToHistory: false,
  error: null,
  ...overrides,
});

describe("Result", () => {
  it("shows streaming text and exposes cancel, copy, and always-on-top controls", () => {
    const onCancel = vi.fn();
    const onCopy = vi.fn();
    const onAlwaysOnTop = vi.fn();
    render(
      <Result
        snapshot={snapshot()}
        alwaysOnTop
        onCancel={onCancel}
        onCopy={onCopy}
        onAlwaysOnTop={onAlwaysOnTop}
      />,
    );
    expect(screen.getByText("逐步输出")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "取消分析" }));
    fireEvent.click(screen.getByRole("button", { name: "复制全文" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "窗口置顶" }));
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onCopy).toHaveBeenCalledWith("逐步输出");
    expect(onAlwaysOnTop).toHaveBeenCalledWith(false);
  });

  it("renders completed and failed terminal states without unsafe rich text", () => {
    const { rerender } = render(
      <Result
        snapshot={snapshot({
          state: "completed",
          text: "<script>纯文本</script>",
        })}
      />,
    );
    expect(screen.getByText("<script>纯文本</script>")).toBeInTheDocument();
    expect(document.querySelector("script")).toBeNull();
    rerender(
      <Result
        snapshot={snapshot({
          state: "failed",
          text: "",
          error: { code: "timeout", message: "请求超时", retryable: true },
        })}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("请求超时");
  });
});
