pub mod anthropic;
pub mod gemini;
pub mod openai;

use crate::error::{AppError, ErrorCode};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use reqwest::{Client, Method, header::HeaderMap, redirect::Policy};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, io::Cursor, net::IpAddr, time::Duration};
use url::{Host, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderProtocol {
    OpenAi,
    Anthropic,
    Gemini,
}

impl TryFrom<&str> for ProviderProtocol {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "openai" => Ok(Self::OpenAi),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" => Ok(Self::Gemini),
            _ => Err(AppError::invalid("未知模型协议")),
        }
    }
}

impl ProviderProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }
}

pub struct ProviderRequest {
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub api_key: Option<SecretString>,
    pub prompt: String,
    pub image_png: Vec<u8>,
    pub stream: bool,
}

pub struct PreparedRequest {
    pub method: Method,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderEvent {
    TextDelta(String),
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub passed: bool,
    pub latency_ms: u128,
    pub error: Option<AppError>,
}

pub fn validate_endpoint(value: &str) -> Result<Url, AppError> {
    let url = Url::parse(value).map_err(|_| AppError::invalid("端点地址格式无效"))?;
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(AppError::invalid("端点不得包含凭据或片段"));
    }
    let host = url
        .host()
        .ok_or_else(|| AppError::invalid("端点缺少主机"))?;
    match url.scheme() {
        "https" => Ok(url),
        "http" if is_loopback(host) => Ok(url),
        "http" => Err(AppError::new(
            ErrorCode::InsecureEndpoint,
            "远程端点必须使用 HTTPS",
            false,
            Some("edit_model_config"),
        )),
        _ => Err(AppError::invalid("端点只支持 HTTPS 或本机 HTTP")),
    }
}

fn is_loopback(host: Host<&str>) -> bool {
    match host {
        Host::Domain(domain) => domain.eq_ignore_ascii_case("localhost"),
        Host::Ipv4(address) => IpAddr::V4(address).is_loopback(),
        Host::Ipv6(address) => IpAddr::V6(address).is_loopback(),
    }
}

pub fn build_http_request(request: &ProviderRequest) -> Result<PreparedRequest, AppError> {
    validate_endpoint(&request.base_url)?;
    match request.protocol {
        ProviderProtocol::OpenAi => openai::prepare(request),
        ProviderProtocol::Anthropic => anthropic::prepare(request),
        ProviderProtocol::Gemini => gemini::prepare(request),
    }
}

pub fn parse_stream_event(
    protocol: ProviderProtocol,
    event_name: Option<&str>,
    data: &str,
) -> Result<Vec<ProviderEvent>, AppError> {
    match protocol {
        ProviderProtocol::OpenAi => openai::parse(event_name, data),
        ProviderProtocol::Anthropic => anthropic::parse(event_name, data),
        ProviderProtocol::Gemini => gemini::parse(event_name, data),
    }
}

pub fn parse_model_list(
    protocol: ProviderProtocol,
    data: &str,
) -> Result<Vec<RemoteModel>, AppError> {
    match protocol {
        ProviderProtocol::OpenAi => openai::parse_models(data),
        ProviderProtocol::Anthropic => anthropic::parse_models(data),
        ProviderProtocol::Gemini => gemini::parse_models(data),
    }
}

pub async fn list_models(
    client: &Client,
    protocol: ProviderProtocol,
    base_url: &str,
    api_key: Option<&SecretString>,
) -> Result<Vec<RemoteModel>, AppError> {
    let request = ProviderRequest {
        protocol,
        base_url: base_url.to_owned(),
        model_id: String::new(),
        api_key: api_key.cloned(),
        prompt: String::new(),
        image_png: Vec::new(),
        stream: false,
    };
    let mut builder = client.get(endpoint(base_url, "models")?);
    match protocol {
        ProviderProtocol::OpenAi => {
            if let Some(key) = secret_header(&request) {
                builder = builder.bearer_auth(key);
            }
        }
        ProviderProtocol::Anthropic => {
            builder = builder.header("anthropic-version", "2023-06-01");
            if let Some(key) = secret_header(&request) {
                builder = builder.header("x-api-key", key);
            }
        }
        ProviderProtocol::Gemini => {
            if let Some(key) = secret_header(&request) {
                builder = builder.header("x-goog-api-key", key);
            }
        }
    }
    let response = builder.send().await.map_err(map_reqwest_error)?;
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(map_error_response(status, &body));
    }
    let data = response
        .text()
        .await
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "无法读取模型列表", true))?;
    parse_model_list(protocol, &data)
}

pub fn client() -> Result<Client, AppError> {
    Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(60))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|_| AppError::provider(ErrorCode::ProviderError, "无法创建网络客户端", false))
}

pub async fn stream_text(
    client: &Client,
    request: &ProviderRequest,
    mut on_event: impl FnMut(ProviderEvent),
) -> Result<String, AppError> {
    use eventsource_stream::Eventsource;

    let prepared = build_http_request(request)?;
    let mut builder = client
        .request(prepared.method, prepared.url)
        .json(&prepared.body);
    let mut headers = HeaderMap::new();
    for (name, value) in prepared.headers {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AppError::invalid("请求头名称无效"))?;
        let value = reqwest::header::HeaderValue::from_str(&value)
            .map_err(|_| AppError::invalid("请求头内容无效"))?;
        headers.insert(name, value);
    }
    builder = builder.headers(headers);
    let response = builder.send().await.map_err(map_reqwest_error)?;
    if response.status().is_redirection() {
        return Err(AppError::provider(
            ErrorCode::ProviderError,
            "模型端点返回了不允许的重定向",
            false,
        ));
    }
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(map_error_response(status, &body));
    }

    let mut output = String::new();
    let mut stream = response.bytes_stream().eventsource();
    while let Some(event) = stream.next().await {
        let event = event
            .map_err(|_| AppError::provider(ErrorCode::ProviderError, "模型流格式无效", true))?;
        for normalized in parse_stream_event(
            request.protocol,
            (!event.event.is_empty()).then_some(event.event.as_str()),
            &event.data,
        )? {
            if let ProviderEvent::TextDelta(delta) = &normalized {
                output.push_str(delta);
            }
            on_event(normalized);
        }
    }
    if output.trim().is_empty() {
        return Err(AppError::provider(
            ErrorCode::EmptyResponse,
            "模型没有返回文字",
            true,
        ));
    }
    Ok(output)
}

pub async fn test_connection(client: &Client, request: ProviderRequest) -> ConnectionTestResult {
    let started = std::time::Instant::now();
    match stream_text(client, &request, |_| {}).await {
        Ok(_) => ConnectionTestResult {
            passed: true,
            latency_ms: started.elapsed().as_millis(),
            error: None,
        },
        Err(error) => ConnectionTestResult {
            passed: false,
            latency_ms: started.elapsed().as_millis(),
            error: Some(error),
        },
    }
}

pub fn connection_test_png() -> Vec<u8> {
    use image::{ImageFormat, Rgba, RgbaImage};
    let image = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
    let mut encoded = Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, ImageFormat::Png)
        .expect("in-memory PNG encoding must succeed");
    encoded.into_inner()
}

fn map_reqwest_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::provider(ErrorCode::Timeout, "模型请求超时", true)
    } else if error.is_connect() {
        AppError::provider(ErrorCode::NetworkUnavailable, "无法连接模型端点", true)
    } else {
        AppError::provider(ErrorCode::ProviderError, "模型请求失败", true)
    }
}

fn map_status(status: u16) -> AppError {
    match status {
        401 | 403 => {
            AppError::provider(ErrorCode::AuthFailed, "API Key 无效或无权访问该模型", false)
        }
        404 => AppError::provider(ErrorCode::ModelNotFound, "模型或端点路径不存在", false),
        429 => AppError::provider(ErrorCode::RateLimited, "模型服务请求过多", true),
        500..=599 => AppError::provider(ErrorCode::ProviderError, "模型服务暂时不可用", true),
        _ => AppError::provider(ErrorCode::ProviderError, "模型服务拒绝了请求", false),
    }
}

fn map_error_response(status: u16, body: &str) -> AppError {
    let body = body.to_ascii_lowercase();
    if status == 400
        && body.contains("image")
        && (body.contains("support") || body.contains("vision"))
    {
        AppError::provider(
            ErrorCode::ImageNotSupported,
            "当前模型不支持图片输入",
            false,
        )
    } else {
        map_status(status)
    }
}

pub(crate) fn image_data(request: &ProviderRequest) -> String {
    STANDARD.encode(&request.image_png)
}

pub(crate) fn secret_header(request: &ProviderRequest) -> Option<String> {
    request
        .api_key
        .as_ref()
        .map(|value| value.expose_secret().to_owned())
}

pub(crate) fn endpoint(base: &str, suffix: &str) -> Result<String, AppError> {
    let mut url = validate_endpoint(base)?;
    let base_path = url.path().trim_end_matches('/');
    let suffix = suffix.trim_start_matches('/');
    url.set_path(&format!("{base_path}/{suffix}"));
    Ok(url.into())
}

pub fn normalize_png(bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    use image::{ImageFormat, ImageReader};

    let mut image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AppError::invalid("无法读取截图"))?
        .decode()
        .map_err(|_| AppError::invalid("截图格式无效"))?;
    for _ in 0..8 {
        let mut encoded = Cursor::new(Vec::new());
        image
            .clone()
            .write_to(&mut encoded, ImageFormat::Png)
            .map_err(|_| AppError::provider(ErrorCode::ImageTooLarge, "无法编码截图", false))?;
        let encoded = encoded.into_inner();
        let base64_size = encoded.len().div_ceil(3) * 4;
        if image.width() <= 8000 && image.height() <= 8000 && base64_size <= 8 * 1024 * 1024 {
            return Ok(encoded);
        }
        let dimension_ratio = (8000.0 / image.width().max(image.height()) as f64).min(1.0);
        let size_ratio = ((8 * 1024 * 1024) as f64 / base64_size as f64)
            .sqrt()
            .min(0.9);
        let ratio = dimension_ratio.min(size_ratio).max(0.5);
        let width = ((image.width() as f64 * ratio).floor() as u32).max(320);
        let height = ((image.height() as f64 * ratio).floor() as u32).max(200);
        if width >= image.width() && height >= image.height() {
            break;
        }
        image = image.resize(width, height, image::imageops::FilterType::Lanczos3);
    }
    Err(AppError::provider(
        ErrorCode::ImageTooLarge,
        "截图超过模型共同限制",
        false,
    ))
}
