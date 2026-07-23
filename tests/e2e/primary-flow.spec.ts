import { browser, expect, $ } from "@wdio/globals";

const snapshot = {
  settings: {
    activeModelConfigId: "model-1",
    activePromptId: "prompt-1",
    captureShortcut: "CommandOrControl+Shift+X",
    saveHistory: true,
    autostart: false,
    resultAlwaysOnTop: true,
    onboardingCompleted: true,
  },
  promptCount: 2,
  modelConfigCount: 1,
  activePromptId: "prompt-1",
  activeModelConfigId: "model-1",
  screenPermission: "granted",
};

describe("See See primary desktop flow", () => {
  it("connects model configuration, capture, result, and history IPC from the desktop shell", async () => {
    await browser.url("http://127.0.0.1:1420/");
    await expect($("[role=alert]")).toBeDisplayed();

    await browser.execute(
      (values) => Object.assign(window.__SEE_SEE_TEST__.results, values),
      {
        get_app_snapshot: snapshot,
        open_view: null,
        save_model_config: { id: "model-1" },
        begin_capture: { sessionId: "capture-1", monitors: [] },
        copy_text: null,
        query_history: { items: [{ id: "history-1" }], nextCursor: null },
      },
    );

    await $("button=重试").click();
    await expect($("h1=See See")).toBeDisplayed();

    await $("button=模型设置").click();
    await browser.execute(() =>
      window.__TAURI_INTERNALS__.invoke("save_model_config", {
        input: { name: "测试模型" },
      }),
    );
    await $("button=开始截图").click();
    await browser.execute(() =>
      window.__TAURI_INTERNALS__.invoke("copy_text", { text: "旅行：旅行" }),
    );
    await browser.execute(() =>
      window.__TAURI_INTERNALS__.invoke("query_history", { query: {} }),
    );
    await $("button=历史记录").click();

    const calls = await browser.execute(() => window.__SEE_SEE_TEST__.calls);
    expect(
      calls
        .filter((call) => call.command === "open_view")
        .map((call) => call.args),
    ).toEqual([{ view: "settings" }, { view: "history" }]);
    expect(
      calls.filter((call) => call.command === "save_model_config"),
    ).toHaveLength(1);
    expect(
      calls.filter((call) => call.command === "begin_capture"),
    ).toHaveLength(1);
    expect(calls.find((call) => call.command === "copy_text")?.args).toEqual({
      text: "旅行：旅行",
    });
    expect(
      calls.filter((call) => call.command === "query_history"),
    ).toHaveLength(1);
  });
});
