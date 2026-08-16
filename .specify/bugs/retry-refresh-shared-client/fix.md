# Bug Fix: 重试刷新后续请求的共享连接池

- **Slug**: retry-refresh-shared-client
- **Fixed**: 2026-08-16
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

重试创建的新 HTTP 客户端现在会替换应用共享客户端。重试本身及替换后的普通分析、远程模型列表和模型连接测试都会使用同一个新连接池。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/state.rs` | modified | 共享 HTTP 客户端改为 `Mutex<Client>`，允许安全替换 |
| `src-tauri/src/commands.rs` | modified | 新请求 clone 当前共享客户端；重试在重置后写回新客户端 |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | 约束替换顺序及所有模型请求读取当前共享客户端 |

## Diff Highlights

- `http_client` 只在请求开始前短暂加锁并 clone `Client`，不会在异步网络等待期间持锁。
- `retry_analysis` 先创建新客户端并锁定共享槽，成功重置分析后替换共享客户端，再释放锁并启动重试。
- 已在途请求保留其原客户端；替换之后创建的请求使用新连接池。

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::retry_analysis_replaces_the_shared_http_client_for_future_requests` — 固定普通分析、模型列表、连接测试和重试的共享客户端行为与调用顺序。
- `src-tauri/tests/analysis_flow.rs::retry_resets_all_failures_and_keeps_the_source_image` — 保持所有分析错误均可重试。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle --test analysis_flow --test provider_contracts` → pass，28 tests。
- `npm run lint`; `npm run format:check`; `npm test` → pass，前端 60 tests。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml` → pass，Rust 67 tests。
- 签名 `npm run tauri build` → pass，生成 0.10.2 MSI、NSIS 和两个 updater 签名。
- NSIS `/S` 覆盖安装 → pass，退出码 0；安装版文件/产品版本为 0.10.2，可隐藏启动并保持响应。

## Deviations from Assessment

无。

## Follow-ups

- 无。
