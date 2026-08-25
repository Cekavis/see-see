# Bug Assessment: 首字等待过久时的流式请求超时与错误归类

- **Slug**: slow-first-token-timeout
- **Created**: 2026-08-25
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 现在模型输出太慢会超时吗？有的模型思考结果不输出，可能需要比较久才首字。

## Symptom

当前实现确实会在模型长时间没有任何响应数据时超时。若模型在首个 token 前超过读取等待窗口，分析会失败；若已经收到 HTTP 响应但后续流读取超时，该错误目前还可能被归类为“模型流格式无效”，而不是“模型请求超时”。

## Reproduction

1. 配置一个支持流式响应的模型端点。
2. 让端点返回成功 HTTP 响应，但在发送首个 SSE 数据块前保持连接静默超过 60 秒；或让响应流中途静默超过 60 秒。
3. 观察结果窗口：请求失败；流式读取路径的错误文案可能显示“模型流格式无效”。

若模型端点在等待期间持续发送 SSE/HTTP 心跳字节，则 `read_timeout` 会因每次成功读取而重置，首字可能延迟超过 60 秒；但整次请求仍受 300 秒总期限限制。

## Suspected Code Paths

- `src-tauri/src/providers/mod.rs:296-301` — 共享 `reqwest::Client` 设置了 10 秒连接超时、60 秒逐次读取超时和 300 秒总请求超时。
- `src-tauri/src/providers/mod.rs:306-345` — 流式响应使用 `bytes_stream().eventsource()`；首 token 前没有单独的“首字宽限期”或心跳策略。
- `src-tauri/src/providers/mod.rs:345` — `eventsource_stream` 的所有流错误都被映射为 `ProviderError("模型流格式无效")`，未区分底层 `reqwest::Error` 的 timeout。
- `src-tauri/src/analysis.rs:331-367` — 网络分析任务只在取消信号和 `stream_text` 完成之间选择；超时后通过失败事件结束，取消按钮在等待期间仍可用。
- `src/views/Result.tsx:58-68,87-92` — 在收到 thinking/text delta 前状态保持 `submitting`，界面显示“正在提交图片…”和“等待模型返回文字…”，收到首个 thinking/text delta 后才变为 `streaming`。

## Root Cause Hypothesis

**高置信度**：问题不是“思考结果没有输出”本身，而是请求层没有把“首字等待”与“流中断”单独建模。按 reqwest 0.12.28 的客户端语义，`read_timeout` 是每次读取操作的超时，成功读取后重置；因此无任何字节的静默阶段约 60 秒即会触发，持续心跳可延长该阶段，但 `.timeout(300s)` 仍是整次请求的硬上限。流开始后发生的底层读取超时经过 `eventsource_stream::EventStreamError::Transport`，当前代码把它统一包装成流格式错误，导致用户无法判断是模型慢、网络断开还是 SSE 格式错误。

## Proposed Remediation

**Preferred**: 保留总请求上限作为安全边界，但为流式请求明确区分首字等待和后续读取策略。最小改动是把 `eventsource_stream` 的 transport error 解包并复用 `map_reqwest_error`，确保读取超时显示为 `Timeout`；同时为结果窗口增加明确的“等待模型首字”状态/提示。若产品要求支持超过 60 秒首字，再仅对流式分析客户端提高或关闭逐次 `read_timeout`，并用 300 秒总期限兜底，避免影响模型列表和连接测试。

**Alternatives**:

- 要求供应商发送 SSE 心跳：无需改变客户端超时，但依赖各供应商行为，不能解决没有心跳的模型。
- 仅延长全局 60 秒读取超时：改动最小，但会让模型列表、连接测试和分析共享更长的网络等待，且不能正确区分错误类型。

**Files likely to change**:

- `src-tauri/src/providers/mod.rs`
- `src-tauri/tests/provider_contracts.rs`
- `src-tauri/src/analysis.rs` and/or `src/views/Result.tsx` for explicit first-token status
- `src/views/Result.test.tsx`

**Tests to add or update**:

- 使用 mock SSE 端点验证流开始后产生 `reqwest` timeout 时返回 `ErrorCode::Timeout`，而不是 `ProviderError`。
- 验证首字前没有 delta 时状态保持可取消，并在超时后发送失败事件。
- 验证心跳字节会重置逐次读取超时，但总请求期限仍能结束请求。
- 验证收到 thinking delta 后仍显示思考内容，并在收到正式文本前保持“等待正式回答”提示。

## Risks & Considerations

- 提高或关闭逐次读取超时会让无响应端点占用更久；必须保留总请求截止时间和取消路径。
- 修改错误映射会改变历史记录中的错误码/文案，但属于修正错误分类，不应破坏 API 合同。
- 仅在 UI 改文案不能解决底层请求超时；必须先确认客户端超时策略。

## Open Questions

- [NEEDS CLARIFICATION: 产品希望允许的最长首字等待时间是多少；是否接受分析最长等待 5 分钟？]
- [NEEDS CLARIFICATION: 目标供应商是否会在思考阶段发送 SSE 心跳字节？]
