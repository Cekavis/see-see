# Bug Fix: Windows 截图遮罩黑屏闪烁

- **Slug**: windows-capture-overlay-black-flash
- **Fixed**: 2026-07-30
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

capture window 不再由创建路径立即显示。前端先取得并解码截图 PNG、提交背景，再通过受会话和显示器校验的 IPC 命令显示对应窗口，使首个可见帧就是截图。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | 新增已校验的 `show_capture_overlay`，移除创建后的立即显示 |
| `src-tauri/src/lib.rs` | modified | 注册 ready/show IPC 命令 |
| `src/ipc.ts` | modified | 添加 `showCaptureOverlay` wrapper |
| `src/views/CaptureOverlay.tsx` | modified | PNG 解码并提交背景后才请求显示窗口 |
| `src/views/CaptureOverlay.test.tsx` | added test | 固定解码前不显示、背景提交后才显示 |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 固定创建路径隐藏，ready IPC 才显示 |

## Diff Highlights

```ts
await image.decode();
setFrameUrl(url);
```

```rust
pub fn show_capture_overlay(...) -> Result<(), AppError> {
    // Validate the active session and monitor before showing its window.
}
```

## Tests Added or Updated

- `src/views/CaptureOverlay.test.tsx` — 验证图片 decode 完成前 `showCaptureOverlay` 未调用，背景提交后才调用。
- `src-tauri/tests/desktop_lifecycle.rs::capture_overlay_waits_for_frontend_frame_readiness` — 验证创建路径不立即显示，ready IPC 负责显示。

## Local Verification

- 聚焦前端测试 → 2 passed。
- 聚焦 Rust ready test → 1 passed。
- `npm run build`、`npm test`、`npm run lint`、`npm run format:check` → passed；前端 42 tests。
- `cargo test --manifest-path src-tauri/Cargo.toml`、`cargo fmt -- --check` → passed；11 项 desktop lifecycle tests。
- `npm run tauri build` → passed，生成最终 0.4.3 MSI 与 NSIS。
- NSIS 静默安装 → passed；Display/File/Product version 均为 0.4.3。

## Deviations from Assessment

- 复用当前 session 的 0.4.3，不进行第二次版本递增。

## Follow-ups

- 在最终安装版触发一次截图，确认首次可见帧不再是黑色占位背景。
