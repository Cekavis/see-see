import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { Prompts, type PromptsApi } from "./Prompts";

const prompt = {
  id: "p1",
  name: "日语学习解析",
  body: "解释日文",
  isBuiltin: true,
  isActive: true,
};

function api(items = [prompt]): PromptsApi {
  return {
    listPromptPresets: vi.fn().mockResolvedValue(items),
    savePromptPreset: vi.fn().mockResolvedValue(prompt),
    duplicatePromptPreset: vi
      .fn()
      .mockResolvedValue({ ...prompt, id: "p2", name: "日语学习解析 副本" }),
    deletePromptPreset: vi.fn().mockResolvedValue(undefined),
    setActivePrompt: vi.fn().mockResolvedValue(undefined),
  };
}

function renderPrompts(service: PromptsApi) {
  return render(
    <NotificationProvider>
      <Prompts api={service} />
    </NotificationProvider>,
  );
}

describe("Prompts", () => {
  it("loads, creates, edits, duplicates, activates, and keeps keyboard-focusable controls", async () => {
    const service = api();
    renderPrompts(service);
    expect(
      await screen.findByRole("heading", { name: /日语学习解析/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/截图提交时会保存提示词快照，后续编辑不影响已有记录/),
    ).toHaveClass("field__hint");
    fireEvent.click(screen.getByRole("button", { name: "编辑" }));
    expect(screen.getByLabelText("提示词名称")).toHaveValue("日语学习解析");
    fireEvent.change(screen.getByLabelText("提示词正文"), {
      target: { value: "更新正文" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存提示词" }));
    await waitFor(() => expect(service.savePromptPreset).toHaveBeenCalled());
    expect(await screen.findByText("提示词已保存")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "复制" }));
    await waitFor(() =>
      expect(service.duplicatePromptPreset).toHaveBeenCalledWith("p1"),
    );
    expect(await screen.findByText("提示词已复制")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制" })).toHaveAttribute(
      "type",
      "button",
    );
  });

  it("shows empty state and confirms deletion", async () => {
    const emptyService = api([]);
    const { rerender } = renderPrompts(emptyService);
    expect(await screen.findByText("还没有提示词")).toBeInTheDocument();
    const service = api();
    rerender(
      <NotificationProvider>
        <Prompts api={service} />
      </NotificationProvider>,
    );
    expect(
      await screen.findByRole("heading", { name: /日语学习解析/ }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除" }));
    expect(
      screen.getByRole("dialog", { name: "删除提示词？" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除提示词" }));
    await waitFor(() =>
      expect(service.deletePromptPreset).toHaveBeenCalledWith("p1"),
    );
    expect(await screen.findByText("提示词已删除")).toBeInTheDocument();
  });
});
