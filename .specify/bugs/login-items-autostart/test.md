# Bug Verification: macOS 原生登录项

- **Slug**: login-items-autostart
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

签名并安装的 0.3.3 应用已通过真实 `SMAppService` 完成注册和注销，应用状态、数据库和旧 LaunchAgent 清理均正确，自动化与完整回归检查通过。System Settings 已打开 Login Items 页面，但自动化辅助功能桥无法读取列表行，因此原始“列表可见性”症状尚缺最后一次人工视觉确认；注销/登录启动也未执行。

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Reproduction (post-fix) | 安装 0.3.3，打开开机启动并导航到 System Settings Login Items | partial | `SMAppService.status=Enabled`、数据库和开关均为 on；列表行读取时辅助功能桥关闭 |
| Disable / cleanup | 在安装应用中关闭开机启动 | pass | 系统注销成功后数据库和开关为 off；无旧 LaunchAgent，恢复测试前状态 |
| New Rust test | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` | pass | 4/4，包括 SMAppService 状态与审批错误契约 |
| New frontend test | `npm test -- --run src/views/Settings.desktop.test.tsx` | pass | 5/5，包括打开 Login Items 设置动作 |
| Frontend regression | `npm run lint && npm run format:check && npm test && npm run build` | pass | 12 个测试文件、37 个测试通过 |
| Rust regression | `cargo clippy ... -D warnings`、`cargo test --manifest-path src-tauri/Cargo.toml` | pass | Clippy 无警告；完整 Rust 测试通过 |
| Release build | `npm run tauri build` | pass | 生成签名的 0.3.3 arm64 `.app` 和 DMG |
| Signature | `npm run verify:macos-signature` | pass | 固定证书和 bundle identifier 验证通过 |
| Installed metadata | `plutil`、只读 SQLite、LaunchAgents 检查 | pass | 版本 0.3.3、最低 macOS 26.0、最终 autostart=0、无旧 plist |
| Login launch | 注销并重新登录 | skipped | 会中断当前桌面会话，未自动执行 |
| Windows regression | Windows 实机注册 | not-run | 当前主机为 macOS |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests       37 passed (37)

test result: ok. 4 passed; 0 failed

签名验证通过：src-tauri/target/release/bundle/macos/See See.app
证书：See See Local Release

installed version: 0.3.3
LSMinimumSystemVersion: 26.0
final autostart: 0
legacy-launch-agent-absent
```

## Residual Risks

- 仍需人工查看 System Settings，确认 See See 行在开启时出现、关闭时消失。
- 仍需一次注销/登录验证，确认启用后系统确实启动应用。
- 当前只构建了 arm64 包，尚未完成 universal 和 Windows 实机回归。
- `RequiresApproval` 分支已自动化覆盖，但本机未人为拒绝登录项后执行真实恢复流程。

## Recommendation

保留修复并进入人工验收。原生注册、状态一致性、签名包和回归测试均通过；在人工确认 Login Items 行与一次注销/登录启动后，可将此 bug 从 partial 更新为 verified 并关闭。
