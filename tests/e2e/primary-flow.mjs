/* global document, getComputedStyle, window */

import assert from "node:assert/strict";

const snapshot = {
  settings: {
    activeModelConfigId: "model-1",
    activePromptId: "prompt-1",
    captureShortcut: "Command+Shift+X",
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

async function assertCurrentPage(button) {
  await button.waitFor({ state: "visible" });
  assert.equal(await button.getAttribute("aria-current"), "page");
}

export async function runPrimaryFlow(page) {
  await page.addInitScript(
    ({ initialResults }) => {
      const testBridge = { calls: [], results: initialResults };
      window.__SEE_SEE_TEST__ = testBridge;
      window.__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main" },
        },
        invoke(command, args) {
          testBridge.calls.push({ command, args });
          return Object.hasOwn(testBridge.results, command)
            ? Promise.resolve(testBridge.results[command])
            : Promise.reject(
                new Error(`Tauri backend unavailable: ${command}`),
              );
        },
      };
    },
    {
      initialResults: {
        get_app_snapshot: snapshot,
        get_settings: snapshot.settings,
        list_model_configs: [],
        list_prompt_presets: [],
        save_model_config: { id: "model-1" },
        copy_text: null,
        query_history: { items: [], nextCursor: null },
        "plugin:app|version": "0.3.1",
      },
    },
  );
  await page.goto("http://127.0.0.1:1420/");

  const sidebar = page.getByRole("navigation", { name: "设置栏目" });
  await sidebar.waitFor({ state: "visible" });
  await assertCurrentPage(
    sidebar.getByRole("button", { name: "常规", exact: true }),
  );

  await page.setViewportSize({ width: 1440, height: 900 });
  const desktopLayout = await page.evaluate(() => {
    const shell = document.querySelector(".settings-shell");
    const sidebarElement = document.querySelector(".settings-sidebar");
    const content = document.querySelector(".settings-content");
    if (!shell || !sidebarElement || !content) return null;
    return {
      columns: getComputedStyle(shell).gridTemplateColumns,
      sidebarHeight: sidebarElement.clientHeight,
      contentScrollable: getComputedStyle(content).overflowY,
    };
  });
  assert.notEqual(desktopLayout, null);
  assert.equal(desktopLayout.columns.split(" ").length, 2);
  assert.ok(desktopLayout.sidebarHeight > 700);
  assert.equal(desktopLayout.contentScrollable, "auto");

  await page.setViewportSize({ width: 720, height: 520 });
  const compactLayout = await page.evaluate(() => {
    const sidebarElement = document.querySelector(".settings-sidebar");
    const nav = document.querySelector(".settings-nav");
    if (!sidebarElement || !nav) return null;
    return {
      sidebarDisplay: getComputedStyle(sidebarElement).display,
      navDisplay: getComputedStyle(nav).display,
      noPageOverflow:
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    };
  });
  assert.deepEqual(compactLayout, {
    sidebarDisplay: "grid",
    navDisplay: "flex",
    noPageOverflow: true,
  });
  await page.setViewportSize({ width: 1024, height: 720 });

  assert.equal(
    await page.getByRole("button", { name: "开始截图", exact: true }).count(),
    0,
  );

  for (const name of ["模型", "提示词", "历史", "关于"]) {
    const button = sidebar.getByRole("button", { name, exact: true });
    await button.click();
    await assertCurrentPage(button);

    if (name === "模型") {
      assert.equal(await page.getByLabel("配置名称").count(), 0);
      await page.getByRole("button", { name: "新增配置" }).click();
      await page.getByLabel("配置名称").waitFor({ state: "visible" });
      await page.getByRole("button", { name: "取消" }).click();
      assert.equal(await page.getByLabel("配置名称").count(), 0);
      await page.evaluate(() =>
        window.__TAURI_INTERNALS__.invoke("save_model_config", {
          input: { name: "测试模型" },
        }),
      );
    }
    if (name === "历史") {
      await page.evaluate(() =>
        window.__TAURI_INTERNALS__.invoke("copy_text", { text: "旅行：旅行" }),
      );
    }
  }

  const calls = await page.evaluate(() => window.__SEE_SEE_TEST__.calls);
  assert.equal(calls.filter((call) => call.command === "open_view").length, 0);
  assert.equal(
    calls.filter((call) => call.command === "begin_capture").length,
    0,
  );
  assert.equal(
    calls.filter((call) => call.command === "save_model_config").length,
    1,
  );
  assert.deepEqual(calls.find((call) => call.command === "copy_text")?.args, {
    text: "旅行：旅行",
  });
  assert.ok(
    calls.filter((call) => call.command === "list_model_configs").length >= 1,
  );
  assert.ok(
    calls.filter((call) => call.command === "list_prompt_presets").length >= 1,
  );
  assert.ok(
    calls.filter((call) => call.command === "query_history").length >= 1,
  );
}
