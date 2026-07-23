use super::{PreparedRequest, ProviderEvent, ProviderRequest, endpoint, image_data, secret_header};
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
    Ok(PreparedRequest {
        method: Method::POST,
        url: endpoint(&request.base_url, "chat/completions")?,
        headers,
        body: json!({
            "model": request.model_id,
            "stream": request.stream,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_data(request))}},
                    {"type": "text", "text": request.prompt}
                ]
            }]
        }),
    })
}

pub fn parse(_event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    if data.trim() == "[DONE]" {
        return Ok(vec![ProviderEvent::Completed]);
    }
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "OpenAI 流格式无效", true))?;
    Ok(value["choices"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|choice| choice["delta"]["content"].as_str())
        .filter(|text| !text.is_empty())
        .map(|text| ProviderEvent::TextDelta(text.to_owned()))
        .collect())
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
