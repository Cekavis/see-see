# Provider Adapter Contract

三个协议适配器接受同一规范化输入并产生同一事件流。实现使用 `ProviderProtocol` 枚举分派，不需要动态插件、工厂注册表或单实现接口。

## Normalized request

```ts
type ProviderRequest = {
  protocol: "openai" | "anthropic" | "gemini";
  baseUrl: string;
  modelId: string;
  apiKey?: SecretString;
  prompt: string;
  imagePng: Uint8Array;
  stream: boolean;
};
```

## Normalized output

```ts
type ProviderEvent =
  | { type: "textDelta"; text: string }
  | { type: "completed" }
  | { type: "failed"; error: ProviderError };
```

- 适配器不得自动重试。
- HTTP client 禁止自动跟随重定向，避免把凭据发送到非预期主机；3xx 作为 `provider_error` 返回。
- 连接超时 10 秒、连续 60 秒无响应数据视为超时、单次请求总时限 5 分钟；首版不向用户暴露这些参数。
- 未知供应商 SSE/event 类型必须忽略并继续，除非流无法再正确解释。
- 正常结束但没有任何可见文本必须返回 `empty_response`。
- HTTP 响应体只可用于内部错误分类，不得直接进入日志或用户消息。

## Endpoint validation

1. 使用严格 URL 解析器。
2. 禁止用户名、密码和 fragment。
3. scheme 为 `https` 时允许远程主机。
4. scheme 为 `http` 时，解析后的主机必须是 `localhost`、`127.0.0.0/8` 或 `::1`；DNS 名称不能仅因解析到回环地址而绕过规则。
5. 端点路径作为 API base 保留，但适配器追加路径时必须避免双斜杠和丢失用户明确的 base path。
6. 不提供跳过 TLS 验证、自定义 CA 或任意 header 编辑。

## Common image preparation

- 捕获结果先编码为 PNG。
- 任一边超过 8,000 px 或 base64 后超过 8 MiB 时，保持纵横比缩小后重新编码。
- 缩放使用高质量滤波并优先保留文字可读性；不得转为有损 JPEG。
- 若缩小到最小可读边界后仍超限，返回 `image_too_large`，不发送请求。
- API 请求只包含一张图片和一段提示词。

## OpenAI protocol

**Preset base URL**: `https://api.openai.com/v1`

**Endpoint**: `{baseUrl}/chat/completions`

**Authentication**: `Authorization: Bearer <apiKey>` when provided.

**Request shape**:

```json
{
  "model": "<modelId>",
  "stream": true,
  "messages": [
    {
      "role": "user",
      "content": [
        { "type": "image_url", "image_url": { "url": "data:image/png;base64,..." } },
        { "type": "text", "text": "<prompt>" }
      ]
    }
  ]
}
```

**Streaming extraction**: concatenate non-empty `choices[].delta.content` strings; `[DONE]` terminates the stream.

**Model listing**: GET `{baseUrl}/models`; failure maps to `models_unavailable` and does not invalidate manual input.

**Compatibility choice**: official OpenAI supports image analysis through Chat Completions; this endpoint is selected over Responses for wider custom endpoint compatibility.

## Anthropic protocol

**Preset base URL**: `https://api.anthropic.com/v1`

**Endpoint**: `{baseUrl}/messages`

**Authentication**: `x-api-key`; send the stable required `anthropic-version` header owned by the adapter.

**Request shape**:

```json
{
  "model": "<modelId>",
  "max_tokens": 8192,
  "stream": true,
  "messages": [
    {
      "role": "user",
      "content": [
        { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": "..." } },
        { "type": "text", "text": "<prompt>" }
      ]
    }
  ]
}
```

**Streaming extraction**: accept `content_block_delta` events whose delta type is text; `message_stop` terminates the stream.

**Model listing**: use the provider Models endpoint when available; otherwise return the optional capability error.

## Gemini protocol

**Preset base URL**: `https://generativelanguage.googleapis.com/v1beta`

**Endpoint**: `{baseUrl}/models/{modelId}:streamGenerateContent?alt=sse`

**Authentication**: `x-goog-api-key` when provided.

**Request shape**:

```json
{
  "contents": [
    {
      "role": "user",
      "parts": [
        { "inlineData": { "mimeType": "image/png", "data": "..." } },
        { "text": "<prompt>" }
      ]
    }
  ]
}
```

**Streaming extraction**: concatenate text from `candidates[].content.parts[].text`; a normal stream end terminates the request.

**Model listing**: GET `{baseUrl}/models` and retain models supporting `generateContent`; allow manual model IDs regardless.

**Compatibility choice**: GenerateContent is used instead of the newer Interactions API because it directly supports inline image data and is more commonly implemented by Gemini-style custom endpoints.

## Error mapping

| Condition | Stable code | Retryable |
|---|---|---:|
| URL rejected before network | `insecure_endpoint` or `invalid_input` | no |
| DNS/connectivity failure | `network_unavailable` | yes, manual |
| TLS/certificate failure | `tls_failed` | no until configuration changes |
| timeout | `timeout` | yes, manual |
| HTTP 401/403 | `auth_failed` | no |
| HTTP 404 model/path | `model_not_found` or `provider_error` | no |
| HTTP 429 | `rate_limited` | yes, manual |
| provider says image unsupported | `image_not_supported` | no |
| HTTP 5xx | `provider_error` | yes, manual |
| malformed stream/JSON | `provider_error` | yes, manual |
| success with no text | `empty_response` | yes, manual |

`retryable` 只控制界面是否显示重试建议；应用永远不自动重试。
