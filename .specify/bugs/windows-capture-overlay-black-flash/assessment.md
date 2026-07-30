# Bug Assessment: Windows 截图遮罩黑屏闪烁

- **Slug**: windows-capture-overlay-black-flash
- **Created**: 2026-07-30
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

> 现在虽然动画没有了，但仍然有一个黑屏闪一下的过程

## Symptom

Windows 截图遮罩已经不再播放系统窗口动画，但显示时仍先短暂出现黑色画面，随后才切换为冻结的屏幕截图。预期第一次可见帧就是已加载的截图。

## Reproduction

1. 在 Windows 安装版触发截图。
2. 观察遮罩首次显示的内容。
3. 遮罩先显示黑色背景，再切换为截图画面。

## Suspected Code Paths

- `src-tauri/src/commands.rs:769` — 后端在创建、定位窗口后立即调用 `show_capture_window`。
- `src/views/CaptureOverlay.tsx:56` — WebView 挂载后才异步调用 `get_capture_frame` 获取 PNG。
- `src/views/CaptureOverlay.tsx:64` — PNG 返回后才设置 `frameUrl`。
- `src/styles.css:1129` — `frameUrl` 尚未设置时 overlay 显示黑色 `background-color`，因此立即显示窗口必然暴露黑色占位帧。

## Root Cause Hypothesis

**Confidence: high.** capture window 的显示时机早于截图 PNG 的 IPC 读取、浏览器解码和 React 背景提交。系统动画修复只改变了窗口过渡，不改变内容准备顺序，所以黑色占位背景仍会作为首个可见帧出现。

## Proposed Remediation

**Preferred**: 保持 capture window 隐藏创建和原生 DWM 配置，但不在 `create_capture_windows` 中立即显示。前端取得截图 PNG 后先调用 `Image.decode()`，提交 `backgroundImage`；React effect 在该提交完成后调用一个受会话与显示器校验的 Rust IPC 命令，由后端显示对应 capture window。

**Alternatives**:

- 固定延时后显示；依赖机器性能，不能保证图片已准备好。
- 移除黑色背景；只会把闪烁改成透明或默认页面背景，不解决显示时机错误。

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/ipc.ts`
- `src/views/CaptureOverlay.tsx`
- `src/views/CaptureOverlay.test.tsx`
- `src-tauri/tests/desktop_lifecycle.rs`

**Tests to add or update**:

- 前端测试截图解码完成前不显示窗口，解码并提交背景后才调用 ready/show IPC。
- Rust 回归检查窗口创建路径不立即显示，并且 ready IPC 才调用 `show_capture_window`。
- 运行完整前端、Rust 和 Windows Tauri 发布构建；安装后复测首次可见帧。

## Risks & Considerations

- 图片读取或解码失败时，仍需显示窗口以呈现现有错误通知，避免留下不可见但活跃的截图会话。
- ready IPC 必须校验当前 capture session 和 monitor，不能允许任意窗口标签被显示。
- 多显示器窗口分别在各自截图准备好后显示，不需要全局等待最慢显示器。

## Open Questions

- 无。
