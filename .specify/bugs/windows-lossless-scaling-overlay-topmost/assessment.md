# Bug Assessment: Lossless Scaling 下截图遮罩未置顶

- **Slug**: windows-lossless-scaling-overlay-topmost
- **Created**: 2026-08-04
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 启用 Lossless Scaling 缩放窗口后，截图本身正常，但按截图键后仍显示缩放后的全屏游戏；必须 Alt+Tab 到其他窗口后，框选界面才出现。期望截图框选界面强制显示在最前面。

## Symptom

Windows 截图遮罩在普通窗口上可见，但可能落在 Lossless Scaling 的全屏置顶窗口之后。预期遮罩在截图帧准备完成后立即位于当前 topmost z-order 的最前端。

## Reproduction

1. 在 Windows 中用 Lossless Scaling 将窗口化游戏缩放到全屏。
2. 按 See See 截图快捷键。
3. 截图遮罩未显示在游戏上方；Alt+Tab 后才可见。

## Suspected Code Paths

- `src-tauri/src/commands.rs:create_capture_windows()` — 创建窗口时使用 `.always_on_top(true)`，只为 HWND 设置 topmost 状态。
- `src-tauri/src/windowing.rs:show_capture_window()` — Windows 路径在前端帧就绪后只调用 `show()`，没有在此时重新将 HWND 排到 topmost z-order 顶端。
- `src-tauri/tests/desktop_lifecycle.rs` — 已约束显示时序和 DWM 动画，但未约束实际显示时重新提升 z-order。

## Root Cause Hypothesis

**Confidence: high.** Capture window 在隐藏创建时已经带有 `WS_EX_TOPMOST`，但 topmost 窗口之间仍有 z-order。TAO 的 `set_always_on_top(true)` 仅在状态位发生变化时调用 `SetWindowPos`，因此对已经 topmost 的隐藏窗口重复设置不会重新排序。Lossless Scaling 的全屏窗口位于它前面时，随后单独 `show()` 不能保证遮罩成为最前面的 topmost 窗口。

## Proposed Remediation

**Preferred**: 在 Windows `show_capture_window()` 路径中先保留现有 DWM transition 禁用和 `show()`，再对 capture HWND 直接调用 `SetWindowPos(HWND_TOPMOST, SWP_NOMOVE | SWP_NOSIZE)`，在实际显示时重新提升 z-order。继续限定在 capture window，不改变主窗口或结果窗口策略。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src-tauri/Cargo.toml`
- `package.json`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 添加 Windows 生命周期回归检查，约束 `SetWindowPos` 使用 `HWND_TOPMOST` 且发生在 capture `show()` 后。
- 运行 Rust 测试、前端测试与检查、Tauri Windows 发布构建和本地安装。

## Risks & Considerations

- 只重新排序 capture HWND，窗口在截图结束时仍按现有路径销毁，不遗留置顶状态。
- Windows 对前台焦点有额外限制；本修复目标是保证可见 z-order，不依赖抢占键盘焦点。
- 需要启用现有 `windows` crate 的 `Win32_UI_WindowsAndMessaging` feature，不引入新依赖。

## Open Questions

- 安装版仍需在实际运行 Lossless Scaling 的环境中确认兼容性。
