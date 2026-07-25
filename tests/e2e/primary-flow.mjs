/* global document, getComputedStyle, window */

import assert from "node:assert/strict";

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

async function assertCurrentPage(button) {
  await button.waitFor({ state: "visible" });
  assert.equal(await button.getAttribute("aria-current"), "page");
}

export async function runPrimaryFlow(page) {
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

  await page.setViewportSize({ width: 720, height: 720 });
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

  await page.evaluate(
    (values) => Object.assign(window.__SEE_SEE_TEST__.results, values),
    {
      get_app_snapshot: snapshot,
      get_settings: snapshot.settings,
      list_model_configs: [],
      list_prompt_presets: [],
      save_model_config: { id: "model-1" },
      copy_text: null,
      query_history: { items: [], nextCursor: null },
      "plugin:app|version": "0.2.1",
    },
  );

  assert.equal(
    await page.getByRole("button", { name: "开始截图", exact: true }).count(),
    0,
  );

  for (const name of ["模型", "提示词", "历史", "关于"]) {
    const button = sidebar.getByRole("button", { name, exact: true });
    await button.click();
    await assertCurrentPage(button);

    if (name === "模型") {
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
