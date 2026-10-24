use super::{
    PreparedRequest, ProviderEvent, ProviderRequest, ReasoningEffort, RemoteModel, endpoint,
    gemini, image_data, secret_header,
};
use crate::error::AppError;
use reqwest::Method;
use serde_json::json;
use std::collections::BTreeMap;

pub const BASE_URL: &str = "https://aiplatform.googleapis.com/v1";

fn service_base_url(location: &str) -> String {
    match location {
        "global" => BASE_URL.to_owned(),
        "us" | "eu" => format!("https://aiplatform.{location}.rep.googleapis.com/v1"),
        _ => format!("https://{location}-aiplatform.googleapis.com/v1"),
    }
}

fn validate_segment(value: &str, label: &str) -> Result<(), AppError> {
    let valid_chars = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !value.is_empty() && value.len() <= 128 && valid_chars {
        return Ok(());
    }
    Err(AppError::invalid(format!("Vertex AI {label} 无效")))
}

pub fn validate_project_id(project_id: &str) -> Result<(), AppError> {
    validate_segment(project_id, "项目 ID")
}

pub fn validate_location(location: &str) -> Result<(), AppError> {
    validate_segment(location, "区域")
}

fn model_major(model_id: &str) -> Option<u32> {
    let version = model_id.strip_prefix("gemini-")?.split('-').next()?;
    version.split('.').next()?.parse().ok()
}

pub fn validate_model_id(model_id: &str) -> Result<(), AppError> {
    let valid_chars = model_id.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')
    });
    if valid_chars && model_major(model_id).is_some_and(|major| major >= 3) {
        return Ok(());
    }
    Err(AppError::invalid("Vertex AI 仅支持 Gemini 3+ 模型 ID"))
}

fn thinking_level(effort: ReasoningEffort) -> Result<&'static str, AppError> {
    match effort {
        ReasoningEffort::Low => Ok("LOW"),
        ReasoningEffort::Medium => Ok("MEDIUM"),
        ReasoningEffort::High => Ok("HIGH"),
        _ => Err(AppError::invalid("Vertex AI 仅支持低、中、高思考强度")),
    }
}

pub fn prepare(request: &ProviderRequest) -> Result<PreparedRequest, AppError> {
    if request.base_url.trim_end_matches('/') != BASE_URL {
        return Err(AppError::invalid("Vertex AI 必须使用内置端点"));
    }
    let project_id = request
        .project_id
        .as_deref()
        .ok_or_else(|| AppError::invalid("Vertex AI 必须设置项目 ID"))?;
    let location = request
        .location
        .as_deref()
        .ok_or_else(|| AppError::invalid("Vertex AI 必须设置区域"))?;
    validate_project_id(project_id)?;
    validate_location(location)?;
    let model_id = request.model_id.trim();
    validate_model_id(model_id)?;
    let effort = request
        .reasoning_effort
        .ok_or_else(|| AppError::invalid("Vertex AI 必须设置思考强度"))?;
    let level = thinking_level(effort)?;

    let mut headers = BTreeMap::new();
    if let Some(key) = secret_header(request) {
        headers.insert("x-goog-api-key".into(), key);
    }
    let operation = if request.stream {
        format!(
            "projects/{project_id}/locations/{location}/publishers/google/models/{model_id}:streamGenerateContent"
        )
    } else {
        format!(
            "projects/{project_id}/locations/{location}/publishers/google/models/{model_id}:generateContent"
        )
    };
    let base_url = service_base_url(location);
    let mut url = endpoint(&base_url, &operation)?;
    if request.stream {
        url.push_str("?alt=sse");
    }
    Ok(PreparedRequest {
        method: Method::POST,
        url,
        headers,
        body: json!({
            "generationConfig": {
                "thinkingConfig": {
                    "includeThoughts": true,
                    "thinkingLevel": level,
                }
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

pub fn parse(event_name: Option<&str>, data: &str) -> Result<Vec<ProviderEvent>, AppError> {
    gemini::parse(event_name, data)
}

pub fn parse_models(_data: &str) -> Result<Vec<RemoteModel>, AppError> {
    Err(AppError::invalid(
        "Vertex AI 标准模式不支持获取模型列表，请手动填写 Gemini 3+ 模型 ID",
    ))
}
