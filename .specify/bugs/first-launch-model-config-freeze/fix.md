# Bug Fix: 首次启动配置模型窗口空白卡死

- **Slug**: first-launch-model-config-freeze
- **Fixed**: 2026-07-23
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

将 `open_view` 改为异步 Tauri 命令，避免在 WebView IPC 回调中重入创建 Windows WebView；托盘入口将同一 Future 提交到 Tauri runtime。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | `open_view` 改为异步命令 |
| `src-tauri/src/lib.rs` | modified | 托盘窗口入口提交异步任务 |
| `package.json` | modified | 补丁版本升级到 0.1.2 |
| `package-lock.json` | modified | 同步 npm 包版本 |
| `src-tauri/Cargo.toml` | modified | 补丁版本升级到 0.1.2 |
| `src-tauri/Cargo.lock` | modified | 同步 Rust 包版本 |
| `src-tauri/tauri.conf.json` | modified | 同步应用版本 |

## Diff Highlights

`open_view` 的窗口逻辑保持不变，仅从同步命令改为异步命令；没有新增依赖或窗口抽象。

## Tests Added or Updated

- 未增加代码测试；真实 Tauri/WebView2 回归验证直接覆盖该线程重入问题，现有编译与测试覆盖命令签名及托盘调用。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → 通过。
- Commands run: `npm test` → 8 个测试文件、15 个测试全部通过。
- Commands run: `npm run build` → 通过。
- Commands run: `npm run lint` → 通过。
- Commands run: `npm run format:check` → 通过。
- Manual checks: 首次启动状态下，修复前点击后新目标停在 `about:blank`；修复后新窗口加载 `http://127.0.0.1:1420/`，`document.readyState` 为 `complete`，并显示完整“模型设置”表单。

## Deviations from Assessment

无。

## Follow-ups

- 无。
