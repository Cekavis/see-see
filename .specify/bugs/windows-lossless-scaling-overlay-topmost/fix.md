# Bug Fix: Lossless Scaling 下截图遮罩未置顶

- **Slug**: windows-lossless-scaling-overlay-topmost
- **Fixed**: 2026-08-04
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Windows 截图遮罩显示后直接对 HWND 调用 `SetWindowPos(HWND_TOPMOST)`，绕过 Tauri 对既有 topmost 状态的缓存并重新提升 z-order。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | modified | capture `show()` 后重新提升 HWND 到 topmost 顶端 |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 约束原生置顶调用与显示顺序 |
| `src-tauri/Cargo.toml` | modified | 启用已有 `windows` crate 的 WindowsAndMessaging API |
| `package.json` | modified | 版本升级至 0.5.1 |
| `src-tauri/Cargo.toml` | modified | 版本升级至 0.5.1 |
| `src-tauri/tauri.conf.json` | modified | 版本升级至 0.5.1 |

## Diff Highlights

```rust
window.show()?;
raise_capture_window(window)
```

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::windows_capture_overlay_is_raised_after_show` — 约束 capture 显示路径调用 `SetWindowPos`、使用 `HWND_TOPMOST`，且提升发生在 `show()` 后。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_is_raised_after_show` → passed，1 passed。
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed，完整 Rust 测试通过。
- `npm run build`、`npm test`、`npm run lint`、`npm run format:check` → passed；前端 42 tests。
- `npm run tauri build` → passed，生成 0.5.1 MSI 与 NSIS。
- NSIS `/S` 本地安装 → passed；卸载注册表 DisplayVersion 为 0.5.1。

## Deviations from Assessment

无。

## Follow-ups

- 在实际启用 Lossless Scaling 的全屏游戏上触发截图，确认遮罩立即显示在缩放层上方。
