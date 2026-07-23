import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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

describe("model settings", () => {
  it("supports preset endpoints, manual model IDs, and clears the key after save", async () => {
    const service = api();
    render(<Settings api={service} />);
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
    expect(screen.getByLabelText("API Key")).toHaveValue("");
    expect(service.saveModelConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        baseUrl: "https://api.openai.com/v1",
        modelId: "gpt-vision",
        apiKey: "secret",
      }),
    );
  });

  it("keeps manual input available when model listing fails and classifies connection state", async () => {
    const service = api({
      listRemoteModels: vi.fn().mockRejectedValue({
        code: "network_unavailable",
        message: "无法获取模型列表",
      }),
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
    render(<Settings api={service} />);
    fireEvent.click(screen.getByRole("button", { name: "获取模型列表" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "无法获取模型列表",
    );
    expect(screen.getByLabelText("模型 ID")).toBeEnabled();
    expect(
      screen.getByText("连接测试会发送一张极小图片，可能产生少量调用费用。"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));
    expect(await screen.findByText("API Key 无效")).toBeInTheDocument();
  });
});
