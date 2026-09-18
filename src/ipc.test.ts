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

describe("IPC result navigation", () => {
  it("passes the current analysis identity when opening the main window", async () => {
    invoke.mockResolvedValue(undefined);

    await ipc.openMainWindow("run-1");

    expect(invoke).toHaveBeenCalledWith("open_main_window", { runId: "run-1" });
  });
});

describe("IPC result image", () => {
  it("passes the current analysis identity when loading the screenshot", async () => {
    invoke.mockResolvedValue(new ArrayBuffer(2));

    await ipc.getAnalysisImage("run-1");

    expect(invoke).toHaveBeenCalledWith("get_analysis_image", {
      runId: "run-1",
    });
  });
});

describe("IPC WebDAV configuration sync", () => {
  it("uses typed settings and sync command payloads", async () => {
    invoke.mockResolvedValue({
      url: "https://dav.example.test/root",
      username: "alice",
      remoteRoot: "see-see",
      hasPassword: true,
    });

    await ipc.getWebdavSettings();
    await ipc.saveWebdavSettings({
      url: "https://dav.example.test/root",
      username: "alice",
      remoteRoot: "see-see",
      password: "secret",
    });
    await ipc.uploadConfiguration();
    await ipc.downloadConfiguration();

    expect(invoke).toHaveBeenCalledWith("get_webdav_settings");
    expect(invoke).toHaveBeenCalledWith("save_webdav_settings", {
      input: {
        url: "https://dav.example.test/root",
        username: "alice",
        remoteRoot: "see-see",
        password: "secret",
      },
    });
    expect(invoke).toHaveBeenCalledWith("upload_configuration");
    expect(invoke).toHaveBeenCalledWith("download_configuration");
  });
});
