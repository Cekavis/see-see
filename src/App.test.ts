import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { createElement, StrictMode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "./components/Notifications";
import type { AnalysisEvent, AnalysisSnapshot } from "./ipc";
import {
  App,
  RESULT_ALWAYS_ON_TOP_CHANGED,
  mergeAttachedAnalysisSnapshot,
  shouldCloseWindowOnKeydown,
  updateAnalysisSnapshot,
} from "./App";
import { ipc } from "./ipc";

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage: ((event: unknown) => void) | undefined;

    constructor(onmessage?: (event: unknown) => void) {
      this.onmessage = onmessage;
    }
  },
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: vi.fn(),
}));

const key = (overrides: Partial<KeyboardEvent> = {}) => ({
  key: "",
  code: "",
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...overrides,
});

const resultSnapshot: AnalysisSnapshot = {
  runId: "run-1",
  modelConfigName: "模型配置",
  promptConfigName: "提示词配置",
  state: "streaming",
  thinking: "",
  text: "结果",
  inputTokens: null,
  outputTokens: null,
  savedToHistory: false,
  error: null,
};

const appSnapshot = {
  settings: {
    activeModelConfigId: null,
    saveHistory: true,
    autostart: false,
    resultAlwaysOnTop: false,
    onboardingCompleted: true,
  },
  promptCount: 0,
  modelConfigCount: 0,
  activeModelConfigId: null,
  screenPermission: "unknown" as const,
};

describe("result window shared settings", () => {
  let onAlwaysOnTopChanged: ((event: { payload: boolean }) => void) | undefined;
  const unlisten = vi.fn();

  beforeEach(() => {
    window.history.replaceState({}, "", "/?run=run-1");
    onAlwaysOnTopChanged = undefined;
    unlisten.mockReset();
    vi.mocked(getCurrentWebviewWindow).mockReturnValue({
      label: "result-run-1",
      close: vi.fn().mockResolvedValue(undefined),
    } as unknown as ReturnType<typeof getCurrentWebviewWindow>);
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      onAlwaysOnTopChanged = handler as (event: { payload: boolean }) => void;
      return unlisten;
    });
    vi.spyOn(ipc, "attachAnalysis").mockResolvedValue(resultSnapshot);
    vi.spyOn(ipc, "getAnalysisImage").mockResolvedValue(new ArrayBuffer(0));
    vi.spyOn(ipc, "getAppSnapshot").mockResolvedValue(appSnapshot);
    vi.spyOn(ipc, "setResultModelChooserOpen").mockResolvedValue(undefined);
  });

  it("retries using the current window run and the model selected in the chooser", async () => {
    vi.spyOn(ipc, "listModelConfigs").mockResolvedValue([
      {
        id: "model-2",
        name: "另一模型",
        protocol: "gemini",
        baseUrl: "https://example.test",
        modelId: "vision-2",
        reasoningEffort: null,
        hasApiKey: true,
        isActive: false,
      },
    ]);
    const retry = vi.spyOn(ipc, "retryAnalysisWithModel").mockResolvedValue({
      runId: "run-2",
    });
    const { unmount } = render(
      createElement(NotificationProvider, null, createElement(App)),
    );

    await screen.findByText("模型配置：模型配置");
    fireEvent.click(screen.getByRole("button", { name: "换模型重试" }));
    fireEvent.click(await screen.findByRole("button", { name: /另一模型/ }));

    await waitFor(() => expect(retry).toHaveBeenCalledWith("run-1", "model-2"));
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(ipc.setResultModelChooserOpen).toHaveBeenNthCalledWith(1, true);
    expect(ipc.setResultModelChooserOpen).toHaveBeenNthCalledWith(2, false);
    expect(screen.getByText("结果")).toBeInTheDocument();
    expect(window.location.search).toBe("?run=run-1");
    unmount();
  });

  it("dismisses the model chooser with Escape before allowing the result window to close", async () => {
    vi.spyOn(ipc, "listModelConfigs").mockResolvedValue([
      {
        id: "model-2",
        name: "另一模型",
        protocol: "gemini",
        baseUrl: "https://example.test",
        modelId: "vision-2",
        reasoningEffort: null,
        hasApiKey: true,
        isActive: false,
      },
    ]);
    const currentWindow = getCurrentWebviewWindow();
    render(createElement(NotificationProvider, null, createElement(App)));
    const trigger = screen.getByRole("button", { name: "换模型重试" });
    trigger.focus();
    fireEvent.click(trigger);

    const model = await screen.findByRole("button", { name: "另一模型" });
    expect(ipc.setResultModelChooserOpen).toHaveBeenCalledExactlyOnceWith(true);
    model.focus();
    fireEvent.keyDown(model, { key: "Escape", code: "Escape" });

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(currentWindow.close).not.toHaveBeenCalled();
    expect(ipc.setResultModelChooserOpen).toHaveBeenNthCalledWith(2, false);
    expect(trigger).toHaveFocus();

    fireEvent.keyDown(trigger, { key: "Escape", code: "Escape" });
    expect(currentWindow.close).toHaveBeenCalledOnce();
  });

  it("applies each streamed delta once under StrictMode while preserving repeated tokens", async () => {
    const attach = vi.mocked(ipc.attachAnalysis).mockResolvedValue({
      ...resultSnapshot,
      state: "submitting",
      text: "",
    });
    render(
      createElement(
        StrictMode,
        null,
        createElement(NotificationProvider, null, createElement(App)),
      ),
    );

    await screen.findByText("模型配置：模型配置");
    expect(attach).toHaveBeenCalledTimes(2);
    const channels = attach.mock.calls.map(([, channel]) => channel);
    const broadcast = (event: AnalysisEvent) => {
      act(() => {
        channels.forEach((channel) => channel.onmessage(event));
      });
    };

    for (const text of ["思", "思", "考"]) {
      broadcast({ type: "thinkingDelta", runId: "run-1", text });
    }
    expect(screen.getByText("思思考")).toBeInTheDocument();

    for (const text of ["a", "a", "b"]) {
      broadcast({ type: "delta", runId: "run-1", text });
    }
    expect(screen.getByText("aab")).toBeInTheDocument();
    expect(screen.getByText("思思考")).toBeInTheDocument();
  });

  it.each(["snapshot", "mismatched snapshot", "failure"] as const)(
    "ignores a stale attach %s after StrictMode cleanup",
    async (outcome) => {
      let resolveStale!: (value: AnalysisSnapshot) => void;
      let rejectStale!: (reason: Error) => void;
      let resolveCurrent!: (value: AnalysisSnapshot) => void;
      const stale = new Promise<AnalysisSnapshot>((resolve, reject) => {
        resolveStale = resolve;
        rejectStale = reject;
      });
      const current = new Promise<AnalysisSnapshot>((resolve) => {
        resolveCurrent = resolve;
      });
      const attach = vi
        .mocked(ipc.attachAnalysis)
        .mockReturnValueOnce(stale)
        .mockReturnValueOnce(current);
      render(
        createElement(
          StrictMode,
          null,
          createElement(NotificationProvider, null, createElement(App)),
        ),
      );
      expect(attach).toHaveBeenCalledTimes(2);

      await act(async () => {
        if (outcome === "failure") {
          rejectStale(new Error("过期订阅失败"));
        } else {
          resolveStale({
            ...resultSnapshot,
            runId: outcome === "snapshot" ? "run-1" : "run-old",
            text: "过期结果",
          });
        }
      });

      expect(screen.queryByText("过期结果")).not.toBeInTheDocument();
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();

      await act(async () => {
        resolveCurrent({ ...resultSnapshot, text: "当前结果" });
      });
      expect(screen.getByText("当前结果")).toBeInTheDocument();
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    },
  );

  it("updates the checkbox from the shared always-on-top event", async () => {
    const { unmount } = render(
      createElement(NotificationProvider, null, createElement(App)),
    );

    const checkbox = await screen.findByRole("checkbox", { name: "窗口置顶" });
    await waitFor(() => expect(checkbox).not.toBeChecked());
    expect(listen).toHaveBeenCalledWith(
      RESULT_ALWAYS_ON_TOP_CHANGED,
      expect.any(Function),
    );

    onAlwaysOnTopChanged?.({ payload: true });
    await waitFor(() => expect(checkbox).toBeChecked());

    unmount();
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it("loads and cleans up the image belonging to the current run", async () => {
    const createObjectUrl = vi.fn(() => "blob:run-1");
    const revokeObjectUrl = vi.fn();
    const originalCreateObjectUrl = URL.createObjectURL;
    const originalRevokeObjectUrl = URL.revokeObjectURL;
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: createObjectUrl,
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: revokeObjectUrl,
    });
    vi.spyOn(ipc, "getAnalysisImage").mockResolvedValue(new ArrayBuffer(2));

    const { unmount } = render(
      createElement(NotificationProvider, null, createElement(App)),
    );

    const image = await screen.findByRole("img", { name: "原始截图" });
    expect(image).toHaveAttribute("src", "blob:run-1");
    expect(ipc.getAnalysisImage).toHaveBeenCalledWith("run-1");
    expect(createObjectUrl).toHaveBeenCalledOnce();

    unmount();
    expect(revokeObjectUrl).toHaveBeenCalledWith("blob:run-1");
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: originalCreateObjectUrl,
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: originalRevokeObjectUrl,
    });
  });

  it("keeps the result usable when the image cannot be retrieved", async () => {
    vi.spyOn(ipc, "getAnalysisImage").mockRejectedValue(
      new Error("图片加载失败"),
    );

    render(createElement(NotificationProvider, null, createElement(App)));

    expect(await screen.findByText("结果")).toBeInTheDocument();
    expect(await screen.findByRole("alert")).toHaveTextContent("图片加载失败");
  });
});

describe("window close shortcuts", () => {
  it.each(["main", "result-run-1"])("closes %s with Command+W", (label) => {
    expect(
      shouldCloseWindowOnKeydown(label, key({ key: "w", metaKey: true })),
    ).toBe(true);
    expect(
      shouldCloseWindowOnKeydown(label, key({ code: "KeyW", metaKey: true })),
    ).toBe(true);
    for (const modifier of [{ altKey: true }, { shiftKey: true }]) {
      expect(
        shouldCloseWindowOnKeydown(
          label,
          key({ key: "w", metaKey: true, ...modifier }),
        ),
      ).toBe(false);
    }
  });

  it("closes result windows with Escape or Ctrl+W", () => {
    expect(
      shouldCloseWindowOnKeydown("result-run-1", key({ key: "Escape" })),
    ).toBe(true);
    expect(
      shouldCloseWindowOnKeydown(
        "result-run-1",
        key({ key: "w", ctrlKey: true }),
      ),
    ).toBe(true);
  });

  it("closes the main window with Ctrl+W only", () => {
    expect(
      shouldCloseWindowOnKeydown("main", key({ key: "w", ctrlKey: true })),
    ).toBe(true);
    expect(shouldCloseWindowOnKeydown("main", key({ key: "Escape" }))).toBe(
      false,
    );
  });

  it("ignores shortcuts for other windows and modified Escape", () => {
    expect(
      shouldCloseWindowOnKeydown("settings", key({ key: "w", ctrlKey: true })),
    ).toBe(false);
    expect(
      shouldCloseWindowOnKeydown(
        "result-run-1",
        key({ key: "Escape", shiftKey: true }),
      ),
    ).toBe(false);
  });
});

describe("analysis event state", () => {
  it("does not replace a window with an attached snapshot from another run", () => {
    const current: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型一",
      promptConfigName: "提示词一",
      state: "streaming",
      thinking: "",
      text: "第一路",
      inputTokens: null,
      outputTokens: null,
      savedToHistory: false,
      error: null,
    };
    const attached = { ...current, runId: "run-2", text: "第二路" };

    expect(mergeAttachedAnalysisSnapshot(current, attached, "run-1")).toBe(
      current,
    );
  });

  it("does not let an attach snapshot overwrite live updates", () => {
    const current: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      state: "streaming",
      thinking: "",
      text: "已收到增量",
      inputTokens: null,
      outputTokens: null,
      savedToHistory: false,
      error: null,
    };
    const attached = { ...current, state: "submitting" as const, text: "" };

    expect(mergeAttachedAnalysisSnapshot(current, attached, "run-1")).toBe(
      current,
    );
  });

  it("clears the previous failure when a retry starts", () => {
    const failed: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "原模型配置",
      promptConfigName: "原提示词配置",
      state: "failed",
      thinking: "old thinking",
      text: "partial",
      inputTokens: 40,
      outputTokens: 12,
      savedToHistory: true,
      error: {
        code: "timeout",
        message: "模型请求超时",
        details: "request timed out",
        retryable: true,
      },
    };

    expect(
      updateAnalysisSnapshot(failed, {
        type: "started",
        runId: "run-1",
        modelConfigName: "重试模型配置",
        promptConfigName: "重试提示词配置",
      }),
    ).toEqual({
      runId: "run-1",
      modelConfigName: "重试模型配置",
      promptConfigName: "重试提示词配置",
      state: "submitting",
      thinking: "",
      text: "",
      inputTokens: null,
      outputTokens: null,
      savedToHistory: false,
      error: null,
    });
  });

  it("accumulates thinking separately and uses the terminal snapshot", () => {
    const initial: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      state: "submitting",
      thinking: "",
      text: "",
      inputTokens: null,
      outputTokens: null,
      savedToHistory: false,
      error: null,
    };
    const thinking = updateAnalysisSnapshot(initial, {
      type: "thinkingDelta",
      runId: "run-1",
      text: "分析",
    });
    expect(thinking).toMatchObject({
      state: "streaming",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      thinking: "分析",
      text: "",
    });
    const answer = updateAnalysisSnapshot(thinking, {
      type: "delta",
      runId: "run-1",
      text: "答案",
    });
    expect(answer).toMatchObject({ thinking: "分析", text: "答案" });
    const usage = updateAnalysisSnapshot(answer, {
      type: "usage",
      runId: "run-1",
      inputTokens: 99,
      outputTokens: 33,
    });
    expect(usage).toMatchObject({
      state: "streaming",
      thinking: "分析",
      text: "答案",
      inputTokens: 99,
      outputTokens: 33,
    });
    expect(
      updateAnalysisSnapshot(usage, {
        type: "completed",
        runId: "run-1",
        thinking: "完整分析",
        text: "完整答案",
        inputTokens: 99,
        outputTokens: 33,
        savedToHistory: true,
      }),
    ).toMatchObject({
      state: "completed",
      thinking: "完整分析",
      text: "完整答案",
      savedToHistory: true,
    });
  });
});
