# Bug Assessment: macOS 应使用原生区域截图选择器

- **Slug**: macos-native-region-picker
- **Created**: 2026-07-26
- **Source**: pasted clarification and Pot Desktop reference
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

用户要的不是高清自制遮罩，而是与 macOS `Command+Shift+4` 相同的交互：鼠标直接变成十字，在当前全屏/桌面 Space 原地框选，不显示暗色遮罩或“拖动选择区域”文字。Pot 已实现该体验，可作为参考。

Pot Desktop 的 `src-tauri/src/window.rs::ocr_translate()` 在 macOS 直接执行：

```text
/usr/sbin/screencapture -i -r <output.png>
```

## Symptom

See See 当前为每块显示器创建全屏 WebView 遮罩，既与系统原生截图交互不一致，也持续暴露双显示器 Space 归属、坐标和清晰度问题。预期 macOS 把区域选择交给系统截图工具，See See 只读取完成后的 PNG 并显示紧凑分析结果窗。

## Reproduction

1. 在两块显示器中让主屏显示其他应用的系统全屏 Space，第二屏显示普通桌面。
2. 触发 See See 截图快捷键。
3. 当前实现创建 `See See Capture` 窗口并显示暗色遮罩和文字提示。
4. 对比 `Command+Shift+4` 或 Pot：系统只显示十字区域选择器，不创建应用遮罩窗口。

## Suspected Code Paths

- `src-tauri/src/commands.rs:begin_capture_action()` — macOS 先冻结所有显示器，再创建 WebView 遮罩。
- `src-tauri/src/commands.rs:create_capture_windows()` — 生成暗色全屏截图窗口。
- `src/views/CaptureOverlay.tsx` — 渲染冻结画面、暗色遮罩和文字提示。
- `src-tauri/src/capture/macos.rs` — 自行通过 ScreenCaptureKit 捕获全屏帧，原生区域选择器方案不再需要该路径。

## Root Cause Hypothesis

**高置信度**。问题不是某一个 frame 或 DPI 参数，而是 macOS 交互架构选错：应用试图重建系统区域截图 UI。系统自带 `/usr/sbin/screencapture -i -r` 正好提供用户要求的公共交互，并由 macOS 自身处理全屏 Space、显示器、Retina 和取消。

## Proposed Remediation

**Preferred**: macOS 的 `begin_capture_action` 在检查配置、权限和并发状态后，通过阻塞任务执行 `/usr/sbin/screencapture -i -r -t png <unique-cache-path>`。若输出文件存在，读取并规范化 PNG，清理临时文件，然后直接调用现有 `start_analysis_with_image`；若 Esc 取消且没有文件，清理捕获状态并静默返回成功。

macOS 不调用 `CaptureSession::capture_all`、不创建 capture WebView windows。Windows 保留现有冻结帧和遮罩实现。撤回仅为 macOS 自制遮罩引入的 logical frame、跨应用 Space 和 Retina 拼接逻辑。

**Files likely to change**:

- `src-tauri/src/capture.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/state.rs`
- `src-tauri/src/windowing.rs`
- `src/ipc.ts`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src-tauri/tests/capture_flow.rs`
- `.specify/bugs/macos-native-region-picker/fix.md`

**Tests to add or update**:

- macOS 原生截图命令固定为 `/usr/sbin/screencapture -i -r -t png <path>`。
- 输出文件存在时读取并规范化 PNG；文件不存在视为取消。
- native capture pending 状态阻止重复快捷键，并在成功、取消、失败后释放。
- 安装版触发后没有任何 `See See Capture` 窗口，出现系统十字选择器；Esc 静默取消。
- 双显示器任一当前 Space 完成选区后，紧凑结果窗在同一工作区显示。

## Risks & Considerations

- 子进程等待期间必须放到 blocking runtime，不能阻塞 Tauri UI 线程。
- 输出路径必须唯一且仅清理该次捕获文件。
- 取消不能触发错误对话框；真正无法启动系统工具或读取有效输出仍应返回稳定的 `capture_failed`。
- macOS 26+ 直接使用系统方案，不添加旧系统兼容分支。
- 版本保持本次未完成目标的 0.3.5，不重复递增。

## Open Questions

- 无。用户澄清与 Pot 源码给出了明确目标和参考实现。
