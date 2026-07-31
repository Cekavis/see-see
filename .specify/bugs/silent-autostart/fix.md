# Bug Fix: 开机启动静默运行

- **Slug**: silent-autostart
- **Fixed**: 2026-07-31
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

主窗口改为默认隐藏，仅在普通启动时显式显示。Windows/Linux 自启动使用 `--autostart`，macOS 使用登录项 Apple Event 判断；旧的非 macOS 启动项会在应用启动时自动刷新。

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs` | modified | 统一启动显示判断，普通第二实例仍显示主窗口 |
| `src-tauri/src/autostart.rs` | modified | 增加自启动参数、macOS 登录项识别和旧启动项刷新 |
| `src-tauri/tauri.conf.json` | modified | 主窗口默认隐藏，版本更新为 0.4.4 |
| `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` | modified | 启用 Apple Event 绑定并同步 0.4.4 |
| `package.json`, `package-lock.json` | modified | 同步 0.4.4 |

## Tests Added or Updated

- `src-tauri/src/lib.rs::tests::autostart_launches_stay_silent` — 覆盖普通启动、参数自启动和 macOS 登录项启动的显示判断。

## Local Verification

- `npm run lint && npm test && npm run build` → passed；13 个测试文件、42 个测试通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed。
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed。
- `npm run tauri build` → passed；生成 0.4.4 MSI 与 NSIS。
- NSIS `/S` 安装 → passed；安装版文件与产品版本均为 0.4.4。
- 安装版 `--autostart` 启动 → 主窗口不可见；随后普通第二次启动会显示主窗口。

## Deviations from Assessment

- 增加已启用 Windows/Linux 启动项的幂等刷新，避免升级用户必须手动关闭再打开“开机启动”。

## Follow-ups

- 在 macOS 签名安装包上执行一次登录启动实测。
