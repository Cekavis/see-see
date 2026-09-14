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
        headers.insert("authorization".into(), format!("Bearer {key}"));
    }
    let mut body = json!({
        "model": request.model_id,
        "stream": request.stream,
        "messages": [{
            "role": "user",
            "content": [
                {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_data(request))}},
                {"type": "text", "text": request.prompt}
            ]
        }]
    });
    if let Some(reasoning_effort) = request.reasoning_effort {
        body["reasoning_effort"] = json!(reasoning_effort.as_str());
    }
    if request.stream {
        body["stream_options"] = json!({"include_usage": true});
    }
    Ok(PreparedRequest {
        method: Method::POST,
        url: endpoint(&request.base_url, "chat/completions")?,
        headers,
        body,
    })
}

pub fn parse(_event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    if data.trim() == "[DONE]" {
        return Ok(vec![ProviderEvent::Completed]);
    }
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "OpenAI 流格式无效", true))?;
    let mut events = Vec::new();
    for choice in value["choices"].as_array().into_iter().flatten() {
        let delta = &choice["delta"];
        for key in ["reasoning_content", "reasoning"] {
            if let Some(text) = delta[key].as_str().filter(|text| !text.is_empty()) {
                events.push(ProviderEvent::ThinkingDelta(text.to_owned()));
            }
        }
        for detail in delta["reasoning_details"].as_array().into_iter().flatten() {
            if let Some(text) = detail["text"].as_str().filter(|text| !text.is_empty()) {
                events.push(ProviderEvent::ThinkingDelta(text.to_owned()));
            }
        }
        if let Some(text) = delta["content"].as_str().filter(|text| !text.is_empty()) {
            events.push(ProviderEvent::TextDelta(text.to_owned()));
        }
    }
    if let Some(event) = usage_event(
        value
            .get("usage")
            .and_then(|usage| usage.get("prompt_tokens"))
            .and_then(token_count),
        value
            .get("usage")
            .and_then(|usage| usage.get("completion_tokens"))
            .and_then(token_count),
    ) {
        events.push(event);
    }
    Ok(events)
}

pub fn parse_models(data: &str) -> Result<Vec<RemoteModel>, AppError> {
    let value: serde_json::Value = serde_json::from_str(data).map_err(|_| {
        AppError::provider(ErrorCode::ProviderError, "OpenAI 模型列表格式无效", true)
    })?;
    Ok(value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|model| model["id"].as_str())
        .map(|id| RemoteModel {
            id: id.into(),
            name: id.into(),
        })
        .collect())
}
