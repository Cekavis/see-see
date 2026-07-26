# Bug Assessment: macOS 遮罩未进入其他应用全屏 Space

- **Slug**: macos-multidisplay-capture-space-switch-3
- **Created**: 2026-07-26
- **Source**: post-fix dual-display visual reproduction
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

0.3.5 第二次修复后，两个 `See See Capture` 窗口的原生 frame 已正确覆盖两块显示器；但分别截取显示器画面时，普通桌面 Space 上的第二屏显示遮罩，主屏上另一应用的全屏 Codex Space 没有遮罩。用户仍需离开全屏工作区才能框选。

## Symptom

在双显示器环境中从另一应用的全屏 Space 触发快捷键，See See 遮罩没有出现在该全屏 Space。预期两块显示器各自在当前可见 Space 原地显示遮罩，其中包括其他应用拥有的系统全屏 Space。

## Reproduction

1. 主屏进入 Codex 的系统全屏 Space，第二屏保持普通桌面 Space。
2. 启动安装后的 See See 0.3.5，发送 `Command+Shift+2`。
3. 查询原生窗口 frame，确认两个窗口分别为 `1920×1080 @ (0,0)` 与 `1710×1107 @ (99,1080)`。
4. 分别使用 `screencapture -D 1` 和 `screencapture -D 2` 保存当前显示器画面。
5. 观察第二屏遮罩可见，而主屏全屏 Codex Space 没有遮罩。

## Suspected Code Paths

- `src-tauri/src/windowing.rs:policy_for()` — capture policy 有 `CanJoinAllSpaces` 和 `FullScreenAuxiliary`，但没有 macOS 13+ 的 `CanJoinAllApplications`。
- `src-tauri/src/windowing.rs:apply_macos_policy()` — 直接将自定义 bits 转为 `NSWindowCollectionBehavior`，适合补充新行为位。

## Root Cause Hypothesis

**高置信度**。macOS 26 SDK 的 `NSWindow.h` 明确说明 `NSWindowCollectionBehaviorCanJoinAllApplications`（`1 << 18`）允许浮动窗口加入其他应用的集合以及符合条件的全屏 Spaces；现有 `CanJoinAllSpaces` 只处理 Spaces 复制。主屏全屏 Space 的视觉失败与第二屏普通 Space 成功正好符合缺少该行为位的边界。

## Proposed Remediation

**Preferred**: 在 macOS capture overlay policy 中加入 `NSWindowCollectionBehaviorCanJoinAllApplications`，保留已经实机验证正确的 logical geometry、固定 native frame、`CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle` 和弹出菜单层级。结果窗口继续使用 `MoveToActiveSpace | FullScreenAuxiliary`。

项目最低系统版本已是 macOS 26，因此直接使用当前推荐行为，不添加旧系统兼容分支。Windows 逻辑不变。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `.specify/bugs/macos-multidisplay-capture-space-switch-3/fix.md`

**Tests to add or update**:

- capture policy 包含 `CanJoinAllApplications`，result policy 不包含。
- 重新安装 0.3.5 后，双屏原生 frame 仍正确。
- 主屏为其他应用全屏 Space 时，两块显示器截图都显示暗色遮罩和选区提示。
- Esc 后两个遮罩均关闭；完成框选后结果小窗出现在发起截图的同一工作区。

## Risks & Considerations

- `CanJoinAllApplications` 会扩大遮罩可见范围，只应用于短生命周期 capture overlay，不能用于设置窗或结果窗。
- 必须保留 `Stationary` 和固定 native frame，避免重现窗口被搬到同一显示器。
- 自动化只能固定策略位；最终结论仍需真实双显示器、跨应用全屏 Space 的视觉验证。

## Open Questions

- 无。SDK 头文件与实机画面提供了直接证据。
