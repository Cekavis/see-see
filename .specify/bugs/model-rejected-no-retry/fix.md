# Bug Fix: 所有分析错误允许使用新连接重试

- **Slug**: model-rejected-no-retry
- **Fixed**: 2026-08-16
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

结果窗口现在为所有分析失败提供手动重试，不再用供应商错误的 `retryable` 分类限制按钮或后端重置。重试会创建新的 `reqwest::Client` 和连接池，首次分析继续使用应用共享客户端。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/views/Result.tsx` | modified | 所有带 `onRetry` 的失败结果均显示“重试” |
| `src-tauri/src/analysis.rs` | modified | 失败状态均可重置；网络分析显式接收 HTTP 客户端 |
| `src-tauri/src/commands.rs` | modified | 首次分析传入共享客户端；重试通过 `providers::client()` 新建客户端 |
| `src/views/Result.test.tsx` | updated test | 验证 `retryable: false` 的错误仍显示并执行重试 |
| `src-tauri/tests/analysis_flow.rs` | updated test | 验证不可重试分类的认证失败也能重置，同时非失败终态仍被拒绝 |
| `src-tauri/tests/desktop_lifecycle.rs` | added test | 约束首次分析复用共享客户端、重试在重置前新建客户端 |
| Version files | modified | 同步升级至 `0.10.2` |

## Diff Highlights

- 手动重试许可只取决于分析是否处于 `failed` 状态，不再复用面向错误分类的 `retryable` 字段。
- `start_network_analysis` 显式接收 `reqwest::Client`；`retry_analysis` 在重置运行前创建新客户端，创建失败时保留原失败状态。

## Tests Added or Updated

- `src-tauri/tests/analysis_flow.rs::retry_resets_all_failures_and_keeps_the_source_image` — 固定所有失败可重置以及非失败终态不可重置。
- `src-tauri/tests/desktop_lifecycle.rs::retry_analysis_creates_a_fresh_http_client_before_resetting` — 固定重试新建客户端及调用顺序。
- `src/views/Result.test.tsx::renders completed and failed terminal states without unsafe rich text` — 固定 `retryable: false` 时仍可见并可点击重试。
- `src-tauri/tests/provider_contracts.rs` — 现有合同测试继续确认错误分类不会触发自动重试。

## Local Verification

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → pass。
- `cargo test --manifest-path src-tauri/Cargo.toml --test analysis_flow --test provider_contracts` → pass，13 tests。
- `npm test -- src/views/Result.test.tsx` → pass，9 tests。
- `npm run lint`; `npm run format:check`; `npm test`; `npm run build` → pass，前端 60 tests。
- `cargo test --manifest-path src-tauri/Cargo.toml` → pass，Rust 67 tests。
- 签名 `npm run tauri build` → pass，生成 0.10.2 MSI、NSIS 和两个 updater 签名。
- NSIS `/S` 安装与隐藏启动 → pass；安装器退出码 0，文件/产品版本均为 0.10.2，进程响应正常。

## Deviations from Assessment

- 用户明确要求所有错误均允许手动重试，因此没有仅补充个别 HTTP 状态，而是把手动重试许可与 `AppError.retryable` 分类解耦。
- 用户新增“重试必须新建连接”的要求，因此扩展到 `src-tauri/src/commands.rs` 和 `src-tauri/src/analysis.rs`，显式传递首次请求的共享客户端和重试的新客户端。

## Follow-ups

- 无。
