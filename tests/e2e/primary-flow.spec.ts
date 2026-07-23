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
  it("keeps management in one sidebar window without capture or open_view controls", async () => {
    await browser.url("http://127.0.0.1:1420/");
    const sidebar = $('nav[aria-label="设置栏目"]');
    await expect(sidebar).toBeDisplayed();
    await expect(sidebar.$("button=常规")).toHaveAttribute(
      "aria-current",
      "page",
    );

    await browser.setWindowSize(1440, 900);
    const desktopLayout = await browser.execute(() => {
      const shell = document.querySelector<HTMLElement>(".settings-shell");
      const sidebarElement =
        document.querySelector<HTMLElement>(".settings-sidebar");
      const content = document.querySelector<HTMLElement>(".settings-content");
      if (!shell || !sidebarElement || !content) return null;
      return {
        columns: getComputedStyle(shell).gridTemplateColumns,
        sidebarHeight: sidebarElement.clientHeight,
        contentScrollable: getComputedStyle(content).overflowY,
      };
    });
    expect(desktopLayout).not.toBeNull();
    expect(desktopLayout?.columns.split(" ")).toHaveLength(2);
    expect(desktopLayout?.sidebarHeight).toBeGreaterThan(700);
    expect(desktopLayout?.contentScrollable).toBe("auto");

    await browser.setWindowSize(720, 720);
    const compactLayout = await browser.execute(() => {
      const sidebarElement =
        document.querySelector<HTMLElement>(".settings-sidebar");
      const nav = document.querySelector<HTMLElement>(".settings-nav");
      if (!sidebarElement || !nav) return null;
      return {
        sidebarDisplay: getComputedStyle(sidebarElement).display,
        navDisplay: getComputedStyle(nav).display,
        noPageOverflow:
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      };
    });
    expect(compactLayout).toEqual({
      sidebarDisplay: "grid",
      navDisplay: "flex",
      noPageOverflow: true,
    });
    await browser.setWindowSize(1024, 720);

    await browser.execute(
      (values) => Object.assign(window.__SEE_SEE_TEST__.results, values),
      {
        get_app_snapshot: snapshot,
        get_settings: snapshot.settings,
        list_model_configs: [],
        list_prompt_presets: [],
        save_model_config: { id: "model-1" },
        copy_text: null,
        query_history: { items: [], nextCursor: null },
        "plugin:app|version": "0.2.0",
      },
    );

    await expect($("button=开始截图")).not.toExist();

    await sidebar.$("button=模型").click();
    await expect(sidebar.$("button=模型")).toHaveAttribute(
      "aria-current",
      "page",
    );
    await browser.execute(() =>
      window.__TAURI_INTERNALS__.invoke("save_model_config", {
        input: { name: "测试模型" },
      }),
    );
    await sidebar.$("button=提示词").click();
    await expect(sidebar.$("button=提示词")).toHaveAttribute(
      "aria-current",
      "page",
    );
    await sidebar.$("button=历史").click();
    await expect(sidebar.$("button=历史")).toHaveAttribute(
      "aria-current",
      "page",
    );
    await browser.execute(() =>
      window.__TAURI_INTERNALS__.invoke("copy_text", { text: "旅行：旅行" }),
    );
    await sidebar.$("button=关于").click();
    await expect(sidebar.$("button=关于")).toHaveAttribute(
      "aria-current",
      "page",
    );

    const calls = await browser.execute(() => window.__SEE_SEE_TEST__.calls);
    expect(calls.filter((call) => call.command === "open_view")).toHaveLength(
      0,
    );
    expect(
      calls.filter((call) => call.command === "begin_capture"),
    ).toHaveLength(0);
    expect(
      calls.filter((call) => call.command === "save_model_config"),
    ).toHaveLength(1);
    expect(calls.find((call) => call.command === "copy_text")?.args).toEqual({
      text: "旅行：旅行",
    });
    expect(
      calls.filter((call) => call.command === "list_model_configs").length,
    ).toBeGreaterThanOrEqual(1);
    expect(
      calls.filter((call) => call.command === "list_prompt_presets").length,
    ).toBeGreaterThanOrEqual(1);
    expect(
      calls.filter((call) => call.command === "query_history").length,
    ).toBeGreaterThanOrEqual(1);
  });
});
