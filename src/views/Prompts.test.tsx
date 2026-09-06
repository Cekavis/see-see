import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { Prompts, type PromptsApi } from "./Prompts";

const prompt = {
  id: "p1",
  name: "日语学习解析",
  body: "解释日文",
  isBuiltin: true,
  captureShortcut: null,
};

function api(items = [prompt]): PromptsApi {
  return {
    listPromptPresets: vi.fn().mockResolvedValue(items),
    savePromptPreset: vi.fn().mockResolvedValue(prompt),
    duplicatePromptPreset: vi
      .fn()
      .mockResolvedValue({ ...prompt, id: "p2", name: "日语学习解析 副本" }),
    deletePromptPreset: vi.fn().mockResolvedValue(undefined),
    setPromptShortcut: vi.fn().mockResolvedValue(prompt),
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
  it("loads, creates, edits, duplicates, and keeps keyboard-focusable controls", async () => {
    const service = api();
    renderPrompts(service);
    expect(
      await screen.findByRole("heading", { name: /日语学习解析/ }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "编辑" }));
    expect(screen.getByLabelText("提示词名称")).toHaveValue("日语学习解析");
    expect(
      screen.getByText(/截图提交时会保存提示词快照，后续编辑不影响已有记录/),
    ).toHaveClass("field__hint");
    fireEvent.change(screen.getByLabelText("提示词正文"), {
      target: { value: "更新正文" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存" }));
    await waitFor(() => expect(service.savePromptPreset).toHaveBeenCalled());
    expect(await screen.findByText("提示词已保存")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "克隆" }));
    await waitFor(() =>
      expect(service.duplicatePromptPreset).toHaveBeenCalledWith("p1"),
    );
    expect(await screen.findByText("提示词已克隆")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "克隆" })).toHaveAttribute(
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

  it("records, clears, and cancels a prompt shortcut", async () => {
    const service = api();
    service.setPromptShortcut = vi
      .fn()
      .mockResolvedValue({ ...prompt, captureShortcut: "Ctrl+X" });
    renderPrompts(service);
    const shortcut = await screen.findByRole("button", {
      name: "日语学习解析截图快捷键",
    });
    fireEvent.click(shortcut);
    expect(shortcut).toHaveTextContent("请按新的快捷键…");
    fireEvent.keyDown(window, {
      key: "x",
      code: "KeyX",
      ctrlKey: true,
    });
    await waitFor(() =>
      expect(service.setPromptShortcut).toHaveBeenCalledWith("p1", "Ctrl+X"),
    );
    expect(await screen.findByText("快捷键已保存并生效")).toBeInTheDocument();

    fireEvent.click(shortcut);
    fireEvent.keyDown(window, { key: "Escape", code: "Escape" });
    expect(shortcut).toHaveTextContent("Ctrl+X");
    expect(service.setPromptShortcut).toHaveBeenCalledTimes(1);

    fireEvent.click(shortcut);
    fireEvent.keyDown(window, { key: "Delete", code: "Delete" });
    await waitFor(() =>
      expect(service.setPromptShortcut).toHaveBeenLastCalledWith("p1", null),
    );
  });
});
