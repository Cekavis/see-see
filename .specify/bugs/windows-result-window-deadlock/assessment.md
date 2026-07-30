# Bug Assessment: Windows 截图后结果窗口死锁

- **Slug**: windows-result-window-deadlock
- **Created**: 2026-07-30
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 之前只在 macos 上测试过可以使用，但是Windows现在试了一下截完图没有跳出来结果窗口，而是未响应卡死了，请你修复。

## Symptom

Windows 完成区域截图后，截图遮罩不会正常切换到结果窗口，应用进入未响应状态。预期行为是关闭截图遮罩、显示结果窗口并开始分析。

## Reproduction

1. 在 Windows 启动应用并完成模型与提示词配置。
2. 开始截图并拖动选择一个非空区域。
3. 松开鼠标完成截图。
4. 应用未显示结果窗口并进入未响应状态。

## Suspected Code Paths

- `src/views/CaptureOverlay.tsx:139` — Windows 截图遮罩在鼠标释放后调用 `finish_capture`。
- `src-tauri/src/commands.rs:248` — `finish_capture` 当前是同步 Tauri 命令。
- `src-tauri/src/commands.rs:270` — 同步命令继续进入 `start_analysis_with_image`。
- `src-tauri/src/commands.rs:299` — 分析启动流程创建结果窗口。
- `src-tauri/src/commands.rs:780` — `WebviewWindowBuilder::new(...).build()` 在 Windows 同步命令中触发 Tauri 已知死锁条件。

## Root Cause Hypothesis

**Confidence: high.** Windows 截图流程从同步 `finish_capture` IPC 命令内创建新的 WebView 结果窗口。Tauri 的 `WebviewWindowBuilder` 文档明确指出，在 Windows 的同步命令和事件处理器中调用该构建器会死锁，并要求使用异步命令或独立线程。macOS 不受影响，因为原生选区截图从异步 `begin_capture_action` 路径调用同一个结果窗口创建逻辑。

## Proposed Remediation

**Preferred**: 将 `finish_capture` 改为异步 Tauri 命令，使结果窗口创建不再运行在 Windows 同步 IPC 处理上下文中。保留现有截图合成、状态管理和窗口创建顺序，不新增平台分支或抽象。

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 添加编译级回归检查，约束 `finish_capture` 保持异步返回，防止以后改回同步命令。
- 运行 Rust 测试、前端构建和 Tauri Windows 构建；在本机 Windows 手动执行一次完整截图流程。

## Risks & Considerations

- 变更只改变命令调度方式，不改变 IPC 名称、参数、返回值或分析状态语义。
- 自动化测试无法单独证明 WebView2 原生窗口不再死锁，仍需 Windows 桌面端手动验证。
- `compose_selection` 会在异步运行时执行；若未来超大截图表现出明显 CPU 延迟，再考虑单独移入 `spawn_blocking`，本次不预先扩展范围。

## Open Questions

- 无。
