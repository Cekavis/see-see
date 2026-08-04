# Bug Assessment: 模型超时重试导致应用卡死

- **Slug**: model-timeout-retry-freeze
- **Created**: 2026-08-04
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 模型请求超时后点击重试，程序卡死，各页面无法显示；关于页显示版本 0.2.0。模型调用错误还应输出原始响应信息，并在结果弹窗中提供重试按钮。

## Symptom

Windows 上从失败历史记录再次提交时，应用可能进入无响应状态，随后设置页数据和应用版本等异步信息无法加载。模型失败结果目前只通过通知显示归一化错误，没有在结果窗口内保留可见的响应详情或直接重试入口。

## Reproduction

1. 让一次模型分析失败并保存到历史记录，例如请求超时。
2. 在历史详情点击“使用当前配置再次提交”。
3. Windows 同步 IPC 命令创建结果 WebView，应用主线程死锁。
4. 切换设置页面或打开“关于”，页面异步数据不再返回，版本停留在硬编码的 `0.2.0`。

## Suspected Code Paths

- `src/views/History.tsx:209` — 失败历史详情通过 `resubmitHistory` 触发重新提交。
- `src-tauri/src/commands.rs:544` — `resubmit_history` 是同步 Tauri 命令，并直接进入结果窗口创建流程。
- `src-tauri/src/commands.rs:start_analysis_with_image()` — 为新分析创建结果 WebView；在 Windows 同步命令上下文调用会死锁。
- `src/views/SettingsShell.tsx:43` — 关于页把 `0.2.0` 作为异步版本读取完成前的硬编码回退值。
- `src-tauri/src/providers/mod.rs:map_error_response()` — HTTP 响应体只用于分类，未随安全的错误详情返回。
- `src/views/Result.tsx` — 失败状态只发布通知，没有内联响应详情或重试按钮。

## Root Cause Hypothesis

**Confidence: high.** `resubmit_history` 与此前修复过的 `finish_capture` 走同一个 `create_result_window` 路径，但没有保持异步，因此在 Windows 同步 IPC 上下文中构建 WebView 时死锁。关于页的 `0.2.0` 是独立的陈旧硬编码回退值，在主线程死锁导致 `getVersion()` 不返回时暴露出来。结果窗口缺少重试能力，是因为活动分析只保存运行状态，没有保留重试所需的截图；HTTP 错误响应体则在后端分类后被丢弃。

## Proposed Remediation

**Preferred**:

- 将所有可能创建结果窗口的重提命令保持为异步，首先修正 `resubmit_history`，并用编译级测试防止回归。
- 让当前活动分析保留原始截图；新增同一运行窗口内的 `retry_analysis` 命令，仅允许可重试的失败状态重置并重新调用模型，避免再创建窗口。
- 在 `AppError` 增加可选详情字段。HTTP 错误携带状态码和经过敏感字段清理、长度限制后的原始响应体；无响应的网络错误携带底层请求错误文本。
- 结果窗口在失败时内联显示错误和详情，并为可重试错误提供有忙碌状态的“重试”按钮。
- 关于页从 `package.json` 读取构建版本作为回退值，移除陈旧的 `0.2.0`。

**Files likely to change**:

- `src-tauri/src/error.rs`
- `src-tauri/src/providers/mod.rs`
- `src-tauri/src/analysis.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/state.rs`
- `src-tauri/tests/analysis_flow.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src-tauri/tests/provider_contracts.rs`
- `src/ipc.ts`
- `src/App.tsx`
- `src/views/Result.tsx`
- `src/views/Result.test.tsx`
- `src/views/SettingsShell.tsx`
- `src/views/SettingsShell.test.tsx`
- synchronized version files and lockfiles

**Tests to add or update**:

- 编译级约束 `resubmit_history` 保持异步。
- 活动分析仅允许可重试的失败运行重置，并保留重试截图。
- HTTP 原始响应详情会显示状态码和正文，但敏感字段被清理且超长正文被截断。
- 结果窗口显示失败详情、重试按钮及重试忙碌状态；`started` 事件清除旧错误。
- 关于页在版本 IPC 未完成或失败时显示当前包版本，而非 `0.2.0`。

## Risks & Considerations

- 原始响应可能包含凭据或回显请求内容，必须先清理敏感 JSON 字段、常见认证标记并限制长度，不能无条件透传。
- 重试沿用当前模型和提示词配置；若用户在失败后修改了配置，重试使用最新配置，与历史记录“使用当前配置再次提交”的既有语义一致。
- 结果窗口内重试复用同一运行和订阅通道，必须在重新开始时清空旧文本、错误和历史保存状态。

## Open Questions

- 无。
