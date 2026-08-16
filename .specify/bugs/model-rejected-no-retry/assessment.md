# Bug Assessment: 模型服务拒绝请求时没有重试按钮

- **Slug**: model-rejected-no-retry
- **Created**: 2026-08-16
- **Source**: pasted text
- **Verdict**: likely valid, needs reproduction
- **Severity**: low

## Report (verbatim or summarized)

> 为什么现在出现“模型服务拒绝了请求”的情况没有重试按钮

## Symptom

模型分析失败并显示“模型服务拒绝了请求”时，结果窗口不显示“重试”按钮。用户期望能够在失败结果窗口内再次提交请求。

## Reproduction

1. 配置一个返回未单独分类 HTTP 错误状态的模型端点。
2. 发起截图分析，使模型端点返回非成功响应。
3. 确认结果窗口显示“模型服务拒绝了请求”，但没有“重试”按钮。

[NEEDS CLARIFICATION: 本次失败详情中的具体 HTTP 状态码和响应正文。]

## Suspected Code Paths

- `src-tauri/src/providers/mod.rs:422` — `map_status` 将未单独匹配的 HTTP 状态统一映射为“模型服务拒绝了请求”，并将 `retryable` 设为 `false`。
- `src/views/Result.tsx:171` — 结果窗口仅在失败错误的 `retryable` 为 `true` 时渲染“重试”按钮。
- `src-tauri/src/analysis.rs:291` — 后端 `reset_for_retry` 同样拒绝重试 `retryable == false` 的失败，说明缺少按钮不是单纯的前端渲染问题。
- `src-tauri/src/commands.rs:419` — 重试会用保留的截图和当前模型、提示词配置重新发起分析。

## Root Cause Hypothesis

**Confidence: high.** “模型服务拒绝了请求”是所有未被专门识别的非成功 HTTP 状态的兜底错误；该兜底分支把错误标成不可重试。前端和后端都以同一个 `retryable` 字段作为手动重试许可，因此按钮被隐藏且 IPC 也会拒绝重试。这个分类自 2026-07-23 的初始实现起即存在；2026-08-04 新增结果窗口重试功能时只接入了既有的可重试分类，因此不是近期按钮逻辑被移除。

当前信息不足以判断分类本身是否错误：若详情为 HTTP 400、405、413、415 或 422，原请求通常需要修改而不是原样重试；若为 HTTP 408、425 或供应商自定义的临时错误状态，兜底为不可重试则过于宽泛。

## Proposed Remediation

**Preferred**: 先根据失败详情中的具体 HTTP 状态确认语义，再在 `map_status` 中只补充已知的临时状态为可重试，保留认证、模型不存在、请求格式和图片能力等永久性错误为不可重试。不要只改前端显示条件，否则按钮出现后后端仍会返回“当前分析不可重试”。

如果产品意图是允许用户对所有未知供应商错误进行手动重试，则应把兜底分支改为 `retryable: true`，并以测试明确“手动重试许可”语义；当前代码不存在自动重试循环，因此主要风险是用户重复提交一个确定无效的请求。

**Files likely to change**:

- `src-tauri/src/providers/mod.rs`
- `src-tauri/tests/provider_contracts.rs`
- `src/views/Result.test.tsx`（仅当产品规则改为所有未知供应商错误均显示按钮）

**Tests to add or update**:

- 覆盖本次实际 HTTP 状态映射出的 `retryable` 值。
- 保持 401、403、404 和图片不支持错误不可重试。
- 保持 429、5xx 和网络超时错误可重试。
- 若兜底策略改变，验证结果窗口显示“重试”且 `retry_analysis` 能重置该运行。

## Risks & Considerations

- 把所有 4xx 一律设为可重试会给永久性请求错误提供无效操作。
- 只放宽前端会造成可见按钮必然被后端拒绝。
- 供应商可能使用非标准状态码，修复应以实际响应和兼容目标为依据。

## Open Questions

- [NEEDS CLARIFICATION: 错误详情显示的 HTTP 状态码是什么？]
- [NEEDS CLARIFICATION: 产品是否希望“重试”表示任何失败都可手动再次提交，还是只表示请求可能原样成功？]
