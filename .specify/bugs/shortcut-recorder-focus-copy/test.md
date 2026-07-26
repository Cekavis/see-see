# Bug Verification: macOS 快捷键窗口捕获与辅助文案精简

- **Slug**: shortcut-recorder-focus-copy
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

原始 macOS 症状已在安装后的 `0.3.1` 应用中消失：录制器可以从窗口接收 Command 组合键、立即保存并在重启后保持。重复的开机启动副标题已移除，保留的辅助文案均提供独立信息；完整自动化与发布构建未发现回归。

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Reproduction (post-fix) | 安装 `0.3.1`，点击录入器，按 `Command+Shift+K` | pass | 录制态立即结束并显示“快捷键已保存并生效” |
| Persistence | 退出并重新启动 `/Applications/See See.app` | pass | 界面仍显示 `Command+Shift+K` |
| Default restoration | 再次录入 `Command+Shift+X` | pass | 成功恢复 macOS 默认组合 |
| Copy audit | 检查常规设置及全部 React 辅助文案 | pass | 删除同义副标题；保留状态、约束、隐私、费用、恢复与不可逆信息 |
| New / updated tests | `npm test -- --run src/views/Settings.desktop.test.tsx` | pass | 4 tests passed |
| Frontend regression suite | `npm test` | pass | 12 files / 36 tests passed |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 所有单元、集成与契约测试通过 |
| Browser smoke flow | `npm run test:e2e` | pass | See See primary desktop flow passed |
| Lint / formatting / build | `npm run lint`; `npm run format:check`; `npm run build` | pass | 全部通过 |
| Release build | `npm run tauri build` | pass | `.app` 与 `.dmg` 均成功生成 |
| Local installation | 备份旧版并安装 `/Applications/See See.app` | pass | 已安装版本 `0.3.1` |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests  36 passed (36)
```

```text
test result: ok
✓ See See primary desktop flow
Finished 2 bundles: See See.app, See See_0.3.1_aarch64.dmg
```

```text
真实应用通知：快捷键已保存并生效
重启后显示：Command+Shift+K
最终恢复：Command+Shift+X
```

## Residual Risks

- macOS 系统保留的组合键仍可能在事件到达 WebView 前被系统拦截，这是操作系统限制。
- Windows 默认值与录入交互已由代码和自动化覆盖，但本次没有 Windows 真机人工验证。

## Recommendation

关闭缺陷。macOS 原始复现已完成端到端验证，自动化、发布构建和本机安装均通过。
