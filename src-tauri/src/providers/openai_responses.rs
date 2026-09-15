use super::{
    PreparedRequest, ProviderEvent, ProviderRequest, RemoteModel, endpoint, image_data,
    sanitize_provider_response, secret_header, token_count, usage_event,
};
use crate::error::{AppError, ErrorCode};
use reqwest::Method;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn prepare(request: &ProviderRequest) -> Result<PreparedRequest, AppError> {
    let mut headers = BTreeMap::new();
    if let Some(key) = secret_header(request) {
        headers.insert("authorization".into(), format!("Bearer {key}"));
    }
    let mut body = json!({
        "model": request.model_id,
        "stream": request.stream,
        "store": false,
        "input": [{
            "role": "user",
            "content": [
                {
                    "type": "input_image",
                    "image_url": format!("data:image/png;base64,{}", image_data(request))
                },
                {"type": "input_text", "text": request.prompt}
            ]
        }],
        "reasoning": {"summary": "auto"}
    });
    if let Some(reasoning_effort) = request.reasoning_effort {
        body["reasoning"]["effort"] = json!(reasoning_effort.as_str());
    }
    Ok(PreparedRequest {
        method: Method::POST,
        url: endpoint(&request.base_url, "responses")?,
        headers,
        body,
    })
}

pub fn parse(event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    if data.trim() == "[DONE]" {
        return Ok(vec![ProviderEvent::Completed]);
    }
    let value: Value = serde_json::from_str(data)
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "Responses 流格式无效", true))?;
    let event_type = event_name.or_else(|| value.get("type").and_then(Value::as_str));
    match event_type {
        Some("response.reasoning_summary_text.delta") => Ok(value
            .get("delta")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(|text| ProviderEvent::ThinkingDelta(text.to_owned()))
            .into_iter()
            .collect()),
        Some("response.output_text.delta") => Ok(value
            .get("delta")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(|text| ProviderEvent::TextDelta(text.to_owned()))
            .into_iter()
            .collect()),
        Some("response.completed") => {
            let usage = value
                .get("response")
                .and_then(|response| response.get("usage"))
                .or_else(|| value.get("usage"));
            let mut events = usage_event(
                usage
                    .and_then(|usage| usage.get("input_tokens"))
                    .and_then(token_count),
                usage
                    .and_then(|usage| usage.get("output_tokens"))
                    .and_then(token_count),
            )
            .into_iter()
            .collect::<Vec<_>>();
            events.push(ProviderEvent::Completed);
            Ok(events)
        }
        Some("response.failed") => Err(response_error(&value, "Responses 请求失败")),
        Some("response.incomplete") => Err(response_error(&value, "Responses 请求未完成")),
        Some(_) => Ok(Vec::new()),
        None => Err(AppError::provider(
            ErrorCode::ProviderError,
            "Responses 流事件缺少类型",
            true,
        )),
    }
}

fn response_error(value: &Value, fallback: &str) -> AppError {
    let response = value.get("response").unwrap_or(value);
    let detail = response
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .or_else(|| {
            response
                .get("incomplete_details")
                .and_then(|details| details.get("reason"))
                .and_then(Value::as_str)
        });
    let error = AppError::provider(ErrorCode::ProviderError, fallback, true);
    if let Some(detail) = detail.map(sanitize_provider_response) {
        error.with_details(detail)
    } else {
        error
    }
}

pub fn parse_models(data: &str) -> Result<Vec<RemoteModel>, AppError> {
    super::openai::parse_models(data)
}
