# Data Model: WebDAV 配置同步

## Local WebDAV settings

Persisted in the singleton `app_settings` row:

| Field                | Type | Rules                                                                                          |
| -------------------- | ---- | ---------------------------------------------------------------------------------------------- |
| `webdav_url`         | text | Empty until configured; HTTPS required except loopback HTTP; no query or fragment              |
| `webdav_username`    | text | Trimmed; may be empty for anonymous servers                                                    |
| `webdav_remote_root` | text | Relative path; defaults to `see-see`; no `.` or `..` segments, backslashes, query, or fragment |

The password is stored through the platform credential store under a fixed application key. The database stores no password and IPC responses expose only `hasPassword`.

## Remote snapshot

```json
{
  "version": 1,
  "activeModelConfigId": "stable-model-id",
  "models": [
    {
      "id": "stable-model-id",
      "name": "Vision model",
      "protocol": "openai",
      "baseUrl": "https://example.test/v1",
      "modelId": "vision-model",
      "reasoningEffort": "low",
      "apiKey": null
    }
  ],
  "prompts": [
    {
      "id": "stable-prompt-id",
      "name": "翻译",
      "body": "..."
    }
  ]
}
```

The snapshot deliberately has no `captureShortcut` or WebDAV login fields. Model `apiKey` is required and can be a string or `null`; null clears the matched model's local Key. Unknown fields may be ignored, but unknown versions and missing required fields are rejected. API Keys are never returned to the frontend by the sync commands or logged.

## Merge identity and invariants

- Model and prompt rows are matched by `id` first, then by case-insensitive `name`.
- Names remain unique under the existing database constraints.
- Existing prompt `capture_shortcut` is never changed by import.
- New prompts receive `NULL` for `capture_shortcut`.
- All rows in one downloaded snapshot are validated before any row is changed.
- Local-only model and prompt rows remain untouched.

## Result payload

Upload and download return counts for models and prompts on success. Failures use the existing `AppError` contract.
