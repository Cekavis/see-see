# Bug Assessment: macOS 双显示器截图遮罩错位并切回桌面

- **Slug**: macos-multidisplay-capture-space-switch
- **Created**: 2026-07-26
- **Source**: pasted text and attached screenshot
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 这还是跳出来窗口让我选啊，我现在有两个显示器。

用户提供的 3840×2160 截图显示：截图遮罩出现后，当前画面不是快捷键触发时的目标全屏窗口，而是包含 See See、终端、Finder 等窗口的桌面；两台显示器的冻结内容被缩小、错位并叠放在同一可见画布内。

## Symptom

macOS 连接两台 Retina/缩放显示器时，快捷键触发的每显示器遮罩没有覆盖各自显示器，而是以约一半尺寸和错误原点出现在主显示器，造成两个显示器画面重叠，看起来像跳到一个独立选择窗口。预期是每台显示器各自被原位遮罩覆盖，包含系统全屏 Space，并在原位置直接框选。

## Reproduction

1. 连接两台显示器，并启用当前机器的 Retina/2× 缩放布局。
2. 在任一显示器的普通或系统全屏窗口中按 See See 截图快捷键。
3. 观察遮罩窗口：显示器内容被缩小、错位或叠放到主显示器，而不是覆盖原显示器。
4. 本机 CoreGraphics 数据可稳定解释该现象：主显示器 bounds 为 `0,0 1920×1080`、backing pixels 为 `3840×2160`；第二显示器 bounds 为 `99,1080 1710×1107`、backing pixels 为 `3420×2214`，两者 scale 均为 `2.0`。

## Suspected Code Paths

- `src-tauri/src/capture/macos.rs:monitor_metadata()` — 从 `CGDisplayBounds` 间接取得 macOS 逻辑坐标和尺寸，并把它们存入名为 `PhysicalRect` 的结构。
- `src-tauri/src/commands.rs:create_capture_windows()` — 把上述逻辑值传给 Tauri 的 `PhysicalPosition` / `PhysicalSize`；2× 显示器因此被创建为一半逻辑尺寸，第二显示器原点也被再次除以缩放因子。
- `src/views/CaptureOverlay.tsx:rect()` — URL 仍传入显示器 scale `2.0`，在窗口改为正确逻辑尺寸后会再次放大选区坐标；macOS 冻结帧和 bounds 当前都采用逻辑坐标，应使用选择比例 `1.0`。
- `src-tauri/src/windowing.rs:policy_for()` — 每个显示器遮罩都使用 `CanJoinAllSpaces`，会让多个短生命周期窗口加入所有 Space；双显示器“显示器具有单独 Space”布局下可能让其他显示器的遮罩也出现在当前 Space 并相互覆盖。

## Root Cause Hypothesis

**高置信度**。0.3.4 修复只处理了 AppKit collection behavior，没有统一 macOS 显示器坐标系。`CGDisplayBounds` 是逻辑桌面坐标，而 Tauri `PhysicalPosition` / `PhysicalSize` 会按目标窗口 scale factor 再换算一次。本机数据中 `1920×1080 @2×` 因而变成约 `960×540` 的可见窗口；第二显示器的 `(99,1080)` 也被当作物理像素换算到约 `(49.5,540)`，正好落回主显示器并与第一个遮罩重叠。`CanJoinAllSpaces` 进一步使每显示器窗口在双屏独立 Space 中扩散，违背“一块显示器一个原位遮罩”的模型。

## Proposed Remediation

**Preferred**: 为截图窗口引入明确的平台几何合同。macOS 使用 `LogicalPosition` / `LogicalSize` 放置 `CGDisplayBounds`，并让前端选区比例使用 `1.0`，保持冻结帧、显示器 bounds、CSS client 坐标和合成选区处于同一逻辑坐标系。Windows 保留当前物理坐标和真实 scale factor 路径。

同时把截图遮罩的 Space 策略从 `CanJoinAllSpaces` 改为 `MoveToActiveSpace | FullScreenAuxiliary | Stationary | IgnoresCycle`：窗口先放到对应显示器，再移动到该显示器当前活动 Space 后显示，避免多个显示器遮罩复制到所有 Space。结果窗口继续使用 `MoveToActiveSpace | FullScreenAuxiliary`。

**Alternatives**:

- 把 macOS 冻结帧和所有原点转换为 backing pixels：可以保留物理窗口 API，但混合缩放、多显示器相对原点和跨屏选区需要定义新的虚拟像素坐标系，改动及回归风险更大。
- 只修正窗口尺寸、不改 `CanJoinAllSpaces`：单屏会改善，但双显示器独立 Space 仍可能叠出其他显示器的遮罩。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src/views/CaptureOverlay.test.tsx`（如需固定 macOS 选择比例合同）
- `README.md` / `specs/001-screenshot-ai-translation/quickstart.md`
- 同步版本文件至 `0.3.5`

**Tests to add or update**:

- macOS 几何合同：`1920×1080 @2×` 仍创建 `1920×1080` 逻辑遮罩，选择比例为 `1.0`。
- 双显示器合同：`(0,0 1920×1080)` 与 `(99,1080 1710×1107)` 保持原逻辑原点和尺寸，不被 scale 再次除算。
- Space 策略：截图遮罩使用 `MoveToActiveSpace` 且不再使用 `CanJoinAllSpaces`，继续包含 `FullScreenAuxiliary`。
- 原生回归：两块显示器各自只显示自己的冻结帧；普通与系统全屏 Space 均不切换；完成后结果窗留在发起框选的 Space。

## Risks & Considerations

- macOS 与 Windows 的窗口几何必须显式分流，不能把 macOS 逻辑选择比例改动泄漏到 Windows 的混合 DPI 路径。
- `MoveToActiveSpace` 必须在窗口首次显示前应用，并且窗口应先具有目标显示器 frame，避免 AppKit 根据错误屏幕选择 Space。
- 当前 macOS 冻结帧按逻辑尺寸采集；本修复不提升到原生 backing pixel 分辨率，避免扩大本次 bug 范围。
- 跨显示器选区仍使用 macOS 逻辑虚拟桌面坐标；必须复验第二显示器位于主显示器上、下、左、右时的原点。

## Open Questions

- 无。截图、本机双显示器 CoreGraphics 数据和当前实现已经能完整解释现象。
