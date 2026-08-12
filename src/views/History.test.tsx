import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { History, type HistoryApi } from "./History";

const listen = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/event", () => ({ listen }));

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

const item = {
  id: "h1",
  status: "success" as const,
  resultPreview: "旅行：旅行",
  errorMessage: null,
  promptName: "日语学习解析",
  modelConfigName: "模型",
  modelId: "vision",
  startedAt: "2026-07-23T00:00:00Z",
  completedAt: "2026-07-23T00:00:01Z",
  hasImage: true,
};

const detail = {
  ...item,
  promptConfigId: "prompt-original",
  modelConfigId: "model-original",
  thinkingText: "先识别文字，再翻译",
  resultText: "旅行（りょこう）：旅行",
  errorCode: null,
  promptBody: "解释",
  protocol: "openai",
};

const models = [
  {
    id: "model-original",
    name: "已更新模型",
    protocol: "openai" as const,
    baseUrl: "https://example.com/v1",
    modelId: "vision-v2",
    hasApiKey: true,
    isActive: false,
  },
  {
    id: "model-other",
    name: "备用模型",
    protocol: "gemini" as const,
    baseUrl: "https://example.com",
    modelId: "gemini-vision",
    hasApiKey: true,
    isActive: true,
  },
];

const prompts = [
  {
    id: "prompt-original",
    name: "已更新提示词",
    body: "更新后的正文",
    isBuiltin: false,
    isActive: false,
  },
  {
    id: "prompt-other",
    name: "备用提示词",
    body: "备用正文",
    isBuiltin: false,
    isActive: true,
  },
];

function api(items = [item]): HistoryApi {
  return {
    queryHistory: vi.fn().mockResolvedValue({ items, nextCursor: null }),
    getHistoryEntry: vi.fn().mockResolvedValue(detail),
    getHistoryImage: vi.fn().mockResolvedValue(new ArrayBuffer(2)),
    listModelConfigs: vi.fn().mockResolvedValue(models),
    listPromptPresets: vi.fn().mockResolvedValue(prompts),
    resubmitHistory: vi.fn().mockResolvedValue({ runId: "run-2" }),
    deleteHistoryEntry: vi.fn().mockResolvedValue(undefined),
    clearHistory: vi.fn().mockResolvedValue({ deletedCount: 1 }),
    copyText: vi.fn().mockResolvedValue(undefined),
  };
}

function renderHistory(service: HistoryApi) {
  return render(
    <NotificationProvider>
      <div className="settings-content">
        <History api={service} />
      </div>
    </NotificationProvider>,
  );
}

describe("History", () => {
  beforeEach(() => {
    listen.mockReset();
    listen.mockResolvedValue(vi.fn());
  });

  it("refreshes when a new history record is saved", async () => {
    const service = api([]);
    vi.mocked(service.queryHistory)
      .mockResolvedValueOnce({ items: [], nextCursor: null })
      .mockResolvedValueOnce({ items: [item], nextCursor: null });
    const removeListener = vi.fn();
    listen.mockResolvedValue(removeListener);
    const view = renderHistory(service);

    expect(await screen.findByText("没有历史记录")).toBeInTheDocument();
    await waitFor(() =>
      expect(listen).toHaveBeenCalledWith(
        "history-updated",
        expect.any(Function),
      ),
    );
    const onHistoryUpdated = listen.mock.calls[0][1] as () => void;
    onHistoryUpdated();

    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    expect(service.queryHistory).toHaveBeenLastCalledWith({
      text: undefined,
      promptName: undefined,
      status: undefined,
      cursor: undefined,
      limit: 10,
    });
    view.unmount();
    expect(removeListener).toHaveBeenCalledOnce();
  });

  it("shows the saved model name and uses an image-first wide-card layout", async () => {
    const service = api();
    renderHistory(service);

    const summary = await screen.findByText("旅行：旅行");
    const image = await screen.findByRole("img", { name: "原始截图" });
    const card = summary.closest("article");
    expect(card).not.toBeNull();
    expect(card?.firstElementChild).toBe(image);
    expect(card).toHaveTextContent("模型");
    expect(card).toHaveTextContent("日语学习解析");
    expect(service.getHistoryImage).toHaveBeenCalledWith("h1", "original");

    const styles = nodeProcess
      .getBuiltinModule("node:fs")
      .readFileSync(`${nodeProcess.cwd()}/src/styles.css`, "utf8");
    const itemRule = styles.match(/\.history-item\s*\{([^}]*)\}/)?.[1];
    const imageRule = styles.match(/\.history-item__image\s*\{([^}]*)\}/)?.[1];

    expect(itemRule).toMatch(/grid-template-columns:\s*minmax\(0, 1fr\);/);
    expect(imageRule).toMatch(/width:\s*auto;/);
    expect(imageRule).toMatch(/max-width:\s*100%;/);
    expect(imageRule).toMatch(/height:\s*auto;/);
    expect(imageRule).toMatch(/max-height:\s*280px;/);
    expect(imageRule).not.toMatch(/aspect-ratio:/);
    expect(imageRule).not.toMatch(/min-height:/);
  });

  it("paginates with bounded cursor queries and selectable page sizes", async () => {
    const secondItem = {
      ...item,
      id: "h2",
      resultPreview: "第二页结果",
      hasImage: false,
    };
    const service = api();
    vi.mocked(service.queryHistory)
      .mockResolvedValueOnce({ items: [item], nextCursor: "cursor-2" })
      .mockResolvedValueOnce({ items: [secondItem], nextCursor: null })
      .mockResolvedValueOnce({ items: [item], nextCursor: "cursor-2" })
      .mockResolvedValueOnce({ items: [item], nextCursor: null });

    renderHistory(service);

    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    expect(service.queryHistory).toHaveBeenNthCalledWith(1, { limit: 10 });
    expect(screen.getByText("第 1 页")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "上一页" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "下一页" }));
    const secondSummary = await screen.findByText("第二页结果");
    expect(secondSummary.closest("article")?.firstElementChild).toHaveClass(
      "history-item__content",
    );
    expect(service.queryHistory).toHaveBeenNthCalledWith(2, {
      text: undefined,
      promptName: undefined,
      status: undefined,
      cursor: "cursor-2",
      limit: 10,
    });
    expect(screen.getByText("第 2 页")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "下一页" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "上一页" }));
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    expect(service.queryHistory).toHaveBeenNthCalledWith(3, {
      text: undefined,
      promptName: undefined,
      status: undefined,
      cursor: undefined,
      limit: 10,
    });
    expect(screen.getByText("第 1 页")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("每页条目数"), {
      target: { value: "20" },
    });
    await waitFor(() =>
      expect(service.queryHistory).toHaveBeenNthCalledWith(4, {
        text: undefined,
        promptName: undefined,
        status: undefined,
        cursor: undefined,
        limit: 20,
      }),
    );
    expect(screen.getByLabelText("每页条目数")).toHaveValue("20");
    expect(screen.getByText("第 1 页")).toBeInTheDocument();
  });

  it("opens a dedicated detail view and returns to the preserved list state", async () => {
    const service = api();
    renderHistory(service);
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("搜索结果"), {
      target: { value: "旅行" },
    });
    fireEvent.change(screen.getByLabelText("状态"), {
      target: { value: "success" },
    });
    fireEvent.click(screen.getByRole("button", { name: "搜索" }));
    await waitFor(() =>
      expect(service.queryHistory).toHaveBeenLastCalledWith(
        expect.objectContaining({ text: "旅行", status: "success" }),
      ),
    );
    const queryCount = vi.mocked(service.queryHistory).mock.calls.length;
    const scrollContainer = document.querySelector(
      ".settings-content",
    ) as HTMLElement;
    scrollContainer.scrollTop = 240;
    fireEvent.click(screen.getByRole("button", { name: "查看详情" }));
    expect(
      await screen.findByText("旅行（りょこう）：旅行"),
    ).toBeInTheDocument();
    const thinking = screen.getByText("思考过程").closest("details");
    expect(thinking).not.toHaveAttribute("open");
    expect(screen.getByText("先识别文字，再翻译")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "返回历史记录" }),
    ).toBeInTheDocument();
    expect(scrollContainer.scrollTop).toBe(0);
    expect(screen.queryByLabelText("搜索结果")).not.toBeInTheDocument();
    expect(screen.queryByText("旅行：旅行")).not.toBeInTheDocument();
    expect(await screen.findByLabelText("模型配置")).toHaveValue(
      "model-original",
    );
    expect(screen.getByLabelText("提示词配置")).toHaveValue("prompt-original");
    fireEvent.click(screen.getByRole("button", { name: "复制结果" }));
    fireEvent.change(screen.getByLabelText("模型配置"), {
      target: { value: "model-other" },
    });
    fireEvent.change(screen.getByLabelText("提示词配置"), {
      target: { value: "prompt-other" },
    });
    fireEvent.click(screen.getByRole("button", { name: "重新选择配置提交" }));
    expect(service.copyText).toHaveBeenCalledWith("旅行（りょこう）：旅行");
    expect(service.resubmitHistory).toHaveBeenCalledWith(
      "h1",
      "model-other",
      "prompt-other",
    );
    expect(await screen.findByText("结果已复制")).toBeInTheDocument();
    expect(await screen.findByText("已重新提交")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "返回历史记录" }));
    expect(await screen.findByLabelText("搜索结果")).toHaveValue("旅行");
    expect(screen.getByLabelText("状态")).toHaveValue("success");
    expect(screen.getByText("旅行：旅行")).toBeInTheDocument();
    expect(scrollContainer.scrollTop).toBe(240);
    expect(service.queryHistory).toHaveBeenCalledTimes(queryCount);
  });

  it("keeps the list available when detail loading fails", async () => {
    const service = api();
    vi.mocked(service.getHistoryEntry).mockRejectedValue({
      message: "详情加载失败",
    });
    renderHistory(service);
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "查看详情" }));
    expect(await screen.findByText("详情加载失败")).toBeInTheDocument();
    expect(screen.getByLabelText("搜索结果")).toBeInTheDocument();
    expect(screen.getByText("旅行：旅行")).toBeInTheDocument();
  });

  it("falls back for legacy entries and disables submission without choices", async () => {
    const legacyService = api();
    vi.mocked(legacyService.getHistoryEntry).mockResolvedValue({
      ...detail,
      promptConfigId: null,
      modelConfigId: null,
      promptName: "不存在的提示词",
      modelConfigName: "不存在的模型",
    });
    const legacyView = renderHistory(legacyService);
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "查看详情" }));
    await waitFor(() =>
      expect(screen.getByLabelText("模型配置")).toHaveValue("model-other"),
    );
    expect(screen.getByLabelText("提示词配置")).toHaveValue("prompt-other");
    legacyView.unmount();

    const emptyService = api();
    vi.mocked(emptyService.listModelConfigs).mockResolvedValue([]);
    vi.mocked(emptyService.listPromptPresets).mockResolvedValue([]);
    renderHistory(emptyService);
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "查看详情" }));
    expect(await screen.findByText("没有可用模型配置")).toBeInTheDocument();
    expect(screen.getByText("没有可用提示词配置")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "重新选择配置提交" }),
    ).toBeDisabled();
  });

  it("preserves line breaks and blank lines in result summaries", async () => {
    const service = api([
      {
        ...item,
        resultPreview: "第一行\n\n第三行",
      },
    ]);
    renderHistory(service);
    const summary = await screen.findByText(
      (_, element) => element?.textContent === "第一行\n\n第三行",
    );
    expect(summary.tagName).toBe("PRE");
    expect(summary).toHaveClass("history-item__summary");
  });

  it("shows empty/no-result states and confirms single/all deletion", async () => {
    const emptyService = api([]);
    const { rerender } = renderHistory(emptyService);
    expect(await screen.findByText("没有历史记录")).toBeInTheDocument();
    const service = api();
    rerender(
      <NotificationProvider>
        <div className="settings-content">
          <History api={service} />
        </div>
      </NotificationProvider>,
    );
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除记录" }));
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "删除历史记录？" })).getByRole(
        "button",
        { name: "删除记录" },
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "清空全部历史" }));
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "清空全部历史？" })).getByRole(
        "button",
        { name: "确认清空" },
      ),
    );
    await waitFor(() =>
      expect(service.deleteHistoryEntry).toHaveBeenCalledWith("h1"),
    );
    expect(service.clearHistory).toHaveBeenCalled();
    expect(await screen.findByText("历史记录已清空")).toBeInTheDocument();
  });
});
