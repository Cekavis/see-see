# Bug Assessment: 模型配置操作在 IPC 边界失效

- **Slug**: model-config-actions-no-feedback
- **Created**: 2026-07-25
- **Source**: pasted text and app screenshot
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 模型配置界面的“获取模型列表”“测试连接”“保存配置”点击后仍然没有效果；上一轮反馈布局修复无效，并产生了难看的额外留白，要求撤销后从根本修复。

用户提供的真实 See See macOS 窗口截图显示三个按钮均正常渲染，但点击没有产生模型列表、连接结果、保存结果或错误消息。

## Symptom

真实 Tauri 应用中，三个模型操作按钮都会提交包含 `protocol: "openai"` 的结构化 IPC 参数，但命令无法进入 Rust 业务逻辑，界面也不显示 Tauri 参数错误。预期 OpenAI 协议值能够跨 IPC 正常反序列化，且任何 IPC 边界错误都能显示可读消息。

## Reproduction

1. 启动 macOS See See 应用并打开“模型”。
2. 保持默认协议 `OpenAI Chat Completions`。
3. 点击“获取模型列表”“测试连接”或“保存配置”。
4. 三个操作均无可见结果；已有前端组件测试仍通过，因为测试注入 `SettingsApi` mock，未经过 Tauri/Serde 边界。

## Suspected Code Paths

- `src/ipc.ts:54` — 前端协议契约是 `"openai" | "anthropic" | "gemini"`，三个按钮均通过该契约发送 `"openai"`。
- `src-tauri/src/providers/mod.rs:16` — `ProviderProtocol` 使用 `#[serde(rename_all = "snake_case")]`；`OpenAi` 因而映射为 `"open_ai"`，与前端及 IPC 合同的 `"openai"` 不一致。
- `src-tauri/src/settings.rs:103` — `ModelConfigInput` 在命令进入 `save_model_config` 前反序列化 `ProviderProtocol`。
- `src-tauri/src/commands.rs:43` — `ModelConnectionInput` 在命令进入获取列表或连接测试前反序列化同一协议枚举。
- `src/views/Settings.tsx:205` — catch 回调把未知拒绝值强制标注为 `AppError` 并读取 `.message`；Tauri 参数反序列化失败是字符串，结果被转换成 `undefined` 状态而不渲染错误。
- `src/views/Settings.model.test.tsx` — mock API 直接接收 TypeScript 对象，未覆盖协议的 Rust Serde 契约或字符串 IPC 错误。

## Root Cause Hypothesis

高置信度：三个操作共享的 `ProviderProtocol` Serde 映射错误是主因。Rust 的 `snake_case` 规则将 `OpenAi` 序列化/反序列化为 `open_ai`，而项目 IPC 合同、数据库值和 TypeScript 均使用 `openai`。Tauri 在业务命令执行前拒绝无效参数；其框架级拒绝不是项目的结构化 `AppError`，前端又假设所有错误都有 `.message`，于是拒绝原因被静默丢弃。初始 `list_model_configs` 无结构化协议入参，因此页面仍能正常加载。

## Proposed Remediation

**Preferred**: 将 `ProviderProtocol` 的 Serde 映射改为与 IPC 合同一致的 lowercase，使 `OpenAi` 接受并输出 `openai`，同时保持 `anthropic` 和 `gemini` 不变。增加 Rust 序列化往返测试锁定三个协议值。前端增加一个接受 `unknown` 的错误消息提取函数，并在模型设置 catch 路径中使用它，保证未来框架级字符串/Error/结构化错误都不会再次被静默吞掉。

不修改反馈布局或预留高度；上一轮造成空白的 CSS 和状态改动已通过提交 `0ec4e36` 完整撤销。

**Files likely to change**:

- `src-tauri/src/providers/mod.rs`
- `src-tauri/tests/foundation.rs`
- `src/ipc.ts`
- `src/ipc.test.ts`
- `src/views/Settings.tsx`
- `src/views/Settings.model.test.tsx`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- Rust 测试断言 `ProviderProtocol::OpenAi` 与 JSON `"openai"` 双向一致，另两个协议保持 lowercase。
- 前端测试断言结构化 `AppError`、JavaScript `Error` 和字符串 IPC 拒绝均产生可见消息。
- 模型设置测试用字符串拒绝模拟 Tauri 参数错误，确保不再静默。

## Risks & Considerations

- `ProviderProtocol` 的 JSON 表示从错误的 `open_ai` 修正为合同规定的 `openai`；数据库仍通过显式 `as_str()` 使用 `openai`，无需迁移。
- 错误提取必须提供安全的通用回退，不能把任意对象或供应商原始响应直接渲染到 UI。
- 本次目标沿用已发布过的修复版本 0.2.2，不再次递增版本。

## Open Questions

- 无。
