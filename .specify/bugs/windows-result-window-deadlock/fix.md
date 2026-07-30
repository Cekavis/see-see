# Bug Fix: Windows 截图后结果窗口死锁

- **Slug**: windows-result-window-deadlock
- **Fixed**: 2026-07-30
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

将 Windows 截图完成命令改为异步 Tauri 命令，避免在同步 IPC 上下文中创建 WebView 结果窗口导致死锁。IPC 契约和截图分析流程保持不变。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | 将 `finish_capture` 改为 `async fn` |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 编译级约束 `finish_capture` 保持异步 |
| `package.json` | modified | 补丁版本升级至 0.4.2 |
| `package-lock.json` | modified | 同步 npm 根包版本 |
| `src-tauri/Cargo.toml` | modified | 补丁版本升级至 0.4.2 |
| `src-tauri/Cargo.lock` | modified | 同步 Rust 根包版本 |
| `src-tauri/tauri.conf.json` | modified | 补丁版本升级至 0.4.2 |

## Diff Highlights

```rust
#[tauri::command]
pub async fn finish_capture(
```

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::result_window_creation_stays_out_of_synchronous_windows_commands` — 编译时要求命令返回 Future，防止重新引入 Windows 同步窗口创建死锁。

## Local Verification

- Commands run: `npm run build` → passed.
- Commands run: `npm test` → 13 files / 41 tests passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm run format:check` → passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → all unit, integration, benchmark and doc tests passed.
- Commands run: `npm run tauri build` → passed; MSI and NSIS bundles generated.
- Manual checks: installed See See 0.4.2 on Windows, invoked the configured capture shortcut, selected a region, and observed the capture overlay close and the responsive `See See · 识别结果` window appear.

## Deviations from Assessment

- Added `package-lock.json` and generated `src-tauri/Cargo.lock` updates so lockfile package versions match 0.4.2. No production behavior beyond the assessed remediation changed.

## Follow-ups

- None.
