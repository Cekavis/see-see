import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
  modelConfigName: "视觉模型",
  promptConfigName: "日语解析",
  state: "streaming",
  thinking: "",
  text: "逐步输出",
  savedToHistory: false,
  error: null,
  ...overrides,
});

function renderResult(node: React.ReactNode) {
  return render(<NotificationProvider>{node}</NotificationProvider>);
}

describe("Result", () => {
  it("shows the model and prompt configurations used by the run", () => {
    renderResult(<Result snapshot={snapshot()} />);

    expect(screen.getByText("模型配置：视觉模型")).toBeInTheDocument();
    expect(screen.getByText("提示词配置：日语解析")).toBeInTheDocument();
  });

  it("omits configuration metadata until the analysis attaches", () => {
    renderResult(
      <Result
        snapshot={snapshot({ modelConfigName: "", promptConfigName: "" })}
      />,
    );

    expect(screen.queryByText(/模型配置：/)).not.toBeInTheDocument();
    expect(screen.queryByText(/提示词配置：/)).not.toBeInTheDocument();
  });

  it("keeps the footer visible while the result text scrolls", () => {
    const styles = nodeProcess
      .getBuiltinModule("node:fs")
      .readFileSync(`${nodeProcess.cwd()}/src/styles.css`, "utf8");
    const resultViewRule = styles.match(/\.result-view\s*\{([^}]*)\}/)?.[1];
    const contentRule = styles.match(
      /\.result-view__content\s*\{([^}]*)\}/,
    )?.[1];
    const textRule = styles.match(/\.result-view__text\s*\{([^}]*)\}/)?.[1];

    expect(resultViewRule).toMatch(
      /grid-template-rows:\s*auto minmax\(0, 1fr\) auto;/,
    );
    expect(resultViewRule).not.toMatch(
      /grid-template-rows:\s*auto auto minmax\(0, 1fr\) auto;/,
    );
    expect(contentRule).toMatch(/display:\s*flex;/);
    expect(contentRule).toMatch(/flex-direction:\s*column;/);
    expect(contentRule).toMatch(/overflow:\s*hidden;/);
    expect(textRule).toMatch(/overflow:\s*auto;/);
    expect(styles).not.toMatch(
      /\.result-view__content\s*>\s*\.result-view__text\s*\{[^}]*overflow:\s*visible;/,
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

  it("shows a first-token wait while the request is being submitted", () => {
    renderResult(
      <Result
        snapshot={snapshot({ state: "submitting", thinking: "", text: "" })}
      />,
    );

    expect(screen.getByText("等待模型首字…")).toBeInTheDocument();
    expect(screen.getByText("等待模型返回文字…")).toBeInTheDocument();
  });

  it("keeps the main-window action available in active and terminal states", () => {
    const onOpenMain = vi.fn();
    const { rerender } = renderResult(
      <Result snapshot={snapshot()} onOpenMain={onOpenMain} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "打开主窗口" }));
    expect(onOpenMain).toHaveBeenCalledOnce();

    rerender(
      <NotificationProvider>
        <Result
          snapshot={snapshot({ state: "completed" })}
          onOpenMain={onOpenMain}
        />
      </NotificationProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "打开主窗口" }));
    expect(onOpenMain).toHaveBeenCalledTimes(2);
  });

  it("reports a main-window navigation failure without removing the result", async () => {
    renderResult(
      <Result
        snapshot={snapshot()}
        onOpenMain={() => Promise.reject(new Error("无法打开主窗口"))}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "打开主窗口" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("无法打开主窗口"),
    );
    expect(screen.getByText("逐步输出")).toBeInTheDocument();
  });

  it("keeps thinking open until answer text arrives, then defaults it closed", () => {
    const { rerender } = renderResult(
      <Result snapshot={snapshot({ thinking: "正在分析", text: "" })} />,
    );
    const disclosure = screen.getByText("思考过程").closest("details");
    expect(disclosure).toHaveAttribute("open");
    expect(screen.getByText("正在分析")).toBeInTheDocument();

    rerender(
      <NotificationProvider>
        <Result
          snapshot={snapshot({ thinking: "分析完成", text: "正式回答" })}
        />
      </NotificationProvider>,
    );
    expect(screen.getByText("思考过程").closest("details")).not.toHaveAttribute(
      "open",
    );
    expect(screen.getByText("正式回答")).toBeInTheDocument();
  });

  it("omits empty thinking and copies only the final answer", () => {
    const onCopy = vi.fn();
    const { unmount } = renderResult(
      <Result
        snapshot={snapshot({ thinking: "内部分析", text: "正式回答" })}
        onCopy={onCopy}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "复制全文" }));
    expect(onCopy).toHaveBeenCalledWith("正式回答");
    unmount();

    const { queryByText } = renderResult(
      <Result snapshot={snapshot({ thinking: "", text: "普通回答" })} />,
    );
    expect(queryByText("思考过程")).not.toBeInTheDocument();
  });

  it("renders completed and failed terminal states without unsafe rich text", () => {
    const onRetry = vi.fn();
    const { rerender } = renderResult(
      <Result
        snapshot={snapshot({
          state: "completed",
          thinking: "<script>分析</script>",
          text: "<script>纯文本</script>",
        })}
      />,
    );
    expect(screen.getByText("<script>分析</script>")).toBeInTheDocument();
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
              retryable: false,
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
