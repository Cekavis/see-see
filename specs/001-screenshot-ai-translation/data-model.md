# Data Model: See See 多模态截图翻译

## Storage boundaries

- SQLite 保存所有非秘密持久数据、历史文本和截图。
- 操作系统凭据存储保存 API Key；SQLite 只保留 `credential_ref` 和 `has_api_key`。
- 活动截图与活动分析只存在内存中，应用退出或用户取消后销毁。
- 时间统一保存为 UTC RFC 3339 字符串，界面按本地时区显示。
- 实体主键使用 UUID 字符串，避免依赖数据库自增值穿过 IPC。
- 每个连接启用外键、`secure_delete=ON`、DELETE journal、FULL synchronous 和 `trusted_schema=OFF`；全部清空历史后执行 `VACUUM`。

## Entity: ModelConfig

代表一个用户命名的模型服务连接。

| Field | Type | Required | Rules |
|---|---|---:|---|
| `id` | UUID | yes | 创建后不可变 |
| `name` | text | yes | trim 后 1–80 字符；大小写不敏感唯一 |
| `protocol` | enum | yes | `openai`、`anthropic`、`gemini` |
| `base_url` | URL text | yes | 远程仅 HTTPS；HTTP 仅允许 localhost/回环 IP；不得包含凭据 |
| `model_id` | text | yes | trim 后 1–200 字符 |
| `credential_ref` | text | no | 内部生成；不得由前端任意指定 |
| `has_api_key` | boolean projection | yes | 从凭据读取结果生成，不作为秘密保存 |
| `test_status` | enum | yes | `untested`、`passed`、`failed` |
| `tested_at` | timestamp | no | 最近一次连接测试完成时间 |
| `test_error_code` | enum | no | 只保存脱敏稳定错误码 |
| `is_active` | boolean projection | yes | 由 `AppSettings.active_model_config_id` 派生，不单独存储 |
| `created_at` | timestamp | yes | 创建时设置 |
| `updated_at` | timestamp | yes | 每次修改更新 |

### Validation and lifecycle

- 保存 API Key 时先写凭据存储；数据库事务成功后配置才可见。数据库失败时删除刚写入的凭据。
- 修改配置但未提供新 Key 时保留原凭据；明确清除 Key 时删除凭据并设置 `has_api_key=false`。
- 删除配置时先短暂读取并删除凭据，再删除数据库记录；数据库删除失败时尝试恢复刚删除的凭据并返回错误。恢复也失败时配置保留但标记未测试并要求用户重新输入 Key，不得报告删除成功。
- 修改端点、协议、模型或凭据后必须把 `test_status` 重置为 `untested`。
- 删除当前配置后 `AppSettings.active_model_config_id` 设为 null。

## Entity: PromptPreset

代表一段命名的普通文本提示词。

| Field | Type | Required | Rules |
|---|---|---:|---|
| `id` | UUID | yes | 创建后不可变 |
| `name` | text | yes | trim 后 1–80 字符；大小写不敏感唯一 |
| `body` | text | yes | trim 后 1–20,000 字符 |
| `is_builtin` | boolean | yes | 标记初始内置预设；不限制用户编辑或删除 |
| `created_at` | timestamp | yes | 创建时设置 |
| `updated_at` | timestamp | yes | 每次修改更新 |

### Validation and lifecycle

- 首次数据库初始化插入“通用翻译为中文”和“日语学习解析”。
- 复制预设生成新 UUID 和唯一名称，不保留 `is_builtin`。
- 删除当前预设后 `AppSettings.active_prompt_id` 设为 null。

## Entity: AppSettings

单例应用偏好。

| Field | Type | Required | Rules |
|---|---|---:|---|
| `id` | integer | yes | 固定为 1 |
| `active_model_config_id` | UUID | no | 外键；删除配置时置空 |
| `active_prompt_id` | UUID | no | 外键；删除提示词时置空 |
| `capture_shortcut` | text | yes | 规范化的平台快捷键字符串 |
| `save_history` | boolean | yes | 默认 true |
| `autostart` | boolean | yes | 默认 false；数据库值须与系统注册状态一致 |
| `result_always_on_top` | boolean | yes | 默认 true |
| `updated_at` | timestamp | yes | 每次修改更新 |

## Entity: HistoryEntry

代表一次已经完成或失败的分析；取消中的请求不持久化。

| Field | Type | Required | Rules |
|---|---|---:|---|
| `id` | UUID | yes | 请求开始时生成 |
| `status` | enum | yes | `success` 或 `failed` |
| `result_text` | text | conditional | success 必须非空；failed 必须为空 |
| `error_code` | enum | conditional | failed 必须存在；success 为空 |
| `error_message` | text | conditional | 脱敏、面向用户；最长 2,000 字符 |
| `prompt_name` | text | yes | 请求开始时的名称快照 |
| `prompt_body` | text | yes | 请求开始时的正文快照 |
| `model_config_name` | text | yes | 请求开始时的配置名称快照 |
| `protocol` | enum | yes | 请求开始时的协议快照 |
| `model_id` | text | yes | 请求开始时的模型标识快照 |
| `started_at` | timestamp | yes | 请求开始时间 |
| `completed_at` | timestamp | yes | 成功或失败时间 |

### Relationships

- `HistoryEntry` 与 `HistoryImage` 为 1:0..1；能够开始网络请求的记录通常有图片，极早期本地失败可以没有。
- 历史记录不持有 `ModelConfig` 或 `PromptPreset` 外键，避免用户删除或修改配置后破坏历史快照。
- 删除 `HistoryEntry` 时级联删除 `HistoryImage`。

## Entity: HistoryImage

与历史记录分表保存，避免列表和全文检索读取大 BLOB。

| Field | Type | Required | Rules |
|---|---|---:|---|
| `history_id` | UUID | yes | 主键兼外键，删除级联 |
| `mime_type` | text | yes | 首版固定 `image/png` |
| `width` | integer | yes | 1–8,000 |
| `height` | integer | yes | 1–8,000 |
| `original_bytes` | blob | yes | 归一化 PNG，最大 8 MiB |
| `thumbnail_bytes` | blob | yes | 列表缩略图，最长边 320 px |

## History query rules

- 结果文本搜索使用参数化 `LIKE ... ESCAPE '\\'`；用户输入中的 `%`、`_` 和转义字符必须先转义。提示词使用独立的精确筛选字段。
- `started_at`、`status` 和 `prompt_name` 建立普通索引；查询固定按 `started_at DESC, id DESC` 排序并使用 cursor 分页。
- 实现必须以 10,000 条、包含长中文/日文结果的基准验证 2 秒目标；未测量到失败前不增加 FTS 或外部搜索索引。

## In-memory entity: CaptureSession

| Field | Type | Rules |
|---|---|---|
| `id` | UUID | 每次快捷键触发生成 |
| `state` | enum | `preparing`、`selecting`、`completed`、`cancelled` |
| `monitors` | list | 每项含物理矩形、缩放因子和冻结帧 |
| `selection` | rectangle | 使用虚拟桌面物理坐标；宽高必须大于 0 |

## In-memory entity: AnalysisRun

| Field | Type | Rules |
|---|---|---|
| `id` | UUID | 与成功/失败历史 ID 相同 |
| `state` | enum | `submitting`、`streaming`、`completed`、`failed`、`cancelled` |
| `image` | bytes | 归一化 PNG，仅活动期间保留 |
| `prompt_snapshot` | object | 请求开始时固定 |
| `model_snapshot` | object | 不含可序列化 API Key；Key 只在网络调用栈短暂存在 |
| `result_buffer` | text | 累积已收到增量 |
| `cancel_handle` | token | 用户取消、应用退出或连接中断时触发 |

### State transitions

```text
idle
  -> preparing_capture
  -> selecting
  -> submitting
  -> streaming
  -> completed -> idle
  -> failed    -> idle
  -> cancelled -> idle
```

- `idle` 之外再次触发快捷键不会创建第二个 run，只聚焦现有窗口。
- 只有 `completed` 和 `failed` 在 `save_history=true` 时进入数据库。
- `cancelled` 不进入数据库，并立即释放图片和结果缓冲。
- `completed` 的历史写入必须在一个 SQLite 事务中同时插入 entry 和 image；任一失败则全部回滚，结果仍留在当前浮窗供复制。
