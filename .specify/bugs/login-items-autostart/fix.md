# Bug Fix: macOS 开机启动使用原生登录项

- **Slug**: login-items-autostart
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

macOS 开机启动已从插件默认 LaunchAgent 改为 `SMAppService.mainAppService`，注册成功后会成为系统主应用登录项。项目最低系统版本提升到 macOS 26，并明确不保留旧系统兼容路径。

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src-tauri/src/autostart.rs` | added | 绑定 ServiceManagement，处理注册、注销、状态、审批引导和旧 LaunchAgent 清理 |
| `src-tauri/src/lib.rs` | modified | 启动时用原生状态同步数据库并注册新 IPC 命令 |
| `src-tauri/src/commands.rs` | modified | 开关改用平台自启动适配器，增加打开 Login Items 设置命令 |
| `src-tauri/src/error.rs` | modified | 增加 `autostart_approval_required` 稳定错误码 |
| `src/ipc.ts` | modified | 暴露打开 Login Items 设置的 IPC 包装 |
| `src/views/DesktopSettings.tsx` | modified | 需要系统批准时提供“打开系统设置”操作 |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | 固定 SMAppService 状态映射和审批错误契约 |
| `src/views/Settings.desktop.test.tsx` | updated test | 固定审批通知及设置跳转行为 |
| `src-tauri/tauri.conf.json` | modified | 版本更新为 0.3.3，最低 macOS 版本更新为 26.0 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` | modified | 同步补丁版本 0.3.3 |
| `docs/macos-platform-support.md` | added | 记录 macOS 26+、SMAppService 和无旧系统 fallback 的维护策略 |
| `README.md`, `specs/` | modified | 同步平台要求、验收步骤和本次 macOS 验证结果 |

## Diff Highlights

- `SMAppService.status` 只有在 `Enabled` 时才允许数据库保存开启状态。
- `RequiresApproval` 返回带 `open_login_items` 操作的明确错误；界面可直接打开系统 Login Items。
- 新原生登录项成功注册或关闭开机启动后会清理旧版 `~/Library/LaunchAgents/See See.plist`。
- 如果原生注册成功但旧 LaunchAgent 清理失败，会回滚原生注册，避免系统状态与数据库状态分离。

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::macos_login_item_status_requires_explicit_approval` — 覆盖四个已知 SMAppService 状态、未知状态及审批错误动作。
- `src/views/Settings.desktop.test.tsx::opens macOS Login Items when autostart requires approval` — 覆盖审批通知和系统设置跳转。
- 现有 `desktop_settings_only_persist_after_system_success` — 继续保证系统调用失败时不写数据库。

## Local Verification

- `npm run lint && npm run format:check && npm test && npm run build` → 通过；12 个测试文件、37 个测试通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → 通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` → 通过；Wiremock 用例需在允许绑定本机端口的环境运行。
- `npm run tauri build` → 通过；生成签名的 0.3.3 arm64 `.app` 和 DMG，最低系统版本为 26.0。
- `npm run verify:macos-signature` → 通过；证书为 `See See Local Release`。
- 安装后开启开关 → `SMAppService.status` 为 Enabled、数据库为 `1`、界面为 on，且没有旧 LaunchAgent。
- 安装后关闭开关 → 注销成功、数据库为 `0`、界面为 off；已恢复测试前状态。

## Deviations from Assessment

- 根据用户补充要求，最低版本从评估时的 macOS 14 提升到 macOS 26，并明确不实现旧系统兼容分支。
- 新增独立 `src-tauri/src/autostart.rs`，避免把 Objective-C ServiceManagement 绑定散落在命令和应用启动代码中。
- 增加前端 IPC 与审批操作，使 `RequiresApproval` 状态可恢复，而不是只显示通用存储错误。
- System Settings 已打开 Login Items 页面，但自动化辅助功能桥在读取列表行时关闭；可视条目仍需人工复核。

## Follow-ups

- 在最低支持版本 macOS 26 上人工确认 See See 行在开启时出现、关闭时消失。
- 注销并重新登录，验证开启状态下应用自动启动。
- 在 Windows 主机回归现有 Tauri autostart 后端。
