# Bug Fix: Windows 截图遮罩显示动画

- **Slug**: windows-capture-overlay-animation
- **Fixed**: 2026-07-30
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

在 Windows capture window 显示前按 HWND 禁用 DWM transition，使遮罩从隐藏状态直接出现，同时保留隐藏创建、物理定位和尺寸设置顺序。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | `show()` 前设置 `DWMWA_TRANSITIONS_FORCEDISABLED` |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 固定禁用过渡必须发生在 capture `show()` 前 |
| `src-tauri/Cargo.toml` | modified | Windows 目标直接复用 Tauri 已采用的 `windows` 0.61 绑定 |
| `src-tauri/Cargo.lock` | modified | 同步根包 Windows 依赖 |

## Diff Highlights

```rust
disable_capture_window_transitions(window)?;
window.show()
```

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::windows_capture_overlay_disables_show_transitions` — 约束 `DWMWA_TRANSITIONS_FORCEDISABLED` 在 capture `show()` 前配置。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_disables_show_transitions` → passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed，包含 11 项 desktop lifecycle tests。
- `npm run build`、`npm test`、`npm run lint`、`npm run format:check` → passed；前端 42 tests。
- `npm run tauri build` → passed，生成最终 0.4.3 MSI 与 NSIS。
- NSIS 静默安装 → passed；Display/File/Product version 均为 0.4.3。
- 用户实机确认系统动画已经消失；随后观察到的黑色占位帧作为独立 bug `windows-capture-overlay-black-flash` 处理。

## Deviations from Assessment

- 复用当前 session 已升级的 0.4.3，不进行第二次版本递增。

## Follow-ups

- 无。
