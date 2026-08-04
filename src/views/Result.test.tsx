import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { Result, type ResultSnapshot } from "./Result";

const nodeProcess = (
  globalThis as typeof globalThis & {
    process: {
      cwd(): string;
      getBuiltinModule(name: "node:fs"): {
        readFileSync(path: string, encoding: "utf8"): string;
      };
    };
  }
).process;

const snapshot = (overrides: Partial<ResultSnapshot> = {}): ResultSnapshot => ({
  runId: "run-1",
  state: "streaming",
  text: "逐步输出",
  savedToHistory: false,
  error: null,
  ...overrides,
});

function renderResult(node: React.ReactNode) {
  return render(<NotificationProvider>{node}</NotificationProvider>);
}

describe("Result", () => {
  it("keeps the footer visible while the result text scrolls", () => {
    const styles = nodeProcess
      .getBuiltinModule("node:fs")
      .readFileSync(`${nodeProcess.cwd()}/src/styles.css`, "utf8");
    const resultViewRule = styles.match(/\.result-view\s*\{([^}]*)\}/)?.[1];

    expect(resultViewRule).toMatch(
      /grid-template-rows:\s*auto minmax\(0, 1fr\) auto;/,
    );
    expect(resultViewRule).not.toMatch(
      /grid-template-rows:\s*auto auto minmax\(0, 1fr\) auto;/,
    );
  });

  it("shows streaming text and exposes cancel, copy, and always-on-top controls", () => {
    const onCancel = vi.fn();
    const onCopy = vi.fn();
    const onAlwaysOnTop = vi.fn();
    renderResult(
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
    expect(screen.getByRole("status")).toHaveTextContent("结果已复制");
    expect(onAlwaysOnTop).toHaveBeenCalledWith(false);
  });

  it("renders completed and failed terminal states without unsafe rich text", () => {
    const onRetry = vi.fn();
    const { rerender } = renderResult(
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
      <NotificationProvider>
        <Result
          snapshot={snapshot({
            state: "failed",
            text: "",
            error: {
              code: "timeout",
              message: "请求超时",
              details: "HTTP 504\nupstream timeout",
              retryable: true,
            },
          })}
          onRetry={onRetry}
        />
      </NotificationProvider>,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("请求超时");
    expect(screen.getByText(/HTTP 504/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    expect(onRetry).toHaveBeenCalledOnce();
  });
});
