# Bug Verification: 开机启动静默运行

- **Slug**: silent-autostart
- **Tested**: 2026-07-31
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

Windows 0.4.4 安装版已验证自启动时主窗口隐藏，普通启动和第二次启动仍会显示，旧注册表启动项也自动补上 `--autostart`。macOS 登录项判断已实现，但当前 Windows 主机无法完成 Apple 工具链编译和真实登录验证。

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Windows reproduction | 安装版以 `--autostart` 启动并检查 `See See` 顶层窗口 | pass | 主窗口 `visible=False`，进程与托盘保持运行 |
| Manual launch | 普通启动安装版 | pass | 主窗口可见 |
| Manual reopen | 后台启动后再次普通启动 | pass | 现有进程主窗口变为可见 |
| Upgrade migration | 读取 `HKCU\\...\\Run\\See See` | pass | 旧启动项刷新为 `see-see.exe --autostart` |
| Rust regression | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 全部通过 |
| Rust lint | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | pass | 无警告 |
| Frontend regression | `npm run lint && npm test && npm run build` | pass | 13 个测试文件、42 个测试通过 |
| Release build | `npm run tauri build` | pass | 生成 0.4.4 MSI 与 NSIS |
| Local install | NSIS `/S` | pass | FileVersion/ProductVersion 均为 0.4.4 |
| macOS compile | `cargo check --target aarch64-apple-darwin` | not-run | Windows 主机缺少 Apple C/Objective-C 编译工具链 |
| macOS login launch | 注销并重新登录 macOS | not-run | 当前没有 macOS 主机 |

## Output Excerpts

```text
Test Files  13 passed (13)
Tests       42 passed (42)

autostart_main_visible=False
manual_reopen_main_visible=True

run_entry=C:\Users\cekav\AppData\Local\See See\see-see.exe --autostart
FileVersion=0.4.4
ProductVersion=0.4.4
```

## Residual Risks

- macOS 的 `keyAELaunchedAsLogInItem` 分支需在签名安装包上完成编译和登录实测。
- 升级前已有的 Windows/Linux 启动项会在 0.4.4 首次运行时刷新；若安装后从未运行新版本，第一次登录仍可能沿用旧启动项一次。

## Recommendation

Windows 修复可以发布。macOS 包发布前补一次签名构建和真实登录启动检查，再将结果更新为 verified。
