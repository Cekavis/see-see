# Bug Verification: 重试刷新后续请求的共享连接池

- **Slug**: retry-refresh-shared-client
- **Tested**: 2026-08-16
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

自动化合同确认重试会在启动网络任务前替换共享 HTTP 客户端，之后的普通分析、远程模型列表和连接测试都从该共享槽 clone 当前客户端。完整测试、签名构建、覆盖安装和启动检查均通过。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | `cargo test --test desktop_lifecycle` | pass | 新客户端在重试启动前写回共享槽，三个后续模型请求入口读取当前共享客户端 |
| Retry behavior regression | `cargo test --test analysis_flow --test provider_contracts` | pass | 所有失败仍可重试，供应商请求合同保持正常 |
| Frontend regression suite | `npm test` | pass | 14 files，60 tests |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 67 tests，0 failed |
| Lint / formatting / build | `npm run lint`; `npm run format:check`; `cargo fmt -- --check`; 签名 `npm run tauri build` | pass | 静态检查、生产构建、MSI、NSIS 和 updater 签名均成功 |
| Local installation | NSIS `/S` 覆盖安装并隐藏启动 | pass | 退出码 0；File/Product version 为 0.10.2；进程保持响应 |

## Output Excerpts

- `retry_analysis_replaces_the_shared_http_client_for_future_requests ... ok`。
- `Test Files 14 passed (14)`；`Tests 60 passed (60)`。
- Rust 所有测试目标合计 67 项，均为 `0 failed`。
- `Finished 2 bundles`；`Finished 2 updater signatures`。

## Residual Risks

- 未对真实供应商执行会消耗额度的失败、重试、再请求流程；连接池替换及所有读取入口由自动化源代码合同和 Rust 编译共同约束。
- 替换不会终止已经在途的旧请求；这是有意行为，只有替换后创建的新请求使用新连接池。

## Recommendation

关闭该缺陷。重试后的共享连接池替换、后续请求入口、完整回归和安装版本均已验证。
