use super::{PreparedRequest, ProviderEvent, ProviderRequest, endpoint, image_data, secret_header};
use crate::error::{AppError, ErrorCode};
use reqwest::Method;
use serde_json::json;
use std::collections::BTreeMap;

use super::RemoteModel;

pub fn prepare(request: &ProviderRequest) -> Result<PreparedRequest, AppError> {
    let mut headers = BTreeMap::from([("anthropic-version".into(), "2023-06-01".into())]);
    if let Some(key) = secret_header(request) {
        headers.insert("x-api-key".into(), key);
    }
    Ok(PreparedRequest {
        method: Method::POST,
        url: endpoint(&request.base_url, "messages")?,
        headers,
        body: json!({
            "model": request.model_id,
            "max_tokens": 8192,
            "stream": request.stream,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": image_data(request)}},
                    {"type": "text", "text": request.prompt}
                ]
            }]
        }),
    })
}

pub fn parse(event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    if event_name == Some("message_stop") {
        return Ok(vec![ProviderEvent::Completed]);
    }
    if event_name != Some("content_block_delta") {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "Anthropic 流格式无效", true))?;
    Ok(value["delta"]["text"]
        .as_str()
        .filter(|text| !text.is_empty())
        .map(|text| vec![ProviderEvent::TextDelta(text.to_owned())])
        .unwrap_or_default())
}

pub fn parse_models(data: &str) -> Result<Vec<RemoteModel>, AppError> {
    let value: serde_json::Value = serde_json::from_str(data).map_err(|_| {
        AppError::provider(ErrorCode::ProviderError, "Anthropic 模型列表格式无效", true)
    })?;
    Ok(value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let id = model["id"].as_str()?;
            Some(RemoteModel {
                id: id.into(),
                name: model["display_name"].as_str().unwrap_or(id).into(),
            })
        })
        .collect())
}
