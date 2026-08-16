# Bug Assessment: 重试后仍复用旧共享连接池

- **Slug**: retry-refresh-shared-client
- **Created**: 2026-08-16
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

> 我的意图是“一旦重试，后续请求也全部使用新的连接池”，请你重新修改

## Symptom

重试请求本身会创建新的 HTTP 客户端，但该客户端只传给本次重试。重试结束后的新分析、远程模型列表和模型连接测试仍从应用启动时的共享客户端发起，可能继续复用旧连接池。

## Reproduction

1. 启动应用，使 `AppState` 创建初始共享 HTTP 客户端。
2. 分析失败后执行 `retry_analysis`，该命令创建临时新客户端完成重试。
3. 再发起一次全新分析或模型请求，代码仍读取初始 `state.http`。

## Suspected Code Paths

- `src-tauri/src/state.rs:97` — `AppState.http` 是不可替换的 `Client` 字段。
- `src-tauri/src/commands.rs:343` — 全新分析 clone 应用启动时的共享客户端。
- `src-tauri/src/commands.rs:420` — 重试创建新客户端，但没有写回 `AppState`。
- `src-tauri/src/commands.rs:522` — 模型列表和连接测试也直接使用原共享客户端。
- `src-tauri/tests/desktop_lifecycle.rs:52` — 当前合同只约束重试创建临时客户端，没有约束替换后续请求的共享连接池。

## Root Cause Hypothesis

**Confidence: high.** 上一次修复把新 `reqwest::Client` 作为参数传给重试任务，却保留 `AppState.http: Client` 不变。由于 `Client::clone()` 共享连接池，后续新请求仍 clone 初始客户端；重试的新客户端在任务结束后释放，不能影响后续请求。

## Proposed Remediation

**Preferred**: 将 `AppState.http` 改为 `Mutex<Client>`。所有新请求在发起前短暂加锁并 clone 当前客户端；重试先创建新客户端，在确认运行可重置后将其 clone 写回共享槽，再用同一客户端执行本次重试。这样本次重试和所有后续请求共享新的连接池，而已在途请求可以安全完成。

**Files likely to change**:

- `src-tauri/src/state.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`

**Tests to add or update**:

- 约束首次分析、模型列表和连接测试都从可替换共享客户端槽取当前客户端。
- 约束重试在启动网络任务前将新客户端写回共享槽。
- 保持所有错误允许手动重试的既有测试。

## Risks & Considerations

- 不应在网络 `await` 期间持有同步互斥锁；必须先 clone 客户端再释放锁。
- 不应中断已经在途的旧请求；连接池替换只影响替换之后创建的新请求。
- 本次是同一 0.10.2 目标的纠正，不再次增加版本号。

## Open Questions

- 无。
