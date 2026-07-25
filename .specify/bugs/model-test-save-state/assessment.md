# Bug Assessment: 模型测试后保存状态丢失

- **Slug**: model-test-save-state
- **Created**: 2026-07-25
- **Source**: pasted text and application screenshot
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

用户输入完整模型信息并确认连接测试通过，随后保存配置，但配置卡片仍显示“无 Key · 未测试”，主页仍显示“尚未配置可用模型”。用户怀疑 API Key 的保存或读取存在问题。

## Symptom

未保存的新模型可以使用表单中的 API Key 完成连接测试，但测试成功不会随之后的保存操作保留，配置也不会自动成为当前模型。用户期望已通过连接测试的完整配置在保存后显示已保存 Key、测试通过，并可供主页使用。

## Reproduction

1. 打开模型设置，输入配置名称、协议、端点、模型 ID 和 API Key。
2. 点击“测试连接”，等待连接成功。
3. 点击“保存配置”。
4. 查看已保存配置和主页状态：配置被创建为未测试，且没有当前可用模型。

## Suspected Code Paths

- `src/views/Settings.tsx:71` — 临时连接草稿只携带已有的 `form.id`；新配置测试时没有 ID。
- `src/views/Settings.tsx:211` — “测试连接”直接测试表单草稿，测试前不保存配置或 Key。
- `src/views/Settings.tsx:237` — 保存与测试是彼此独立的操作，保存成功后也不把返回的配置 ID 写回表单。
- `src-tauri/src/commands.rs:389` — 后端仅在连接草稿带 ID 时记录测试结果，新配置测试结果不会持久化。
- `src-tauri/src/settings.rs:581` — 新配置保存时按设计初始化为 `untested`。
- `src-tauri/src/settings.rs:553` — API Key 在保存请求携带 `api_key` 时写入系统凭据存储；引用保存和读取路径一致。

## Root Cause Hypothesis

置信度：高。问题是前端工作流没有把“测试临时草稿”和“保存模型配置”组成一个一致操作。新配置测试时没有持久化 ID，所以后端无法记录测试结果；后续保存会创建一个全新的 `untested` 配置，而且没有设置当前模型。API Key 的系统凭据读写实现本身路径一致，并有 `src-tauri/tests/model_config.rs` 覆盖；让测试先保存完整表单可确保 Key 和稳定 ID 在连接测试前已经落入对应存储。

## Proposed Remediation

**Preferred**: 将“测试连接”改成原子化的用户工作流：先调用 `saveModelConfig` 保存当前完整表单和 API Key；用返回的配置 ID 与规范化连接字段调用 `testModelConfig`，使后端将结果记录到该配置；测试通过后调用 `setActiveModelConfig`，使主页立即识别可用模型。将保存返回的 ID 写回表单并清空 Key 输入，确保用户随后再次点击保存时保留已存 Key 和已通过状态。

测试失败时保留已保存配置及失败状态，但不设为当前模型。所有异步模型操作共享 busy 状态，避免保存、测试和列表请求交错。

**Files likely to change**:

- `src/views/Settings.tsx`
- `src/views/Settings.model.test.tsx`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 新配置测试时先持久化 API Key，再使用返回 ID 测试。
- 测试通过后自动设为当前模型，并保留保存后的 ID/Key 状态。
- 测试失败时不设为当前模型。

## Risks & Considerations

- “测试连接”现在会保存配置，即使远端测试失败；界面反馈需要明确这一点。
- 只有测试通过才可设为当前模型，继续由后端约束保护。
- API Key 不应出现在测试调用或序列化摘要中；测试应通过已保存 ID 从系统凭据存储读取。
- 这是行为修复，需同步提升补丁版本。

## Open Questions

- 无。
