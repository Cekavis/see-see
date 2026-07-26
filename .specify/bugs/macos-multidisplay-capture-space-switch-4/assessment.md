# Bug Assessment: macOS 双屏遮罩需要逐窗口 Space 回退

- **Slug**: macos-multidisplay-capture-space-switch-4
- **Created**: 2026-07-26
- **Source**: post-fix mixed-Space reproduction
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

双显示器同时显示另一应用全屏 Space 与普通桌面 Space 时，全局 `CanJoinAllSpaces` 只让普通桌面遮罩可见；全局追加 `CanJoinAllApplications` 后只让全屏遮罩可见。两个窗口 frame 始终正确。

## Symptom

单一 collection behavior 只能覆盖当前两类 Space 之一，导致另一块显示器不可框选。预期两个遮罩分别适配所在显示器当前可见 Space。

## Reproduction

1. 主屏显示 Codex 系统全屏 Space，第二屏显示普通桌面 Space。
2. 分别安装仅 `CanJoinAllSpaces` 和额外 `CanJoinAllApplications` 的构建。
3. 触发快捷键并查询所有/当前 on-screen 窗口。
4. 观察两种策略各自只显示一块遮罩。

## Suspected Code Paths

- `src-tauri/src/windowing.rs:policy_for()` — 把每屏运行时差异压成一个固定 capture policy。
- `src-tauri/src/windowing.rs:show_capture_window()` — 已在 AppKit 主线程显示窗口，适合用公开的 `NSWindow.isOnActiveSpace()` 决定回退。

## Root Cause Hypothesis

**高置信度**。普通 Space 与其他应用的全屏集合需要不同的归属行为；固定策略会把所有显示器窗口加入同一类集合。AppKit 提供 `isOnActiveSpace` 判断窗口当前是否与活动 Space 关联，可在普通策略显示后只为不可见窗口追加 `CanJoinAllApplications`。

## Proposed Remediation

**Preferred**: capture 基础策略恢复为 `CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle`。在同一个 AppKit 主线程回调中设置 frame、应用基础策略并显示；若该原生窗口 `isOnActiveSpace()` 为 false，再追加 `CanJoinAllApplications` 并 `orderFrontRegardless()`。结果窗口策略不变。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `.specify/bugs/macos-multidisplay-capture-space-switch-4/fix.md`

**Tests to add or update**:

- 基础 capture policy 不包含跨应用行为，full-screen fallback policy 包含。
- 双屏混合 Space 实机触发后两个窗口均 on-screen 且 frame 正确。
- 两屏视觉截图都显示清晰遮罩；Esc 同时关闭。

## Risks & Considerations

- `isOnActiveSpace` 必须在基础策略和首次显示之后读取。
- 回退只影响检测为不在活动 Space 的短生命周期截图窗口。
- 不使用私有 SkyLight/Spaces API。

## Open Questions

- 无。
