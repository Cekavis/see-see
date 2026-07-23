# Bug Assessment: 首次启动配置模型窗口空白卡死

- **Slug**: first-launch-model-config-freeze
- **Created**: 2026-07-23
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 目前首次启动点击配置模型只会弹出空白窗口并且卡死，请你修复

## Symptom

首次启动且尚无模型配置时，从引导页点击“配置模型”会创建一个空白窗口，但设置页不会加载。预期窗口正常显示模型设置表单并保持应用可操作。

## Reproduction

1. 使用 `onboarding_completed = 0`、模型配置数为 0 的本机数据库启动已安装应用。
2. 在首次启动引导页点击“配置模型”。
3. WebView 调试目标新增一个 URL 为 `about:blank` 的窗口，未导航到 `http://tauri.localhost/`，设置页不显示。

## Suspected Code Paths

- `src/views/Onboarding.tsx:67` — 首次启动“配置模型”按钮调用 `openView("settings")`。
- `src/ipc.ts:146` — 前端通过 Tauri IPC 调用 `open_view`。
- `src-tauri/src/commands.rs:58` — `open_view` 当前是同步命令，并在处理 WebView IPC 时直接创建另一个 WebView 窗口。
- `src-tauri/src/commands.rs:72` — `WebviewWindowBuilder::build()` 在当前同步 IPC 回调中重入 Windows WebView 创建流程。
- `src-tauri/src/lib.rs:78` — 托盘菜单也直接调用同一个窗口打开函数，修改命令签名时需要同步调整。

## Root Cause Hypothesis

高置信度：Windows 上的同步 `open_view` IPC 命令在当前 WebView 消息回调线程中直接执行 `WebviewWindowBuilder::build()`，导致 WebView2 创建发生重入。窗口原生外壳被创建，但新 WebView 停留在 `about:blank`，命令也无法正常完成。设置页本身在零配置状态的组件测试中能正常渲染，因此不是空列表或凭据读取导致。

## Proposed Remediation

**Preferred**: 将 `open_view` 改为异步 Tauri 命令，使窗口创建在 Tauri 异步运行时中执行，避免在当前 WebView IPC 回调中重入创建。托盘入口继续调用同一个异步函数，交给 Tauri 异步运行时执行。

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 用首次启动数据库重新运行应用，点击“配置模型”，确认新窗口导航到应用 URL 并显示“模型设置”。
- 运行现有前端与 Rust 测试、构建，确认命令签名与托盘调用均可编译。

## Risks & Considerations

- 命令改为异步后，前端 IPC 类型不变，兼容性风险低。
- 托盘调用需要显式提交异步任务，否则会产生未执行的 Future。
- 其他动态窗口创建路径不由同步 `open_view` IPC 触发，不应扩大修改范围。

## Open Questions

- 无。
