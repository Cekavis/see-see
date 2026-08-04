# Bug Fix: Lossless Scaling 下截图遮罩未获取前台焦点

- **Slug**: windows-lossless-scaling-overlay-focus
- **Fixed**: 2026-08-04
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Windows 截图遮罩完成显示和 topmost 重排后，现在调用项目已有的 `window.set_focus()`，使遮罩请求前台激活并复用 TAO 的 Windows 回退逻辑。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | capture `show → raise` 后获取前台焦点 |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | 固定 `show → raise → focus` 顺序 |

## Diff Highlights

```rust
raise_capture_window(window)?;
window.set_focus()
```

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::windows_capture_overlay_is_raised_and_focused_after_show` — 约束 Windows capture 显示、置顶和聚焦顺序。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_is_raised_and_focused_after_show` → passed，1 passed。
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed，完整 Rust 测试通过。
- `npm run build`、`npm test`、`npm run lint`、`npm run format:check` → passed；前端 42 tests。
- `npm run tauri build` → passed，重新生成 0.5.1 MSI 与 NSIS。
- NSIS `/S` 覆盖安装 → passed；本地安装项仍为 0.5.1，安装文件时间已更新。

## Deviations from Assessment

无。

## Follow-ups

- 完全退出旧进程后启动已安装的 0.5.1，在 Lossless Scaling 全屏场景再次触发截图。
