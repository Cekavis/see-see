# Bug Fix: 首字等待过久时的流式请求超时与错误归类

- **Slug**: slow-first-token-timeout
- **Fixed**: 2026-08-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

流式分析现在使用独立的 HTTP client：保留 10 秒连接超时和 300 秒总请求上限，但不再使用 60 秒逐次读取超时，因此模型可以在思考阶段较长时间后再返回首字。普通模型列表和连接测试仍保留 60 秒读取保护；SSE 传输超时也会正确映射为 `timeout`。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/providers/mod.rs` | modified | 抽出 client builder；新增无逐次读取超时的 `streaming_client()`；解包 SSE transport error；新增延迟 SSE 回归测试 |
| `src-tauri/src/commands.rs` | modified | 初次分析和重试使用 streaming client，不再替换普通模型请求 client |
| `src-tauri/tests/desktop_lifecycle.rs` | modified | 更新 client 分工契约测试 |
| `src/views/Result.tsx` | modified | 首字到达前显示“等待模型首字…” |
| `src/views/Result.test.tsx` | modified | 覆盖首字等待状态 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | 同步版本升级到 `0.11.2` |

## Diff Highlights

- `client()` 使用 `read_timeout(60s)`，`streaming_client()` 不设置逐次读取超时；二者都保留 `timeout(300s)`。
- `EventStreamError::Transport(reqwest_error)` 复用现有 `map_reqwest_error`，读取超时不再显示为“模型流格式无效”。
- 重试不再把无读取超时 client 写入共享 client，避免模型列表/连接测试失去保护。

## Tests Added or Updated

- `src-tauri/src/providers/mod.rs::streaming_client_allows_a_long_first_token_wait` — 延迟 SSE 首字后仍能完成流式回答。
- `src-tauri/src/providers/mod.rs::stream_transport_timeouts_are_classified_as_timeouts` — 流中 transport timeout 返回 `ErrorCode::Timeout`。
- `src-tauri/tests/desktop_lifecycle.rs::analysis_uses_a_streaming_client_without_changing_model_request_client` — 分析使用独立 client，普通模型请求 client 不被替换。
- `src/views/Result.test.tsx` — 首字等待文案回归测试。

## Local Verification

- `cargo test --manifest-path src-tauri/Cargo.toml` → pass；18 个单元测试、各集成测试和文档测试全部通过。
- `C:\nvm4w\nodejs\npm.cmd test` → pass；14 个测试文件、62 个测试通过。
- `C:\nvm4w\nodejs\npm.cmd run build` → pass；TypeScript 与 Vite 生产构建通过。
- `C:\nvm4w\nodejs\npm.cmd run lint` → pass。
- `C:\nvm4w\nodejs\npm.cmd run format:check` → pass。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → pass。
- `git diff --check` → pass。

## Deviations from Assessment

实现采用了 assessment 中建议的流式专用 client，并额外移除了重试时替换共享 client 的旧逻辑。原因是该逻辑会把无逐次读取超时的流式 client 泄漏给模型列表和连接测试，破坏它们原有的 60 秒保护。

## Follow-ups

- 若未来需要让用户配置首字等待上限，可在保留 300 秒总截止时间的前提下增加独立设置；当前修复避免新增配置项。
