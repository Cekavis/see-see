# Model Configuration Contract

## IPC data shapes

### ModelConfigInput

```text
id?: string
name: string
protocol: "openai" | "anthropic" | "gemini"
baseUrl: string
modelId: string
apiKey?: string
clearApiKey?: boolean
```

An omitted `apiKey` preserves the existing key during edit and creates a keyless new configuration. `clearApiKey` explicitly removes an existing key. A submitted key is preserved exactly.

### ModelConfigSummary

```text
id: string
name: string
protocol: "openai" | "anthropic" | "gemini"
baseUrl: string
modelId: string
hasApiKey: boolean
isActive: boolean
```

The summary intentionally excludes the key value and any test status, test time, or test error.

### ModelConnectionInput

```text
id?: string
protocol: "openai" | "anthropic" | "gemini"
baseUrl: string
modelId: string
apiKey?: string
```

The optional saved ID may resolve an existing same-row key when the edit field contains no replacement. It does not cause testing or listing to mutate persistence.

## IPC operations

| Operation | Input | Output | Persistent effects |
|-----------|-------|--------|--------------------|
| `list_model_configs` | none | summaries | None |
| `save_model_config` | input | saved summary | Insert/update exactly one configuration |
| `duplicate_model_config` | saved ID | copied summary | Insert one inactive configuration with a unique name |
| `delete_model_config` | saved ID | none | Delete configuration and its same-row key |
| `set_active_model_config` | saved ID | none | Select any existing configuration; no test prerequisite |
| `list_remote_models` | connection draft | remote models | None |
| `test_model_config` | connection draft | transient result | None |

## Model page UI contract

- Initial/loading-complete state shows the page title, “新增” action, and configuration list or empty state; it does not render form fields.
- “新增” opens a default draft; “编辑” opens the selected saved values including the key.
- “取消” closes the editor without a write.
- Successful save closes the editor and refreshes the list; failed save keeps it open.
- Testing and model listing operate on the open draft, keep the editor open, and publish transient feedback.
- Copying operates directly from a saved card, refreshes the list, and does not open the editor.
- Busy draft operations disable conflicting add, edit, copy, activate, delete, save, cancel, list, and test actions until completion.
- Cards may communicate that a key is configured but must not render the key value.

## Disclosure contract

- Plain-text saved keys are allowed only in the local configuration database and provider request construction. New/replacement keys pass through the masked configuration input and IPC but saved keys are never returned to the webview.
- Keys are forbidden in logs, exported diagnostics, notifications, provider error messages, history, card text, and accessibility labels.
