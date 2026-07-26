# Bug Fix: macOS 双显示器截图遮罩原位显示

- **Slug**: macos-multidisplay-capture-space-switch
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

macOS 截图遮罩现在按 CoreGraphics 逻辑桌面坐标创建，并在前端使用 1:1 逻辑选择比例，避免 2× 显示器窗口被缩半和错放。每块显示器遮罩同时改为移动到对应活动 Space，不再复制到所有 Space；版本同步提升为 0.3.5。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | 增加平台窗口几何合同；macOS 使用逻辑坐标和 1.0 选择比例，Windows 保留物理路径 |
| `src-tauri/src/windowing.rs` | modified | 遮罩由 `CanJoinAllSpaces` 改为 `MoveToActiveSpace`，保留全屏辅助、固定位置和忽略窗口循环 |
| `src-tauri/src/commands.rs` | modified | URL 和窗口放置统一使用平台几何合同 |
| `src-tauri/tests/desktop_lifecycle.rs` | added tests | 固定当前两块 2× 显示器的逻辑原点、尺寸、选择比例和 Space 策略 |
| `src/views/CaptureOverlay.test.tsx` | added test | 固定 macOS 逻辑坐标 1:1 选区换算 |
| `specs/001-screenshot-ai-translation/quickstart.md` | modified | 增加 macOS 两块 2× 显示器上下排列验收项 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | 同步版本至 0.3.5 |

## Diff Highlights

- 本机主屏 `0,0 1920×1080 @2×` 创建完整 `1920×1080` logical overlay，不再成为约 `960×540` 的窗口。
- 第二屏 `99,1080 1710×1107 @2×` 保留原点和尺寸，不再被换算到约 `(49.5,540)` 后叠在主屏。
- macOS 选区从 CSS client 坐标直接映射到冻结帧的逻辑桌面坐标；Windows 继续按显示器 scale factor 映射到物理坐标。

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::macos_capture_windows_preserve_two_display_logical_geometry` — 固定两块实际显示器的逻辑几何与 1.0 选择比例。
- `src-tauri/tests/desktop_lifecycle.rs::macos_capture_and_result_windows_use_distinct_space_policies` — 固定遮罩移动到活动 Space 且不加入所有 Space。
- `src/views/CaptureOverlay.test.tsx::keeps macOS logical display coordinates at scale one` — 固定第二屏原点下的 1:1 选区换算。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` → 7 passed。
- Commands run: `npm test -- src/views/CaptureOverlay.test.tsx` → 2 passed。
- Commands run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed。
- Manual checks: 待安装签名的 0.3.5 后在当前双显示器布局复验。

## Deviations from Assessment

无。

## Follow-ups

- 在当前上下排列的两块 2× 显示器分别触发快捷键，确认每块屏幕只显示自己的原位遮罩。
- 在任一显示器的系统全屏 Space 触发并完成框选，确认结果窗留在同一 Space。
