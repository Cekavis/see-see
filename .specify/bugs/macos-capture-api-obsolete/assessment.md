# Bug Assessment: macOS 27 截图后端调用已废止 API

- **Slug**: macos-capture-api-obsolete
- **Created**: 2026-07-26
- **Source**: pasted text and local application log
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 权限已就绪，但是截图无法触发，包括我手动从菜单栏里点截图也没有反应，这可能是另一个问题，请你调查

签名后的 See See 0.3.2 在 macOS 27.0 显示屏幕权限已就绪，但无论使用全局快捷键还是菜单栏“开始截图”，都不出现截图选区或其他可见反馈。

## Symptom

快捷键和菜单栏事件均能进入同一个截图命令，权限预检也已通过，但在读取显示器画面时返回 `capture_failed`，所以尚未创建选区窗口。两个入口只记录错误码，没有把底层 xcap 错误写入日志或向菜单栏用户展示，表面上看起来像“没有反应”。

## Reproduction

1. 在 macOS 27.0（Build 26A5388g）安装并启动签名的 See See 0.3.2。
2. 确认应用显示“屏幕权限已就绪”。
3. 按已配置的全局截图快捷键，未出现截图选区。
4. 从菜单栏选择“开始截图”，同样未出现截图选区。
5. 查看 `~/Library/Logs/app.seesee.desktop/See See.log`，两个入口分别记录 `capture shortcut failed: capture_failed` 和 `tray capture failed: capture_failed`。

## Suspected Code Paths

- `src-tauri/src/commands.rs:62` — `begin_capture_action()` 在权限预检通过后调用 `CaptureSession::capture_all()`；失败发生在创建 capture windows 之前。
- `src-tauri/src/capture.rs:203` — 对每台显示器调用 xcap 0.9.7 的 `Monitor::capture_image()`，任一步失败都折叠为通用 `capture_failed`。
- `$CARGO_HOME/registry/src/.../xcap-0.9.7/src/macos/capture.rs:15` — xcap 的 macOS 实现直接调用 `CGWindowListCreateImage()`。
- macOS 27 SDK `CoreGraphics.framework/Headers/CGWindow.h:271` — `CGWindowListCreateImage()` 从 macOS 14 deprecated、macOS 15 obsoleted，并明确要求改用 ScreenCaptureKit。
- `src-tauri/src/capture.rs:322` — `capture_error()` 丢弃底层错误文本，无法从日志区分 obsolete API、空 CGImage、权限或显示器错误。
- `src-tauri/src/lib.rs:91` / `src-tauri/src/commands.rs:754` — 菜单栏与快捷键只记录通用错误码；两个入口均没有用户可见的失败通知。

## Root Cause Hypothesis

**高置信度**。运行日志证明两个入口都到达后端且权限 guard 没有返回 `screen_permission_denied`，随后共同失败为 `capture_failed`。锁定的 xcap 0.9.7 在 macOS 上仍使用 `CGWindowListCreateImage()`；当前 macOS 27 SDK 将该函数标记为 `SCREEN_CAPTURE_OBSOLETE(10.5,14.0,15.0)`，即从 macOS 15 已废止，并要求使用 ScreenCaptureKit。该调用在当前系统无法提供有效图像时，xcap 最终会在复制 CGImage data provider 时返回 `Failed to copy data`，而应用把原始错误丢弃。hardened runtime 或新自签名身份不是主要根因：CoreGraphics preflight 已对同一运行进程返回授权，实际失败点是 obsolete 捕获 API。

## Proposed Remediation

**Preferred**: macOS 14+ 改用 ScreenCaptureKit。优先使用 `SCScreenshotManager` 获取各显示器或虚拟桌面的静态 CGImage；macOS 15.2+ 可评估 `captureImageInRect:completionHandler:`，需要兼容最低 macOS 14 时则使用 `SCShareableContent`、`SCContentFilter` 与 `SCStreamConfiguration`。把异步 ScreenCaptureKit 结果转换为现有 `FrozenMonitor`/RGBA 数据，保留 Windows 的 xcap 路径以及现有多显示器物理坐标和选区合成模型。

同时让 `capture_error()` 在本地日志中记录经过脱敏的底层阶段与错误文本，并让快捷键/菜单栏失败时聚焦主窗口或发送通知，避免静默失败。不要仅绕过权限 guard 或关闭 hardened runtime；这两种做法都不会恢复已废止的截图 API。

**Alternatives**:

- 升级到已明确在 macOS 15+ 使用 ScreenCaptureKit 的 xcap 版本（如上游已有），改动较小，但必须检查多屏坐标、缩放、颜色通道和最低系统版本，不能只依据版本号假设实现已迁移。
- macOS 临时调用系统 `screencapture` 工具可恢复部分功能，但进程身份、交互、性能、错误处理和沙箱边界更复杂，不适合作为长期后端。

**Files likely to change**:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/capture.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/tests/capture_flow.rs`
- `README.md` 或 `specs/001-screenshot-ai-translation/quickstart.md`
- 同步版本文件（修复沿用当前会话目标的 0.3.2，不重复递增）

**Tests to add or update**:

- macOS capture adapter 测试：ScreenCaptureKit 返回图像时正确生成 RGBA、显示器 bounds、scale factor 和名称。
- macOS capture adapter 错误测试：API 错误保留可诊断阶段，同时外部 AppError 仍为脱敏的 `capture_failed` 或权限错误。
- 多显示器回归：负坐标、不同 scale factor 和跨屏合成维持现有语义。
- 入口回归：快捷键与菜单栏都调用同一截图动作，失败时记录具体后端阶段并提供可见恢复路径。
- 真实 macOS 27 验证：权限已就绪后，快捷键和菜单栏均显示选区，完成截图并生成分析输入。

## Risks & Considerations

- ScreenCaptureKit 是异步 API，现有 `spawn_blocking` 同步捕获边界需要调整，避免阻塞 Tauri 主线程或引入运行时死锁。
- ScreenCaptureKit 使用 points、pixels、display-local 与 global rect 等多种坐标空间；必须验证 Retina、负坐标和多缩放显示器。
- CGImage 可能包含 BGRA、行填充或扩展动态范围；转换时必须锁定 SDR RGBA 语义和现有 PNG 限制。
- 最低支持 macOS 14，不能仅使用 macOS 15.2 或 26 新增的便捷 API 而不提供可用性分支。
- 底层错误只能写入本地脱敏日志，不能把窗口标题、用户内容或图像数据写入日志。
- Windows 应继续使用现有 xcap 后端并执行平台回归。

## Open Questions

- [NEEDS CLARIFICATION: 是否必须继续支持 macOS 14.0–15.1；如果最低版本可以提高到 15.2，`captureImageInRect` 能显著简化实现。]
- [NEEDS CLARIFICATION: 是否优先升级 xcap，还是在项目内维护一个只覆盖静态截图的 ScreenCaptureKit adapter；需先确认可用上游版本的实现。]
