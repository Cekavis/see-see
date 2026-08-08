import { describe, expect, it, vi } from "vitest";
import { getErrorMessage, ipc } from "./ipc";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("IPC error messages", () => {
  it("supports structured, Error, and string rejections without exposing objects", () => {
    expect(
      getErrorMessage({
        code: "invalid_input",
        message: "模型 ID 不能为空",
        retryable: false,
      }),
    ).toBe("模型 ID 不能为空");
    expect(getErrorMessage(new Error("网络不可用"))).toBe("网络不可用");
    expect(getErrorMessage("invalid args for command")).toBe(
      "invalid args for command",
    );
    expect(getErrorMessage({ rawResponse: "secret" })).toBe("操作失败，请重试");
  });
});

describe("IPC history resubmission", () => {
  it("passes the locally selected model and prompt identities", async () => {
    invoke.mockResolvedValue({ runId: "run-2" });

    await ipc.resubmitHistory("history-1", "model-2", "prompt-2");

    expect(invoke).toHaveBeenCalledWith("resubmit_history", {
      id: "history-1",
      modelConfigId: "model-2",
      promptConfigId: "prompt-2",
    });
  });
});
