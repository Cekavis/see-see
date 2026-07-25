import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
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
    const editor = screen.getByRole("region", { name: "模型配置编辑器" });
    expect(await within(editor).findByRole("alert")).toHaveTextContent(
      "无法获取模型列表",
    );
    expect(screen.getByLabelText("模型 ID")).toBeEnabled();
    expect(
      screen.getByText("连接测试会发送一张极小图片，可能产生少量调用费用。"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));
    expect(await screen.findByText("API Key 无效")).toBeInTheDocument();
  });

  it("shows progress for all three actions and prevents concurrent requests", async () => {
    const listRequest =
      deferred<Awaited<ReturnType<SettingsApi["listRemoteModels"]>>>();
    const testRequest =
      deferred<Awaited<ReturnType<SettingsApi["testModelConfig"]>>>();
    const saveRequest =
      deferred<Awaited<ReturnType<SettingsApi["saveModelConfig"]>>>();
    const service = api({
      listRemoteModels: vi.fn(() => listRequest.promise),
      testModelConfig: vi.fn(() => testRequest.promise),
      saveModelConfig: vi.fn(() => saveRequest.promise),
    });
    render(<Settings api={service} />);

    fireEvent.click(screen.getByRole("button", { name: "获取模型列表" }));
    expect(screen.getByRole("button", { name: "正在获取…" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "测试连接" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "保存配置" })).toBeDisabled();
    listRequest.resolve([]);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "获取模型列表" }),
      ).toBeEnabled(),
    );

    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));
    expect(screen.getByRole("button", { name: "正在测试…" })).toBeDisabled();
    testRequest.resolve({ passed: true, latencyMs: 20, error: null });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "测试连接" })).toBeEnabled(),
    );

    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    expect(screen.getByRole("button", { name: "正在保存…" })).toBeDisabled();
    saveRequest.resolve({
      id: "model-1",
      name: "视觉模型",
      protocol: "openai",
      baseUrl: "https://api.openai.com/v1",
      modelId: "vision",
      hasApiKey: false,
      testStatus: "untested",
      testedAt: null,
      testErrorCode: null,
      isActive: false,
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "保存配置" })).toBeEnabled(),
    );
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
    render(<Settings api={service} />);
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
  });
});
