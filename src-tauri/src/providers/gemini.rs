use super::{
    PreparedRequest, ProviderEvent, ProviderRequest, endpoint, image_data, secret_header,
    token_count, usage_event,
};
use crate::error::{AppError, ErrorCode};
use reqwest::Method;
use serde_json::json;
use std::collections::BTreeMap;

use super::RemoteModel;

pub fn prepare(request: &ProviderRequest) -> Result<PreparedRequest, AppError> {
    let mut headers = BTreeMap::new();
    if let Some(key) = secret_header(request) {
        headers.insert("x-goog-api-key".into(), key);
    }
    let operation = if request.stream {
        format!("models/{}:streamGenerateContent", request.model_id)
    } else {
        format!("models/{}:generateContent", request.model_id)
    };
    let mut url = endpoint(&request.base_url, &operation)?;
    if request.stream {
        url.push_str("?alt=sse");
    }
    Ok(PreparedRequest {
        method: Method::POST,
        url,
        headers,
        body: json!({
            "generationConfig": {
                "thinkingConfig": {"includeThoughts": true}
            },
            "contents": [{
                "role": "user",
                "parts": [
                    {"inlineData": {"mimeType": "image/png", "data": image_data(request)}},
                    {"text": request.prompt}
                ]
            }]
        }),
    })
}

pub fn parse(_event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "Gemini 流格式无效", true))?;
    let mut events = value["candidates"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|candidate| {
            candidate["content"]["parts"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .filter(|part| part["text"].as_str().is_some_and(|text| !text.is_empty()))
        .map(|part| {
            let text = part["text"].as_str().unwrap().to_owned();
            if part["thought"].as_bool() == Some(true) {
                ProviderEvent::ThinkingDelta(text)
            } else {
                ProviderEvent::TextDelta(text)
            }
        })
        .collect::<Vec<_>>();
    let usage = &value["usageMetadata"];
    let candidates_tokens = usage.get("candidatesTokenCount").and_then(token_count);
    let thoughts_tokens = usage.get("thoughtsTokenCount").and_then(token_count);
    let output_tokens = match (candidates_tokens, thoughts_tokens) {
        (Some(candidates), Some(thoughts)) => candidates.checked_add(thoughts),
        (Some(candidates), None) => Some(candidates),
        (None, Some(thoughts)) => Some(thoughts),
        (None, None) => None,
    };
    if let Some(event) = usage_event(
        usage.get("promptTokenCount").and_then(token_count),
        output_tokens,
    ) {
        events.push(event);
    }
    Ok(events)
}

pub fn parse_models(data: &str) -> Result<Vec<RemoteModel>, AppError> {
    let value: serde_json::Value = serde_json::from_str(data).map_err(|_| {
        AppError::provider(ErrorCode::ProviderError, "Gemini 模型列表格式无效", true)
    })?;
    Ok(value["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|model| {
            model["supportedGenerationMethods"]
                .as_array()
                .is_some_and(|methods| {
                    methods
                        .iter()
                        .any(|method| method.as_str() == Some("generateContent"))
                })
        })
        .filter_map(|model| {
            let full_id = model["name"].as_str()?;
            let id = full_id.strip_prefix("models/").unwrap_or(full_id);
            Some(RemoteModel {
                id: id.into(),
                name: model["displayName"].as_str().unwrap_or(id).into(),
            })
        })
        .collect())
}
