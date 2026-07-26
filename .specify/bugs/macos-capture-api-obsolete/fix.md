# Bug Fix: macOS 截图后端迁移到 ScreenCaptureKit

- **Slug**: macos-capture-api-obsolete
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

macOS 14+ 的静态截图已从 xcap 0.9.7 的废止 `CGWindowListCreateImage()` 路径迁移到 `SCScreenshotManager`，并继续使用既有的显示器元数据、物理坐标、RGBA 帧和跨屏合成模型。Windows 和其他非 macOS 平台仍使用原有 xcap 后端；截图失败现在记录脱敏的后端阶段和错误文本，并让快捷键或菜单栏用户看到原生错误对话框。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/capture/macos.rs` | added | 使用 `SCShareableContent`、`SCContentFilter`、`SCStreamConfiguration` 和 `SCScreenshotManager` 捕获各显示器，并将带行填充的 BGRA 转为现有 RGBA 帧 |
| `src-tauri/src/capture.rs` | modified | 按平台选择截图后端，保留非 macOS xcap 路径，增加可诊断错误阶段和纯转换适配器回归测试 |
| `src-tauri/src/commands.rs` | modified | 后端失败时聚焦主窗口；快捷键失败显示原生错误对话框，同时保留已有任务的焦点语义 |
| `src-tauri/src/lib.rs` | modified | 菜单栏截图失败复用同一可见错误报告路径 |
| `src-tauri/Cargo.toml` | modified | 为 macOS ScreenCaptureKit Objective-C 桥声明已锁定的 objc2/block2 依赖 |
| `src-tauri/Cargo.lock` | modified | 同步根包的直接 macOS 依赖，不引入新的第三方包版本 |
| `src-tauri/tauri.conf.json` | modified | 将发布包最低 macOS 版本显式设为文档承诺的 14.0 |
| `README.md` | modified | 记录 macOS 14+ 使用 ScreenCaptureKit |

## Diff Highlights

- macOS 捕获在 `spawn_blocking` 工作线程内等待 ScreenCaptureKit 的异步完成回调，不阻塞 Tauri 主线程。
- 每块显示器仍使用 xcap 的 ID、名称、全局 bounds、scale factor 和 primary 标记；ScreenCaptureKit 只替换失效的图像读取步骤。
- `capture_error(stage, error)` 仅向本地日志写入硬编码阶段和截断/去控制字符后的错误文本，对 IPC 仍返回稳定的 `capture_failed` 或权限错误。

## Tests Added or Updated

- `capture::tests::screen_capture_adapter_converts_padded_bgra_and_preserves_metadata` — 固定 BGRA→RGBA、行填充、负坐标、scale factor、名称和主显示器语义。
- `capture::tests::screen_capture_adapter_rejects_bad_frames_without_leaking_details` — 固定畸形帧诊断与外部错误脱敏。
- `capture::tests::capture_permission_guard_and_backend_error_use_recovery_code` — 固定后端权限错误映射。
- `src-tauri/tests/capture_flow.rs::cross_monitor_crop_uses_virtual_desktop_physical_pixels` — 既有回归继续覆盖负坐标、不同 scale factor 和跨屏合成。

## Local Verification

- Commands run: `npm run format:check` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm test` → 12 files / 36 tests passed.
- Commands run: `npm run build` → TypeScript and Vite production build passed.
- Commands run: `npm run test:e2e` → primary desktop flow passed after granting localhost bind access.
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- Commands run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → all unit, integration, benchmark, contract, and doc tests passed after granting wiremock localhost bind access.
- Commands run: `npm run tauri build` → signed `See See.app` and `See See_0.3.2_aarch64.dmg` produced.
- Commands run: `npm run verify:macos-signature` and `codesign --verify --deep --strict` → stable `See See Local Release` signature and designated requirement passed.
- Manual checks: installed bundle hash matches the built bundle; `/Applications/See See.app` reports `LSMinimumSystemVersion=14.0` and launches its existing configured main UI.
- Manual checks pending: Computer Use cannot expose macOS menu-bar status items or synthesize a true global shortcut, so real shortcut/menu-bar selection-window verification is deferred to `/speckit-bug-test`.

## Deviations from Assessment

- The platform adapter was placed in the new `src-tauri/src/capture/macos.rs` submodule instead of embedding Objective-C bridge code directly in `capture.rs`; this keeps unsafe macOS-only code isolated without changing the requested scope or behavior.
- `src-tauri/tauri.conf.json` now explicitly sets the already documented macOS 14.0 support floor. The previous default emitted `MACOSX_DEPLOYMENT_TARGET=10.13`, which was incompatible with an unconditional macOS 14 ScreenCaptureKit backend.
- Adapter regression tests live beside the pure conversion boundary in `capture.rs`; the existing integration test continues to cover multi-display composition.
- As directed by the assessment, synchronized application versions remain `0.3.2` and were not incremented again.

## Follow-ups

- Run `/speckit-bug-test slug=macos-capture-api-obsolete` and manually verify that the installed app's global shortcut and menu-bar action both show the selection overlay and complete a screenshot on macOS 27.
- Repeat the existing Retina, negative-coordinate, mixed-scale, and cross-display matrix on real multi-monitor macOS hardware.
