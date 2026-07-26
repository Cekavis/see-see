# Bug Fix: macOS 遮罩加入其他应用全屏 Space

- **Slug**: macos-multidisplay-capture-space-switch-3
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: partial

## Summary

截图遮罩加入了 `CanJoinAllApplications`，使主屏遮罩能够进入 Codex 的系统全屏 Space；但实机发现同一全局策略会让第二屏普通桌面的遮罩离开当前 Space，因此仅部分修复。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | capture policy 增加 `CanJoinAllApplications` |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | 固定 capture 包含、result 不包含跨应用行为位 |

## Tests Added or Updated

- `macos_capture_and_result_windows_use_distinct_space_policies` — 固定跨应用行为只属于 capture overlay。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` → pass。
- Manual checks: 主屏全屏 Codex 遮罩可见；第二屏窗口存在于正确 frame，但 `kCGWindowIsOnscreen` 缺失。

## Deviations from Assessment

- 全局加入 `CanJoinAllApplications` 不能同时覆盖一块全屏 Space 和一块普通桌面 Space，需要按每个原生窗口是否位于活动 Space 自适应。

## Follow-ups

- 先按普通 `CanJoinAllSpaces` 显示；只对 `NSWindow.isOnActiveSpace()` 为 false 的窗口追加跨应用全屏策略。
