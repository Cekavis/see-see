# Bug Fix: Windows 截图遮罩向右下偏移

- **Slug**: windows-capture-overlay-offset
- **Fixed**: 2026-07-30
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

关闭 Windows capture overlay 的无边框窗口阴影，移除 TAO 为阴影保留的隐藏客户区 inset，使遮罩从显示器物理原点开始覆盖。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | capture window builder 增加 `.shadow(false)` |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 固定 capture window 同时无边框、无阴影 |
| `package.json` | modified | 补丁版本升级至 0.4.3 |
| `package-lock.json` | modified | 同步 npm 根包版本 |
| `src-tauri/Cargo.toml` | modified | 补丁版本升级至 0.4.3 |
| `src-tauri/Cargo.lock` | modified | 同步 Rust 根包版本 |
| `src-tauri/tauri.conf.json` | modified | 补丁版本升级至 0.4.3 |

## Diff Highlights

```rust
.decorations(false)
.shadow(false)
```

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::capture_overlay_disables_undecorated_window_shadow` — 约束 capture builder 不重新启用 Windows 无边框阴影。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle capture_overlay_disables_undecorated_window_shadow` → passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed.
- `npm run build`、`npm test`、`npm run lint`、`npm run format:check` → passed.
- `npm run tauri build` → passed，生成 0.4.3 MSI 与 NSIS。
- NSIS 静默安装 → passed；注册表、文件版本和产品版本均为 0.4.3。
- 用户在安装版复测后确认：“现在正常了”。

## Deviations from Assessment

- 无。

## Follow-ups

- 无。
