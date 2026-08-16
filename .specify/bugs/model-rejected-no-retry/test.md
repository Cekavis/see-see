# Bug Verification: 所有分析错误允许使用新连接重试

- **Slug**: model-rejected-no-retry
- **Tested**: 2026-08-16
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

自动化等价复现确认原本标记为不可重试的分析错误现在仍显示“重试”并可重置运行；重试命令在重置前创建新的 HTTP 客户端，首次分析继续使用共享客户端。完整测试、签名发布构建、本地安装和启动检查均通过。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | `npm test -- src/views/Result.test.tsx`; `cargo test --test analysis_flow` | pass | `retryable: false` 的错误显示按钮，认证失败可重置并保留截图 |
| Fresh connection contract | `cargo test --test desktop_lifecycle` | pass | 首次分析 clone 共享客户端；重试先调用 `providers::client()`，再重置并启动 |
| Frontend regression suite | `npm test` | pass | 14 files，60 tests |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 67 tests，0 failed |
| Lint / formatting / build | `npm run lint`; `npm run format:check`; `cargo fmt -- --check`; `npm run build` | pass | ESLint、Prettier、rustfmt、TypeScript 和 Vite 均通过 |
| Release build | 签名 `npm run tauri build` | pass | 生成 0.10.2 MSI、NSIS 和两个 updater `.sig` |
| Local installation | NSIS `/S` 后读取版本并隐藏启动 | pass | 安装器退出码 0；File/Product version 为 0.10.2；进程响应正常 |

## Output Excerpts

- `Test Files 14 passed (14)`；`Tests 60 passed (60)`。
- Rust 所有测试目标合计 67 项，均为 `0 failed`。
- `retry_analysis_creates_a_fresh_http_client_before_resetting ... ok`。
- `Finished 2 bundles`；`Finished 2 updater signatures`。
- 安装路径 `C:\Users\cekav\AppData\Local\See See\see-see.exe`，版本 0.10.2。

## Residual Risks

- 未消耗真实模型额度执行一次远端失败再重试；UI、状态机和新客户端调用顺序由自动化等价场景分别覆盖。
- “新连接”由新的 `reqwest::Client` 实例和独立连接池保证，没有强制供应商关闭或重新建立 TLS 之外的上游代理连接。

## Recommendation

关闭该缺陷。所有分析失败的手动重试、新客户端创建顺序、完整回归和安装版本均已验证。
