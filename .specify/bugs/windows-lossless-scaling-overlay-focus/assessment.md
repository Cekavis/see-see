# Bug Assessment: Lossless Scaling 下截图遮罩未获取前台焦点

- **Slug**: windows-lossless-scaling-overlay-focus
- **Created**: 2026-08-04
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 0.5.1 在截图遮罩显示后调用 `SetWindowPos(HWND_TOPMOST)`，但用户在实际 Lossless Scaling 场景确认“仍然不行”。原始现象仍是必须 Alt+Tab 后才能看到框选界面。

## Symptom

截图遮罩已经是 topmost 并在显示时重新排过 z-order，但 Lossless Scaling 保持前台时仍覆盖遮罩。切换前台窗口后遮罩才可见，说明缺失的是前台激活而不只是 topmost 状态。

## Reproduction

1. 安装包含 `SetWindowPos(HWND_TOPMOST)` 修复的 See See 0.5.1。
2. 用 Lossless Scaling 将窗口化游戏缩放到全屏。
3. 按截图快捷键，框选界面仍不可见。
4. Alt+Tab 到其他窗口后，框选界面出现。

## Suspected Code Paths

- `src-tauri/src/windowing.rs:show_capture_window()` — Windows 路径执行 `show()` 和 `raise_capture_window()`，但没有调用窗口聚焦 API。
- `tao/platform_impl/windows/window.rs:set_focus()` — 当前依赖已有 Windows 前台激活实现：先调用 `SetForegroundWindow`，失败时使用其既有 Alt 输入回退后重试。
- `src-tauri/tests/desktop_lifecycle.rs` — 当前测试只约束 topmost 重排发生在 `show()` 后，未约束 capture window 获取焦点。

## Root Cause Hypothesis

**Confidence: high.** 上一修复只保证遮罩处于 topmost 组顶部，没有使 See See 成为前台应用。用户必须 Alt+Tab 才能看到遮罩，且 TAO 将 Windows `set_focus()` 单独实现为前台激活流程，说明 Lossless Scaling 的缩放窗口在保持前台时会继续压住未激活的遮罩。当前 capture 路径恰好遗漏了项目其他窗口显示路径已有的 `set_focus()`。

## Proposed Remediation

**Preferred**: 在 Windows `show_capture_window()` 中保留现有 `show()` 和 `raise_capture_window()`，随后调用 Tauri `window.set_focus()`。复用已安装 TAO 的 Windows 前台激活与回退实现，不自建 `AttachThreadInput`、输入模拟或重试循环。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`

**Tests to add or update**:

- 更新 Windows lifecycle 回归检查，约束 capture 路径依次执行 `show()`、`raise_capture_window()`、`set_focus()`。
- 运行 Rust 全套、前端检查、Tauri 0.5.1 发布构建和覆盖安装。

## Risks & Considerations

- 截图框选本来就需要键盘 Esc 和鼠标输入，获取焦点符合窗口职责。
- `set_focus()` 仅在截图遮罩实际显示时调用，不改变主窗口和结果窗口策略。
- 本次是已发布 0.5.1 的 corrective fix；按仓库规则复用 0.5.1，不再次递增版本。
- 若 Lossless Scaling 以更高完整性级别运行，Windows 仍可能阻止较低权限进程获得前台；需要用户实机复测。

## Open Questions

- 无。
