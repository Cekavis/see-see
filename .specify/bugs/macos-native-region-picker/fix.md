# Bug Fix: macOS 使用系统原生区域截图选择器

- **Slug**: macos-native-region-picker
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

macOS 截图入口现在直接运行系统 `/usr/sbin/screencapture -i -r -t png`，只显示与 `Command+Shift+4` 相同的原生十字选择器，不再创建 See See 遮罩窗口。选区完成后沿用现有分析流程显示紧凑结果窗，按 Esc 则静默取消并释放捕获状态。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/capture.rs` | modified | 增加系统截图命令、唯一 PNG 输出读取、规范化和清理逻辑 |
| `src-tauri/src/commands.rs` | modified | macOS 改走 blocking 原生选择器；Windows 保留原有多屏遮罩路径 |
| `src-tauri/src/state.rs` | modified | 增加原生捕获 reservation，阻止重复快捷键并按所有结果释放 |
| `src/ipc.ts` | modified | `beginCapture` 返回类型调整为 `void`，与平台无关命令合同一致 |
| `specs/001-screenshot-ai-translation/quickstart.md` | modified | 增加双 2× 显示器、当前 Space、无自制遮罩验收项 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | 同步修复版本为 `0.3.5` |

## Diff Highlights

- macOS 精确执行 `/usr/sbin/screencapture -i -r -t png <unique-cache-path>`，由系统负责十字光标、Retina、多显示器和全屏 Space。
- 没有输出文件时视为用户取消，不显示错误；成功输出经过 PNG 规范化并在读取后删除。
- 原生截图子进程在 blocking runtime 等待，Tauri UI 线程不被阻塞。
- macOS 不再调用 `CaptureSession::capture_all` 或 `create_capture_windows`；Windows 行为不变。

## Tests Added or Updated

- `src-tauri/src/capture.rs::tests::native_region_picker_matches_macos_interactive_capture_contract` — 固定系统工具和参数，验证无文件取消、PNG 读取与临时文件清理。
- `src-tauri/src/state.rs::tests::capture_reservation_blocks_duplicates_and_only_owner_releases_it` — 固定重复捕获拦截及 reservation 所有权释放。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --lib` → 11 passed。
- Commands run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed。
- Commands run: `npm run build` → passed。
- Commands run: signed `npm run tauri build` → passed；安装到 `/Applications/See See.app`，安装版与 bundle 可执行文件 SHA-256 均为 `72cd7068593937cca6eef10f2493198de3d91fb0e69fa248e9f1befebdf03f01`。
- Manual checks: 两块显示器中触发 `Command+Shift+2` 后，子进程为 `/usr/sbin/screencapture -i -r -t png <cache-path>`，没有名为 `See See Capture` 的窗口；Esc 静默退出并可立即再次触发。
- Manual checks: 在 `1920×1080` 全屏 Space 内原生框选后，系统截图进程退出，并在同一显示器/工作区出现 `460×500` 的 `See See · 识别结果` 窗口；模型分析完成。

## Deviations from Assessment

无。最终实现采用评估中的首选修复；此前尝试改善自制遮罩的中间方案均保留在各自 bug 报告中，没有进入最终源码。

## Follow-ups

- Windows 仍使用自制多屏遮罩，后续 Windows 手工回归应继续覆盖混合 DPI 和跨屏选区。
- 当前机器没有可用的受信任 codesigning identity；安装版包含 hardened-runtime CMS 签名并可运行，但 `codesign --verify --deep --strict` 的证书信任检查返回 `CSSMERR_TP_NOT_TRUSTED`，发布前仍需使用正式 Developer ID 签名和公证。
