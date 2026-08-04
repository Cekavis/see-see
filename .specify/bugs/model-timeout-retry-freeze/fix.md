# Bug Fix: 模型超时重试导致应用卡死

- **Slug**: model-timeout-retry-freeze
- **Fixed**: 2026-08-04
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

修复了 Windows 从失败记录重新提交时在同步 IPC 中创建结果窗口导致的死锁，并让可重试的模型失败直接在原结果窗口重新执行。模型错误现在可显示经过敏感信息清理和长度限制的原始响应详情，关于页也不再回退到陈旧版本号。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/commands.rs` | modified | `resubmit_history` 改为异步；新增原窗口 `retry_analysis`；复用分析输入构建 |
| `src-tauri/src/analysis.rs` | modified | 活动分析保留截图并允许可重试失败安全重置；历史记录保存完整错误详情 |
| `src-tauri/src/error.rs` | modified | `AppError` 增加可选 `details` 和组合显示方法 |
| `src-tauri/src/providers/mod.rs` | modified | 输出 HTTP 状态和清理、截断后的原始响应；网络错误保留底层详情 |
| `src-tauri/src/history.rs` | modified | 同一运行重试时覆盖原失败记录和截图，避免主键冲突丢失成功结果 |
| `src-tauri/src/lib.rs` | modified | 注册 `retry_analysis` IPC 命令 |
| `src/App.tsx`, `src/ipc.ts` | modified | 接入重试 IPC；重试开始时清空旧失败状态 |
| `src/views/Result.tsx` | modified | 在结果窗口内显示错误详情和带忙碌状态的重试按钮 |
| `src/views/SettingsShell.tsx` | modified | 使用当前包版本作为原生版本读取失败时的回退值 |
| Rust / frontend tests | added or modified | 覆盖异步重提、运行重置、历史覆盖、响应清理、结果重试和版本回退 |
| version files and lockfiles | modified | 同步升级到 `0.5.2` |

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::result_window_creation_stays_out_of_synchronous_windows_commands` — 同时约束截图完成和历史重提命令保持异步。
- `src-tauri/tests/analysis_flow.rs::retry_resets_only_retryable_failures_and_keeps_the_source_image` — 约束重试状态和截图保留。
- `src-tauri/tests/provider_contracts.rs::provider_response_details_are_bounded_and_redact_sensitive_json` — 约束原始响应输出、敏感字段清理和长度限制。
- `src-tauri/tests/history_integration.rs::retry_replaces_the_failed_history_entry_for_the_same_run` — 约束同一运行的失败记录被重试结果覆盖。
- `src/App.test.ts::analysis event state` — 约束重试开始时清空旧错误。
- `src/views/Result.test.tsx` — 约束结果窗口显示详情并提供重试按钮。
- `src/views/SettingsShell.test.tsx` — 约束原生版本调用失败时显示当前包版本。

## Local Verification

- `npm test` → 14 files / 44 tests passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` → all unit, integration, benchmark and doc tests passed.
- `npm run format:check` → passed.
- `npm run lint` → passed.
- `npm run build` → passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- `npm run tauri -- build` → passed; MSI and NSIS bundles generated for 0.5.2.
- NSIS install → installed executable reports ProductVersion and FileVersion `0.5.2`.

## Deviations from Assessment

- Expanded the implementation to `src-tauri/src/history.rs` after confirming that same-window retry reuses the run ID. Without an upsert, a successful retry would conflict with the saved failed record and silently remain unsaved.

## Follow-ups

- None.
