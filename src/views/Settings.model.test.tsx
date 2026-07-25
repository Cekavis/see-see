import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import type { ModelConfigSummary } from "../ipc";
import { Settings, type SettingsApi } from "./Settings";

const savedConfig: ModelConfigSummary = {
  id: "model-1",
  name: "视觉模型",
  protocol: "openai",
  baseUrl: "https://api.example.com/v1",
  modelId: "vision",
  hasApiKey: true,
  isActive: false,
};

function api(overrides: Partial<SettingsApi> = {}): SettingsApi {
  return {
    listModelConfigs: vi.fn().mockResolvedValue([]),
    saveModelConfig: vi.fn().mockImplementation(async (input) => ({
      ...savedConfig,
      id: input.id ?? savedConfig.id,
      name: input.name,
      protocol: input.protocol,
      baseUrl: input.baseUrl,
      modelId: input.modelId,
      hasApiKey: Boolean(input.apiKey),
    })),
    duplicateModelConfig: vi.fn().mockResolvedValue({
      ...savedConfig,
      id: "model-2",
      name: "视觉模型 副本",
    }),
    deleteModelConfig: vi.fn().mockResolvedValue(undefined),
    setActiveModelConfig: vi.fn().mockResolvedValue(undefined),
    listRemoteModels: vi.fn().mockResolvedValue([]),
    testModelConfig: vi
      .fn()
      .mockResolvedValue({ passed: true, latencyMs: 20, error: null }),
    ...overrides,
  };
}

function renderSettings(service: SettingsApi) {
  return render(
    <NotificationProvider>
      <Settings api={service} />
    </NotificationProvider>,
  );
}

function openNewEditor() {
  fireEvent.click(screen.getByRole("button", { name: "新增配置" }));
}

function fillRequiredFields() {
  fireEvent.change(screen.getByLabelText("配置名称"), {
    target: { value: "我的 OpenAI" },
  });
  fireEvent.change(screen.getByLabelText("模型 ID"), {
    target: { value: "gpt-vision" },
  });
}

describe("model settings", () => {
  it("keeps the editor out of the DOM until add and closes it on cancel", async () => {
    renderSettings(api());
    await screen.findByText("还没有模型配置");
    expect(screen.queryByLabelText("配置名称")).not.toBeInTheDocument();
    expect(
      screen.queryByText(/配置支持图片输入的 OpenAI/),
    ).not.toBeInTheDocument();

    openNewEditor();
    expect(
      screen.getByRole("heading", { name: "新增配置" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("配置名称")).toHaveFocus();
    expect(screen.getByRole("button", { name: "新增配置" })).toHaveAttribute(
      "aria-expanded",
      "true",
    );

    fireEvent.change(screen.getByLabelText("配置名称"), {
      target: { value: "未保存" },
    });
    fireEvent.click(screen.getByRole("button", { name: "取消" }));
    expect(screen.queryByLabelText("配置名称")).not.toBeInTheDocument();
  });

  it("saves without testing and closes the editor after success", async () => {
    const service = api();
    renderSettings(service);
    openNewEditor();
    fillRequiredFields();
    fireEvent.change(screen.getByLabelText("API Key"), {
      target: { value: "  secret  " },
    });

    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));

    await waitFor(() =>
      expect(service.saveModelConfig).toHaveBeenCalledWith(
        expect.objectContaining({
          baseUrl: "https://api.openai.com/v1",
          modelId: "gpt-vision",
          apiKey: "  secret  ",
        }),
      ),
    );
    expect(service.testModelConfig).not.toHaveBeenCalled();
    expect(service.setActiveModelConfig).not.toHaveBeenCalled();
    expect(await screen.findByRole("status")).toHaveTextContent("配置已保存");
    expect(screen.queryByLabelText("配置名称")).not.toBeInTheDocument();
  });

  it("tests the draft directly without saving or activating it", async () => {
    const service = api();
    renderSettings(service);
    openNewEditor();
    fillRequiredFields();
    fireEvent.change(screen.getByLabelText("API Key"), {
      target: { value: "draft-secret" },
    });

    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));

    await waitFor(() =>
      expect(service.testModelConfig).toHaveBeenCalledWith({
        protocol: "openai",
        baseUrl: "https://api.openai.com/v1",
        modelId: "gpt-vision",
        apiKey: "draft-secret",
      }),
    );
    expect(service.saveModelConfig).not.toHaveBeenCalled();
    expect(service.setActiveModelConfig).not.toHaveBeenCalled();
    expect(await screen.findByRole("status")).toHaveTextContent(
      "连接测试通过（20 ms）",
    );
    expect(screen.getByLabelText("配置名称")).toHaveValue("我的 OpenAI");
  });

  it("keeps an edited saved key unless explicitly cleared", async () => {
    const service = api({
      listModelConfigs: vi.fn().mockResolvedValue([savedConfig]),
      saveModelConfig: vi.fn().mockResolvedValue(savedConfig),
    });
    renderSettings(service);
    fireEvent.click(
      await screen.findByRole("button", { name: "编辑 视觉模型" }),
    );

    expect(
      screen.getByRole("heading", { name: "编辑配置" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("配置名称")).toHaveValue("视觉模型");
    expect(screen.getByLabelText("API Key")).toHaveValue("");
    expect(screen.getByText(/留空保留已保存的 Key/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await waitFor(() =>
      expect(service.saveModelConfig).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "model-1",
          apiKey: undefined,
          clearApiKey: false,
        }),
      ),
    );
    expect(screen.queryByLabelText("配置名称")).not.toBeInTheDocument();
  });

  it("keeps failed save values open for recovery", async () => {
    const service = api({
      saveModelConfig: vi.fn().mockRejectedValue("名称已存在"),
    });
    renderSettings(service);
    openNewEditor();
    fillRequiredFields();
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("名称已存在");
    expect(screen.getByLabelText("配置名称")).toHaveValue("我的 OpenAI");
  });

  it("copies and activates saved configurations without a test status gate", async () => {
    const service = api({
      listModelConfigs: vi.fn().mockResolvedValue([savedConfig]),
    });
    renderSettings(service);

    fireEvent.click(
      await screen.findByRole("button", { name: "复制 视觉模型" }),
    );
    await waitFor(() =>
      expect(service.duplicateModelConfig).toHaveBeenCalledWith("model-1"),
    );
    expect(await screen.findByText("模型配置已复制")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "设为当前 视觉模型" }));
    await waitFor(() =>
      expect(service.setActiveModelConfig).toHaveBeenCalledWith("model-1"),
    );
    expect(
      screen.queryByText(/测试通过|测试失败|未测试/),
    ).not.toBeInTheDocument();
  });

  it("disables conflicting actions while testing and restores them afterward", async () => {
    let resolveTest:
      | ((value: { passed: boolean; latencyMs: number; error: null }) => void)
      | undefined;
    const service = api({
      listModelConfigs: vi.fn().mockResolvedValue([savedConfig]),
      testModelConfig: vi.fn().mockImplementation(
        () =>
          new Promise((resolve) => {
            resolveTest = resolve;
          }),
      ),
    });
    renderSettings(service);
    fireEvent.click(
      await screen.findByRole("button", { name: "编辑 视觉模型" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));

    expect(screen.getByRole("button", { name: "新增配置" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "取消" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "复制 视觉模型" }),
    ).toBeDisabled();

    resolveTest?.({ passed: true, latencyMs: 20, error: null });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "取消" })).toBeEnabled(),
    );
  });

  it("deletes an edited configuration only after confirmation and closes the editor", async () => {
    const service = api({
      listModelConfigs: vi.fn().mockResolvedValue([savedConfig]),
    });
    renderSettings(service);
    fireEvent.click(
      await screen.findByRole("button", { name: "编辑 视觉模型" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "删除 视觉模型" }));
    expect(
      screen.getByRole("dialog", { name: "删除模型配置？" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除配置" }));
    await waitFor(() =>
      expect(service.deleteModelConfig).toHaveBeenCalledWith("model-1"),
    );
    expect(screen.queryByLabelText("配置名称")).not.toBeInTheDocument();
  });
});
