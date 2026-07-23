# IPC Contract

所有窗口只通过此契约访问 Rust。前端不得直接访问数据库、凭据、远程网络或任意文件路径。字段使用 `camelCase`；所有命令返回 `Result<T, AppError>`。

## Stable error envelope

```json
{
  "code": "auth_failed",
  "message": "API Key 无效或无权访问该模型",
  "retryable": false,
  "action": "edit_model_config"
}
```

允许的稳定 `code`：

- `invalid_input`
- `shortcut_conflict`
- `screen_permission_denied`
- `capture_failed`
- `no_active_model`
- `no_active_prompt`
- `insecure_endpoint`
- `tls_failed`
- `network_unavailable`
- `timeout`
- `auth_failed`
- `rate_limited`
- `model_not_found`
- `image_not_supported`
- `image_too_large`
- `provider_error`
- `empty_response`
- `storage_full`
- `storage_failed`
- `clipboard_failed`
- `already_running`
- `not_found`

`message` 不得包含 API Key、完整端点响应体、截图内容或完整模型输出。

## App and onboarding

### `get_app_snapshot() -> AppSnapshot`

返回当前设置、是否有活动模型/提示词、屏幕权限状态、当前运行状态和当前窗口可用操作。不得返回 API Key。

### `complete_onboarding() -> void`

仅当截图权限、活动模型连接测试和活动提示词都有效时成功。

### `open_view(view: "history" | "prompts" | "settings") -> void`

创建或聚焦唯一窗口实例。

### `quit_app() -> void`

取消活动请求、关闭窗口并退出后台进程。

## Capture and analysis

### `begin_capture() -> CaptureSessionSummary`

- 前置：没有活动 capture 或 analysis；存在活动模型和提示词。
- 行为：捕获所有显示器冻结帧、创建 overlay 窗口并返回 monitor 元数据。
- 失败：权限、截图或并发错误。

### `get_capture_frame(sessionId, monitorId) -> binary image/png`

只允许当前 session 的 monitor；使用二进制 IPC，不返回文件路径或 base64 JSON。

### `update_capture_selection(sessionId, selection) -> void`

`selection` 是虚拟桌面物理像素矩形；Rust 验证 session 和边界，并广播到其他 overlay。

### `finish_capture(sessionId, selection) -> AnalysisStarted`

- 关闭 overlays，裁剪并拼接 PNG。
- 固定当前模型配置和提示词快照。
- 创建唯一活动分析并创建或聚焦结果窗口；overlay 不持有流式 Channel。

### `cancel_capture(sessionId) -> void`

关闭 overlays，释放冻结帧，不发送请求、不写历史。

### `attach_analysis(runId, onEvent: Channel<AnalysisEvent>) -> AnalysisSnapshot`

结果窗口挂载、重载或晚打开时调用。命令先返回当前累计内容和终止状态，再把后续事件发送到该窗口的 Channel；已结束的 run 不再发送事件。

### `cancel_analysis(runId) -> void`

触发取消 token，关闭网络流；取消 run 不写历史。

### `close_result(runId) -> void`

生成中关闭结果窗口等同于取消；终止状态下关闭只释放内存中的截图、结果缓冲和 Channel。已经写入历史的数据不受影响。

### `set_result_always_on_top(value) -> void`

更新结果窗口和持久设置。

### `copy_text(text) -> void`

只写系统剪贴板；失败返回 `clipboard_failed`。

## Analysis event channel

```ts
type AnalysisEvent =
  | { type: "started"; runId: string }
  | { type: "delta"; runId: string; text: string }
  | { type: "completed"; runId: string; text: string; savedToHistory: boolean }
  | { type: "failed"; runId: string; error: AppError; savedToHistory: boolean }
  | { type: "cancelled"; runId: string };
```

- `delta.text` 只包含新增可见文本。
- `completed.text` 是最终完整文本，必须等于所有 delta 拼接或非流式完整结果。
- 每个 run 只允许一个终止事件。

## Model configurations

### `list_model_configs() -> ModelConfigSummary[]`

返回 `hasApiKey`，绝不返回 Key 或 `credentialRef`。

### `save_model_config(input: ModelConfigInput) -> ModelConfigSummary`

```ts
type ModelConfigInput = {
  id?: string;
  name: string;
  protocol: "openai" | "anthropic" | "gemini";
  baseUrl: string;
  modelId: string;
  apiKey?: string;
  clearApiKey?: boolean;
};
```

- `apiKey` 是只写字段；IPC 完成后前端立即清空输入状态。
- `apiKey` 与 `clearApiKey` 不得同时出现。

### `delete_model_config(id) -> void`

删除当前配置时同时清除 active 选择；删除凭据失败不得报告成功。

### `set_active_model_config(id) -> void`

配置必须存在且最近一次测试状态为 `passed`。

### `list_remote_models(draft: ModelConnectionInput) -> RemoteModel[]`

失败不使草稿无效；界面继续允许手动输入。

### `test_model_config(draft: ModelConnectionInput) -> ConnectionTestResult`

发送极小图片请求，验证图片输入和文本输出。结果包括 `passed`、延迟和稳定错误；不得返回供应商原始响应体。

## Prompt presets

- `list_prompt_presets() -> PromptPreset[]`
- `save_prompt_preset(input) -> PromptPreset`
- `duplicate_prompt_preset(id) -> PromptPreset`
- `delete_prompt_preset(id) -> void`
- `set_active_prompt(id) -> void`

名称和正文验证遵循 data-model.md。删除当前提示词后 active 选择变为 null。

## History

### `query_history(query: HistoryQuery) -> HistoryPage`

```ts
type HistoryQuery = {
  text?: string;
  promptName?: string;
  status?: "success" | "failed";
  cursor?: string;
  limit?: number;
};
```

- `limit` 默认 50、最大 100。
- 按 `startedAt DESC, id DESC` 稳定排序。
- cursor 是不透明字符串；无匹配返回空列表和明确空态，不返回错误。

### `get_history_entry(id) -> HistoryEntryDetail`

返回完整结果和快照元数据，不返回图片 BLOB。

### `get_history_image(id, variant: "thumbnail" | "original") -> binary image/png`

缺失图片返回 `not_found`；不得返回本地数据库路径。

### `resubmit_history(id) -> AnalysisStarted`

读取原图，但使用当前活动模型配置和当前提示词；创建新 run 和新历史 ID，并创建或聚焦结果窗口，由结果窗口调用 `attach_analysis`。

### `delete_history_entry(id) -> void`

单事务删除 entry 和 image 行。

### `clear_history() -> { deletedCount: number }`

要求前端先完成确认；后端在单事务中删除全部历史数据，随后执行 `VACUUM` 释放空闲页面。

## Settings and diagnostics

- `get_settings() -> AppSettings`
- `set_capture_shortcut(shortcut) -> AppSettings`
- `set_save_history(value) -> AppSettings`
- `set_autostart(value) -> AppSettings`
- `export_sanitized_logs() -> ExportResult`

导出日志命令由后端打开系统保存对话框，前端不得传入任意文件路径。快捷键更新必须先注册新组合，成功后才注销旧组合并提交设置，避免失败后失去可用快捷键。开机启动的数据库值只有在系统注册成功后才能更新。
