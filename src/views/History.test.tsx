import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { History, type HistoryApi } from "./History";

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
  resultText: "旅行（りょこう）：旅行",
  errorCode: null,
  promptBody: "解释",
  protocol: "openai",
};

function api(items = [item]): HistoryApi {
  return {
    queryHistory: vi.fn().mockResolvedValue({ items, nextCursor: null }),
    getHistoryEntry: vi.fn().mockResolvedValue(detail),
    getHistoryImage: vi.fn().mockResolvedValue(new ArrayBuffer(2)),
    resubmitHistory: vi.fn().mockResolvedValue({ runId: "run-2" }),
    deleteHistoryEntry: vi.fn().mockResolvedValue(undefined),
    clearHistory: vi.fn().mockResolvedValue({ deletedCount: 1 }),
    copyText: vi.fn().mockResolvedValue(undefined),
  };
}

describe("History", () => {
  it("loads, searches, filters, opens detail, copies, and resubmits", async () => {
    const service = api();
    render(<History api={service} />);
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
    fireEvent.click(screen.getByRole("button", { name: "查看详情" }));
    expect(
      await screen.findByText("旅行（りょこう）：旅行"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "复制结果" }));
    fireEvent.click(
      screen.getByRole("button", { name: "使用当前配置再次提交" }),
    );
    expect(service.copyText).toHaveBeenCalledWith("旅行（りょこう）：旅行");
    expect(service.resubmitHistory).toHaveBeenCalledWith("h1");
  });

  it("shows empty/no-result states and confirms single/all deletion", async () => {
    const emptyService = api([]);
    const { rerender } = render(<History api={emptyService} />);
    expect(await screen.findByText("没有历史记录")).toBeInTheDocument();
    const service = api();
    vi.spyOn(window, "confirm").mockReturnValue(true);
    rerender(<History api={service} />);
    expect(await screen.findByText("旅行：旅行")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除记录" }));
    fireEvent.click(screen.getByRole("button", { name: "清空全部历史" }));
    await waitFor(() =>
      expect(service.deleteHistoryEntry).toHaveBeenCalledWith("h1"),
    );
    expect(service.clearHistory).toHaveBeenCalled();
  });
});
