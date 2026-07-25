# Bug Fix: 模型配置操作在 IPC 边界失效

- **Slug**: model-config-actions-no-feedback
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

修正 OpenAI 协议在 Rust Serde 与 TypeScript IPC 合同之间的值不一致，使三个模型操作能够进入真实 Tauri 命令；同时让模型设置正确显示结构化或字符串形式的 IPC 错误。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/providers/mod.rs` | modified | `ProviderProtocol` 改用 lowercase Serde，与 `openai` 合同一致 |
| `src-tauri/tests/foundation.rs` | added regression coverage | 验证三协议 JSON 往返及真实模型命令参数反序列化 |
| `src/ipc.ts` | modified | 新增安全的 `getErrorMessage(unknown)` |
| `src/ipc.test.ts` | added | 覆盖结构化、Error、字符串和未知对象拒绝 |
| `src/views/Settings.tsx` | modified | 所有模型设置 catch 不再强制假设 `AppError` |
| `src/views/Settings.model.test.tsx` | updated test | 用 Tauri 框架级字符串拒绝验证可见错误 |
| 版本文件 | modified | 复用并同步 0.2.2 |

## Diff Highlights

`ProviderProtocol::OpenAi` 现在序列化为并接受 `"openai"`，不再是错误的 `"open_ai"`。获取模型、连接测试和保存配置共用的结构化参数因此可以通过 Tauri 命令反序列化。

本次未修改 `src/styles.css`，也没有增加固定高度或空白占位。

## Tests Added or Updated

- `src-tauri/tests/foundation.rs::provider_protocol_json_matches_the_ipc_contract` — 锁定三协议 JSON 值，并验证 `ModelConnectionInput`、`ModelConfigInput` 接受真实前端 payload。
- `src/ipc.test.ts` — 锁定 IPC 拒绝值的安全消息提取。
- `src/views/Settings.model.test.tsx` — 字符串拒绝仍显示“无法获取模型列表”。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test foundation` → 3 个测试通过。
- Commands run: `npm test -- --run src/ipc.test.ts src/views/Settings.model.test.tsx` → 2 个文件、4 个测试通过。
- Commands run: `npm run typecheck` → 通过。
- Commands run: `npm run lint` → 通过。
- Manual checks: 源码差异确认没有 CSS 或固定反馈高度改动。

## Deviations from Assessment

无。

## Follow-ups

- 在重新构建并安装的真实 macOS 应用中复查三个操作的 Tauri IPC 行为。
