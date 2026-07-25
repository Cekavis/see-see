# Bug Verification: 模型配置操作在 IPC 边界失效

- **Slug**: model-config-actions-no-feedback
- **Tested**: 2026-07-25
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

协议 Serde 根因及模型命令的真实参数形态已通过 Rust 边界测试验证，前端字符串错误处理、全量测试、构建、打包和安装均通过。由于 macOS 拒绝当前自动化进程的辅助功能控制，无法代用户在安装后的原生 WKWebView 中执行最后一次点击，故不将结果夸大为 verified。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 在安装后的 See See 中自动点击三个按钮 | not-run | `osascript is not allowed assistive access (-1719)`；应用已启动，需用户手动点击 |
| IPC protocol boundary | `cargo test --manifest-path src-tauri/Cargo.toml --test foundation` | pass | `openai` 三协议往返，以及两个真实命令输入结构反序列化通过 |
| New / updated frontend tests | `npm test -- --run src/ipc.test.ts src/views/Settings.model.test.tsx` | pass | 2 个文件、4 个测试通过 |
| Frontend regression suite | `npm test` | pass | 11 个文件、21 个测试全部通过 |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 全部单元、集成、合约和文档测试通过 |
| Desktop smoke flow | `npm run test:e2e` | pass | `✓ See See primary desktop flow` |
| Lint / formatting / build | `npm run lint`; `npm run format:check`; `npm run build`; `cargo fmt --check` | pass | 全部通过 |
| Release bundle | `npm run tauri build` | pass | `.app` 和 `See See_0.2.2_aarch64.dmg` 生成成功 |
| Local installation | 安装到 `/Applications/See See.app` 并读取 Info.plist | pass | 已安装并启动版本 0.2.2 |
| Unwanted layout regression | `git diff -- src/styles.css` | pass | 无 CSS 差异、无反馈占位高度 |

## Output Excerpts

```text
test provider_protocol_json_matches_the_ipc_contract ... ok
Test Files  11 passed (11)
Tests       21 passed (21)
✓ See See primary desktop flow
Finished 2 bundles
```

## Residual Risks

- 需要用户在当前已打开的 0.2.2 应用中手动点击按钮，确认真实 WKWebView 交互；自动化受 macOS 辅助功能权限阻止。
- “获取模型列表”和“测试连接”的成功结果仍取决于用户提供的真实端点、模型 ID 与凭据。

## Recommendation

保持缺陷为待人工确认：请在当前应用的模型页点击一次“保存配置”。空表单应显示“配置名称需为 1 到 80 个字符”；填入有效配置后，三个按钮应分别进入对应业务流程。收到该确认后可将结果升级为 verified。
