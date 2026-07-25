import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { Settings, type SettingsApi } from "./Settings";

function api(overrides: Partial<SettingsApi> = {}): SettingsApi {
  return {
    listModelConfigs: vi.fn().mockResolvedValue([]),
    saveModelConfig: vi.fn().mockImplementation(async (input) => ({
      id: "model-1",
      name: input.name,
      protocol: input.protocol,
      baseUrl: input.baseUrl,
      modelId: input.modelId,
      hasApiKey: Boolean(input.apiKey),
      testStatus: "untested",
      testedAt: null,
      testErrorCode: null,
      isActive: false,
    })),
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

describe("model settings", () => {
  it("supports preset endpoints, manual model IDs, and clears the key after save", async () => {
    const service = api();
    renderSettings(service);
    fireEvent.change(screen.getByLabelText("配置名称"), {
      target: { value: "我的 OpenAI" },
    });
    fireEvent.change(screen.getByLabelText("模型 ID"), {
      target: { value: "gpt-vision" },
    });
    fireEvent.change(screen.getByLabelText("API Key"), {
      target: { value: "secret" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await waitFor(() => expect(service.saveModelConfig).toHaveBeenCalled());
    expect(await screen.findByRole("status")).toHaveTextContent("配置已保存");
    expect(screen.getByLabelText("API Key")).toHaveValue("");
    expect(service.saveModelConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        baseUrl: "https://api.openai.com/v1",
        modelId: "gpt-vision",
        apiKey: "secret",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await waitFor(() =>
      expect(service.saveModelConfig).toHaveBeenCalledTimes(2),
    );
    expect(service.saveModelConfig).toHaveBeenLastCalledWith(
      expect.objectContaining({
        id: "model-1",
        apiKey: undefined,
      }),
    );
  });

  it("saves the key before testing and activates a passing model", async () => {
    const service = api();
    renderSettings(service);
    fireEvent.change(screen.getByLabelText("配置名称"), {
      target: { value: "我的 OpenAI" },
    });
    fireEvent.change(screen.getByLabelText("模型 ID"), {
      target: { value: "gpt-vision" },
    });
    fireEvent.change(screen.getByLabelText("API Key"), {
      target: { value: "secret" },
    });

    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));

    await waitFor(() =>
      expect(service.saveModelConfig).toHaveBeenCalledWith(
        expect.objectContaining({ apiKey: "secret" }),
      ),
    );
    await waitFor(() =>
      expect(service.testModelConfig).toHaveBeenCalledWith({
        id: "model-1",
        protocol: "openai",
        baseUrl: "https://api.openai.com/v1",
        modelId: "gpt-vision",
      }),
    );
    await waitFor(() =>
      expect(service.setActiveModelConfig).toHaveBeenCalledWith("model-1"),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "配置已保存、测试通过并设为当前模型",
    );
    expect(screen.getByLabelText("API Key")).toHaveValue("");

    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await waitFor(() =>
      expect(service.saveModelConfig).toHaveBeenCalledTimes(2),
    );
    expect(service.saveModelConfig).toHaveBeenLastCalledWith(
      expect.objectContaining({
        id: "model-1",
        apiKey: undefined,
      }),
    );
  });

  it("keeps manual input available when model listing fails and classifies connection state", async () => {
    const service = api({
      listRemoteModels: vi.fn().mockRejectedValue("无法获取模型列表"),
      testModelConfig: vi.fn().mockResolvedValue({
        passed: false,
        latencyMs: 15,
        error: {
          code: "auth_failed",
          message: "API Key 无效",
          retryable: false,
        },
      }),
    });
    renderSettings(service);
    fireEvent.click(screen.getByRole("button", { name: "获取模型列表" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "无法获取模型列表",
    );
    expect(screen.getByLabelText("模型 ID")).toBeEnabled();
    expect(
      screen.getByText(
        "连接测试会先保存当前配置，再发送一张极小图片，可能产生少量调用费用。",
      ),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));
    expect(await screen.findByText("API Key 无效")).toBeInTheDocument();
    expect(service.saveModelConfig).toHaveBeenCalled();
    expect(service.setActiveModelConfig).not.toHaveBeenCalled();
  });

  it("deletes a saved configuration only after dialog confirmation", async () => {
    const config = {
      id: "model-1",
      name: "视觉模型",
      protocol: "openai" as const,
      baseUrl: "https://api.openai.com/v1",
      modelId: "vision",
      hasApiKey: true,
      testStatus: "passed" as const,
      testedAt: "2026-07-23T00:00:00Z",
      testErrorCode: null,
      isActive: false,
    };
    const service = api({
      listModelConfigs: vi.fn().mockResolvedValue([config]),
    });
    renderSettings(service);
    expect(
      await screen.findByRole("heading", { name: "视觉模型" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "删除" }));
    expect(
      screen.getByRole("dialog", { name: "删除模型配置？" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "取消" }));
    expect(service.deleteModelConfig).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "删除" }));
    fireEvent.click(screen.getByRole("button", { name: "删除配置" }));
    await waitFor(() =>
      expect(service.deleteModelConfig).toHaveBeenCalledWith("model-1"),
    );
    expect(await screen.findByText("模型配置已删除")).toBeInTheDocument();
  });
});
