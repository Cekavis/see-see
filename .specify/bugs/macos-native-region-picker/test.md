# Bug Verification: macOS 系统原生区域截图选择器

- **Slug**: macos-native-region-picker
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

原始症状不再复现：macOS 触发快捷键后由系统 `screencapture` 显示原生区域选择器，See See 不创建遮罩窗口；Esc 静默取消且可再次触发。双显示器中的全屏 Space 完成框选后，紧凑结果窗在同一显示器/工作区出现，自动化回归未发现代码问题。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 安装 0.3.5 后发送 `Command+Shift+2`，检查子进程和窗口 | pass | 仅启动系统 `screencapture -i -r -t png`；`See See Capture` 窗口数为 0 |
| Esc cancellation | 原生选择期间发送 Esc，检查进程并再次触发 | pass | 无错误/结果窗；进程退出；第二次截图正常启动 |
| Full-screen Space completion | 让 See See 主窗进入主屏全屏 Space，再触发并框选 `400×300` points 区域 | pass | 生成 PNG 并出现 `460×500` 紧凑结果窗；窗口与全屏主窗同时位于主显示器当前 Space |
| New / updated tests | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | Rust 全量通过，包括 11 个 lib tests 和原生选择器/reservation 回归 |
| Frontend regression | `npm test` | pass | 12 files、37 tests passed |
| Lint / format / type-check | `npm run lint`; `npm run format:check`; `npm run build`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | pass | 无 lint、格式、类型或 Clippy 问题 |
| Browser smoke | `npm run test:e2e` | pass | `✓ See See primary desktop flow` |
| Release/install | signed `npm run tauri build`；安装版版本、哈希和运行检查 | pass | 版本 `0.3.5`；安装版与 bundle 二进制哈希一致；应用正常完成原生截图和分析 |
| Windows manual capture | Windows 双屏混合 DPI 设备 | skipped | 当前验证环境仅为 macOS；Windows 源码继续使用原遮罩路径 |

## Output Excerpts

```text
/usr/sbin/screencapture -i -r -t png .../native-region-<uuid>.png
See See Capture windows: 0
Esc after: screencapture process absent
second trigger: screencapture process present
```

```text
name=See See · 识别结果 bounds={ Width = 460; Height = 500; X = 730; Y = 168; }
name=See See bounds={ Width = 1920; Height = 1080; X = 0; Y = 0; }
```

```text
Test Files  12 passed (12)
Tests       37 passed (37)
test result: ok
✓ See See primary desktop flow
```

## Residual Risks

- Windows 手工截图回归未在当前 macOS 环境执行；平台条件编译与全量 Rust/前端测试通过。
- 当前机器没有受信任的 Developer ID codesigning identity；本地 CMS 签名可运行，但严格证书信任检查返回 `CSSMERR_TP_NOT_TRUSTED`，正式分发前仍需 Developer ID 签名与公证。

## Recommendation

关闭此 bug。macOS 的原生十字选择、取消、重复触发、双显示器全屏 Space 和紧凑结果窗均已端到端验证；不要重新引入自制 macOS 截图遮罩。
