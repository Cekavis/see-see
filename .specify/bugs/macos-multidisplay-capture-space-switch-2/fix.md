# Bug Fix: macOS 双显示器遮罩固定原生 frame

- **Slug**: macos-multidisplay-capture-space-switch-2
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: partial

## Summary

macOS 遮罩恢复为固定 frame 的 `CanJoinAllSpaces` 策略，并在一个 AppKit 主线程回调内完成 logical frame、窗口策略、层级和显示。原生窗口坐标已分别覆盖两块显示器，但实机视觉复验发现遮罩仍未进入另一应用的全屏 Space，因此修复仅部分完成。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | 恢复 `CanJoinAllSpaces`，增加 CoreGraphics top-left 到 AppKit 坐标转换，并原子设置 native frame 和显示策略 |
| `src-tauri/src/commands.rs` | modified | 将每块显示器 logical geometry 和主屏高度交给原生显示路径 |
| `src-tauri/tests/desktop_lifecycle.rs` | updated tests | 固定两块显示器的 logical frame 转换以及遮罩不使用 `MoveToActiveSpace` |

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::macos_capture_windows_preserve_two_display_logical_geometry` — 固定主屏 `(0,0)` 与下方第二屏 `(99,1080)` 的 AppKit top-left 转换。
- `src-tauri/tests/desktop_lifecycle.rs::macos_capture_and_result_windows_use_distinct_space_policies` — 固定遮罩加入所有 Spaces 且不移动到活动 Space。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` → 7 passed。
- Commands run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed。
- Manual checks: 安装签名 0.3.5 后，原生 frame 为 `1920×1080 @ (0,0)` 与 `1710×1107 @ (99,1080)`；第二屏遮罩可见，主屏上另一应用的全屏 Space 未出现遮罩。

## Deviations from Assessment

- 评估假设 `CanJoinAllSpaces | FullScreenAuxiliary` 足以覆盖其他应用的全屏 Space；macOS 26 实机截图推翻了该假设。SDK 头文件说明还需要 `NSWindowCollectionBehaviorCanJoinAllApplications` 才能在符合条件时加入其他应用的集合和全屏 Spaces。

## Follow-ups

- 重新评估并加入 macOS 13+ 的 `CanJoinAllApplications`；本项目最低版本为 macOS 26，无需旧系统兼容分支。
