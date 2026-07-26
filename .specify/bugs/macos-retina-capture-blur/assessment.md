# Bug Assessment: macOS Retina 截图遮罩模糊

- **Slug**: macos-retina-capture-blur
- **Created**: 2026-07-26
- **Source**: pasted text and installed 0.3.5 reproduction
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

用户在两块 2× 显示器上触发截图后，遮罩里的冻结画面明显模糊，希望达到 macOS 原生截图的清晰度和观感。

## Symptom

遮罩窗口按显示器 logical points 正确覆盖屏幕，但冻结画面也只按 logical dimensions 捕获，导致 1920×1080 PNG 被拉伸到 3840×2160 的 Retina backing pixels。预期冻结画面以显示器原生像素密度捕获和呈现，同时框选坐标保持 logical points。

## Reproduction

1. 在 2× Retina 主屏（1920×1080 points / 3840×2160 pixels）启动 See See 0.3.5。
2. 触发 `Command+Shift+2`。
3. 观察遮罩背景的文字和窗口边缘比原始工作区模糊。
4. 检查 `SCStreamConfiguration`：输出被设置为 `metadata.bounds.width/height`，即 1920×1080 pixels。

## Suspected Code Paths

- `src-tauri/src/capture/macos.rs:request_display_image()` — 把 logical point dimensions 直接传给单位为 pixels 的截图配置。
- `src-tauri/src/capture.rs:FrozenMonitor::new()` — 强制冻结图片尺寸等于 logical bounds，无法保存 Retina 像素帧。
- `src-tauri/src/capture.rs:compose_selection()` — 默认选区坐标与图片像素 1:1，无法从 logical selection 裁切 2× frame。

## Root Cause Hypothesis

**高置信度**。本机显示器分别为 `1920×1080 @2×` 与 `1710×1107 @2×`，但当前 ScreenCaptureKit 配置只请求 1920×1080 和 1710×1107 pixels。SDK 头文件明确标注配置 width/height 的单位为 pixels，并提供 `SCDisplay.pointPixelScale`。因此模糊是确定的 2× 下采样后再放大，不是 CSS 滤镜或透明度问题。

## Proposed Remediation

**Preferred**: 使用最低系统 macOS 26 提供的 `SCScreenshotConfiguration` 与 `captureScreenshotWithFilter:configuration:completionHandler:`，以 `SCDisplay.width/height`（points）乘 `pointPixelScale` 设置原生 pixel dimensions，并使用 local display intent 的 SDR 图像。冻结帧同时保存独立的 pixel scale；窗口与前端框选继续使用 logical coordinates。

更新选区合成：把 logical intersection 映射到每块冻结帧的像素坐标；单屏选区输出该屏原生像素，跨不同缩放显示器时统一到相交显示器的最大像素密度并按需重采样。

**Files likely to change**:

- `src-tauri/src/capture/macos.rs`
- `src-tauri/src/capture.rs`
- `src-tauri/tests/capture_flow.rs`
- `.specify/bugs/macos-retina-capture-blur/fix.md`

**Tests to add or update**:

- 2× logical bounds 接受两倍宽高的 BGRA frame，并拒绝错误像素尺寸。
- 2× 单屏 logical selection 输出原生 2× pixel dimensions 和正确像素。
- 1×/2× 跨屏选区在统一输出密度下保持内容与布局。
- 安装版全屏 frame PNG 尺寸分别为 3840×2160 与 3420×2214；视觉对比无模糊放大。

## Risks & Considerations

- 原生像素 PNG 占用更多内存；两块当前显示器约需 63 MiB RGBA，加上编码缓冲。截图会话结束后仍按现有生命周期释放。
- `SCScreenshotOutput.sdrImage` 的 CGImage 布局必须继续经过现有 BGRA 行步长校验。
- 跨不同缩放显示器的输出必须定义统一密度，避免拼接尺寸或坐标错位。
- 版本保持本次未完成目标的 0.3.5，不重复递增。

## Open Questions

- 无。代码、SDK 单位说明和本机显示器数据足以确定修复。
