# Bug Assessment: macOS 双显示器遮罩被搬到同一显示器

- **Slug**: macos-multidisplay-capture-space-switch-2
- **Created**: 2026-07-26
- **Source**: post-fix native reproduction
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

0.3.5 第一次修复后，自动化合同显示两个遮罩尺寸正确；但安装版本触发快捷键后的原生窗口 frame 为：

```text
1710×1107 at X=99,Y=823
1920×1080 at X=99,Y=1407
```

两个窗口都被放到 X=99 的第二显示器，而不是分别固定在 `0,0 1920×1080` 与 `99,1080 1710×1107`。

## Symptom

把截图遮罩改为 `MoveToActiveSpace` 后，AppKit 会按 See See 当前活动显示器移动多个窗口，导致每显示器遮罩再次集中到同一屏。预期窗口 frame 始终绑定各自显示器，仅加入该位置当前可见的 Space。

## Reproduction

1. 安装第一次 0.3.5 修复并启动 See See。
2. 在本机两块 2× 显示器布局发送 `Command+Shift+2`。
3. 用 `CGWindowListCopyWindowInfo` 读取两个 `See See Capture` frame。
4. 观察两个窗口 X 都为 `99`；原始症状仍存在。

## Suspected Code Paths

- `src-tauri/src/windowing.rs:policy_for()` — `MoveToActiveSpace` 会搬移每个遮罩，而不是只改变其 Space 归属。
- `src-tauri/src/commands.rs:create_capture_windows()` — 通过 Tauri 异步排队位置和尺寸，随后另行排队原生策略与显示；首次 show 可能先于最终 frame 稳定。
- `src-tauri/src/windowing.rs:show_capture_window()` — 主线程回调只设置 collection behavior 和 show，没有在同一原子呈现步骤中固定 native frame。

## Root Cause Hypothesis

**高置信度**。原生 frame 证明第一次评估的逻辑尺寸判断正确，但把 `CanJoinAllSpaces` 替换为 `MoveToActiveSpace` 是错误的多显示器策略。后者会实际移动窗口到应用当前活动 Space/显示器，破坏“一个窗口绑定一块显示器”的 frame。分开的异步定位和显示进一步让 AppKit 在窗口尚未稳定于目标屏幕时决定 Space。

## Proposed Remediation

**Preferred**: 保留 macOS logical geometry 和 selection scale `1.0`，把 capture policy 恢复为 `CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle`。`CanJoinAllSpaces` 只负责让固定 frame 的遮罩在目标显示器当前 Space 可见，不主动搬移窗口；正确的全屏 logical frame 可避免此前因半尺寸和错误原点造成的视觉重叠。

macOS 的 `show_capture_window` 应接收 geometry，并在同一个 AppKit 主线程回调内依次设置 `NSWindow` content size、top-left point、collection behavior、level 和 show。使用主显示器 CoreGraphics logical height 完成 top-left 到 AppKit bottom-left 转换，避免依赖 Tauri 分开的异步 frame 队列。Windows 保留现有 Tauri 物理定位路径。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `.specify/bugs/macos-multidisplay-capture-space-switch-2/fix.md`

**Tests to add or update**:

- Space policy恢复为 join-all-spaces 且不包含 move-to-active-space。
- macOS frame 转换：主屏 top-left `(0,0)` 转为 AppKit `(0,1080)`；下方第二屏 `(99,1080)` 转为 `(99,0)`。
- 安装后 native window frames 分别覆盖两块显示器，且 Esc 关闭全部遮罩。

## Risks & Considerations

- 只有 capture overlay 使用固定 frame + join-all-spaces；结果窗口继续使用 move-to-active-space。
- 原生 frame 设置必须使用逻辑尺寸，与 ScreenCaptureKit 当前逻辑冻结帧保持一致。
- `CanJoinAllSpaces` 在正确全屏 frame 下需要真实双屏复验，确认不会把同一窗口复制到另一显示器坐标。

## Open Questions

- 无。安装后 frame 数据已经直接定位错误策略和呈现时序。
