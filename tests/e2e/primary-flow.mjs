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
      let callbackId = 0;
      window.__SEE_SEE_TEST__ = testBridge;
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener() {},
      };
      window.__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main" },
        },
        transformCallback(callback) {
          const id = ++callbackId;
          window[`_${id}`] = callback;
          return id;
        },
        unregisterCallback(id) {
          delete window[`_${id}`];
        },
        invoke(command, args) {
          testBridge.calls.push({ command, args });
          if (command === "plugin:event|listen") return Promise.resolve(1);
          if (command === "plugin:event|unlisten") return Promise.resolve();
          if (command === "save_webdav_settings") {
            const previous = testBridge.results.get_webdav_settings;
            const { url, username, remoteRoot, password, clearPassword } =
              args.input;
            const saved = {
              url,
              username,
              remoteRoot,
              hasPassword:
                !clearPassword && (Boolean(password) || previous.hasPassword),
            };
            testBridge.results.get_webdav_settings = saved;
            return Promise.resolve(saved);
          }
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
        get_webdav_settings: {
          url: "",
          username: "",
          remoteRoot: "see-see",
          hasPassword: false,
        },
        upload_configuration: { models: 1, prompts: 2 },
        download_configuration: { models: 1, prompts: 2 },
        list_model_configs: [],
        list_prompt_presets: [],
        save_model_config: { id: "model-1" },
        copy_text: null,
        query_history: { items: [], nextCursor: null },
        "plugin:app|version": "0.3.2",
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
  const remoteRoot = page.getByLabel("远程根目录", { exact: true });
  await remoteRoot.waitFor({ state: "visible" });
  assert.equal(await remoteRoot.inputValue(), "see-see");
  assert.equal(
    await page
      .getByRole("button", { name: "上传配置", exact: true })
      .isEnabled(),
    false,
  );
  await page
    .getByLabel("WebDAV 地址", { exact: true })
    .fill("https://dav.example.com/dav");
  await page.getByLabel("用户名", { exact: true }).fill("sync-test-user");
  await page.getByLabel("密码", { exact: true }).fill("fixture-password");
  await remoteRoot.fill("shared/see-see");
  await page
    .getByRole("button", { name: "保存 WebDAV 设置", exact: true })
    .click();
  await page.getByText("WebDAV 设置已保存", { exact: true }).waitFor();
  assert.equal(await page.getByLabel("密码", { exact: true }).inputValue(), "");
  await page.getByRole("button", { name: "上传配置", exact: true }).click();
  await page
    .getByText("配置上传完成：模型 1 个，提示词 2 个", { exact: true })
    .waitFor();
  await page.getByRole("button", { name: "下载配置", exact: true }).click();
  const downloadDialog = page.getByRole("dialog", { name: "下载并合并配置" });
  await downloadDialog.waitFor({ state: "visible" });
  await downloadDialog
    .getByRole("button", { name: "取消", exact: true })
    .click();
  assert.equal(
    await page.evaluate(
      () =>
        window.__SEE_SEE_TEST__.calls.filter(
          (call) => call.command === "download_configuration",
        ).length,
    ),
    0,
  );
  await page.getByRole("button", { name: "下载配置", exact: true }).click();
  await downloadDialog
    .getByRole("button", { name: "下载并合并", exact: true })
    .click();
  await page
    .getByText("配置下载完成：模型 1 个，提示词 2 个", { exact: true })
    .waitFor();
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

  await sidebar.getByRole("button", { name: "常规", exact: true }).click();
  await remoteRoot.waitFor({ state: "visible" });
  assert.equal(await remoteRoot.inputValue(), "shared/see-see");
  assert.equal(await page.getByLabel("密码", { exact: true }).inputValue(), "");
  await page.getByLabel("清除已保存密码").waitFor({ state: "visible" });
  const calls = await page.evaluate(() => window.__SEE_SEE_TEST__.calls);
  assert.equal(
    calls.filter((call) => call.command === "upload_configuration").length,
    1,
  );
  assert.equal(
    calls.filter((call) => call.command === "download_configuration").length,
    1,
  );
  assert.deepEqual(
    calls.find((call) => call.command === "save_webdav_settings")?.args,
    {
      input: {
        url: "https://dav.example.com/dav",
        username: "sync-test-user",
        remoteRoot: "shared/see-see",
        password: "fixture-password",
      },
    },
  );
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
