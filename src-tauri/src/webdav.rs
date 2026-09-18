use crate::{
    error::{AppError, ErrorCode},
    providers::validate_endpoint,
    settings::{WebdavSettings, WebdavSettingsInput},
};
use futures_util::StreamExt;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use url::Url;

pub const REMOTE_FILENAME: &str = "see-see-config.json";
pub const MAX_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub fn normalize_settings_input(input: &mut WebdavSettingsInput) -> Result<(), AppError> {
    input.url = normalize_endpoint(&input.url)?;
    input.username = input.username.trim().to_owned();
    if input.username.len() > 1_024
        || input.username.contains(':')
        || input.username.chars().any(char::is_control)
    {
        return Err(AppError::invalid("WebDAV 用户名格式无效"));
    }
    input.remote_root = normalize_remote_root(&input.remote_root)?;
    if input.clear_password
        && input
            .password
            .as_deref()
            .is_some_and(|password| !password.is_empty())
    {
        return Err(AppError::invalid("清除 WebDAV 密码时不能同时填写新密码"));
    }
    Ok(())
}

pub fn normalize_endpoint(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::invalid("WebDAV 地址不能为空"));
    }
    if value.len() > 2_000 || value.contains('\\') || value.chars().any(char::is_control) {
        return Err(AppError::invalid("WebDAV 地址格式无效"));
    }
    let parsed = validate_endpoint(value)?;
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(AppError::invalid("WebDAV 地址不能包含查询参数或片段"));
    }
    if parsed.path().contains('\\') {
        return Err(AppError::invalid("WebDAV 地址路径格式无效"));
    }
    Ok(parsed.as_str().trim_end_matches('/').to_owned())
}

pub fn normalize_remote_root(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok("see-see".to_owned());
    }
    if trimmed.len() > 200
        || trimmed.contains('\\')
        || trimmed.contains('?')
        || trimmed.contains('#')
        || trimmed.contains('%')
        || trimmed.contains(':')
    {
        return Err(AppError::invalid("WebDAV 远程根目录格式无效"));
    }
    let normalized = trimmed.trim_matches('/');
    if normalized.is_empty() {
        return Ok("see-see".to_owned());
    }
    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments.len() > 16
        || segments.iter().any(|segment| {
            segment.is_empty()
                || *segment == "."
                || *segment == ".."
                || segment.chars().any(char::is_control)
        })
    {
        return Err(AppError::invalid("WebDAV 远程根目录格式无效"));
    }
    Ok(segments.join("/"))
}

pub async fn upload(
    client: &Client,
    settings: &WebdavSettings,
    password: Option<&SecretString>,
    payload: &[u8],
) -> Result<(), AppError> {
    let (endpoint, root_segments) = connection_parts(settings)?;
    if payload.len() > MAX_SNAPSHOT_BYTES {
        return Err(AppError::invalid("WebDAV 配置文件过大"));
    }
    ensure_remote_root(client, &endpoint, &root_segments, settings, password).await?;
    let file_url = append_segments(
        &endpoint,
        root_segments
            .iter()
            .map(String::as_str)
            .chain([REMOTE_FILENAME]),
    );
    let request = with_auth(client.put(file_url), settings, password)?
        .timeout(REQUEST_TIMEOUT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload.to_vec());
    let response = request.send().await.map_err(map_transport_error)?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), "上传 WebDAV 配置失败"));
    }
    Ok(())
}

pub async fn download(
    client: &Client,
    settings: &WebdavSettings,
    password: Option<&SecretString>,
) -> Result<Vec<u8>, AppError> {
    let (endpoint, root_segments) = connection_parts(settings)?;
    let file_url = append_segments(
        &endpoint,
        root_segments
            .iter()
            .map(String::as_str)
            .chain([REMOTE_FILENAME]),
    );
    let response = with_auth(client.get(file_url), settings, password)?
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(map_transport_error)?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), "下载 WebDAV 配置失败"));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_SNAPSHOT_BYTES as u64)
    {
        return Err(AppError::invalid("WebDAV 配置文件过大"));
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(map_transport_error)?;
        if bytes.len().saturating_add(chunk.len()) > MAX_SNAPSHOT_BYTES {
            return Err(AppError::invalid("WebDAV 配置文件过大"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn connection_parts(settings: &WebdavSettings) -> Result<(Url, Vec<String>), AppError> {
    let endpoint = normalize_endpoint(&settings.url)?;
    let endpoint = Url::parse(&endpoint).map_err(|_| AppError::invalid("WebDAV 地址格式无效"))?;
    let root = normalize_remote_root(&settings.remote_root)?;
    Ok((endpoint, root.split('/').map(str::to_owned).collect()))
}

fn append_segments<'a>(endpoint: &Url, segments: impl IntoIterator<Item = &'a str>) -> Url {
    let mut url = endpoint.clone();
    {
        let mut path = url
            .path_segments_mut()
            .expect("HTTP(S) URLs always support path segments");
        path.pop_if_empty();
        for segment in segments {
            path.push(segment);
        }
    }
    url
}

async fn ensure_remote_root(
    client: &Client,
    endpoint: &Url,
    root_segments: &[String],
    settings: &WebdavSettings,
    password: Option<&SecretString>,
) -> Result<(), AppError> {
    for end in 1..=root_segments.len() {
        let segments = root_segments[..end].iter().map(String::as_str);
        let url = append_segments(endpoint, segments.chain([""]));
        let request = with_auth(
            client.request(Method::from_bytes(b"MKCOL").expect("MKCOL"), url),
            settings,
            password,
        )?
        .timeout(REQUEST_TIMEOUT);
        let response = request.send().await.map_err(map_transport_error)?;
        let status = response.status();
        if status.is_success() || status == StatusCode::METHOD_NOT_ALLOWED {
            continue;
        }
        return Err(map_status(status, "无法创建 WebDAV 远程目录"));
    }
    Ok(())
}

fn with_auth(
    request: RequestBuilder,
    settings: &WebdavSettings,
    password: Option<&SecretString>,
) -> Result<RequestBuilder, AppError> {
    if settings.username.trim().is_empty() {
        return Ok(request);
    }
    let password = password.ok_or_else(|| AppError::invalid("请先保存 WebDAV 密码"))?;
    Ok(request.basic_auth(&settings.username, Some(password.expose_secret())))
}

fn map_status(status: StatusCode, operation: &str) -> AppError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AppError::new(
            ErrorCode::AuthFailed,
            "WebDAV 认证失败",
            false,
            Some("edit"),
        ),
        StatusCode::NOT_FOUND => AppError::new(
            ErrorCode::NotFound,
            format!("{operation}：远程文件或目录不存在"),
            true,
            Some("retry"),
        ),
        StatusCode::TOO_MANY_REQUESTS => AppError::new(
            ErrorCode::RateLimited,
            "WebDAV 服务请求过于频繁",
            true,
            Some("retry"),
        ),
        status if status.is_redirection() => {
            AppError::invalid("WebDAV 地址发生重定向，请填写最终的 WebDAV 地址")
        }
        status if status.is_server_error() => AppError::new(
            ErrorCode::NetworkUnavailable,
            format!("{operation}（HTTP {}）", status.as_u16()),
            true,
            Some("retry"),
        ),
        _ => AppError::new(
            ErrorCode::StorageFailed,
            format!("{operation}（HTTP {}）", status.as_u16()),
            false,
            Some("retry"),
        ),
    }
}

fn map_transport_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::new(ErrorCode::Timeout, "WebDAV 请求超时", true, Some("retry"))
    } else if error.is_connect() {
        AppError::new(
            ErrorCode::NetworkUnavailable,
            "无法连接 WebDAV 服务",
            true,
            Some("retry"),
        )
    } else {
        AppError::new(
            ErrorCode::NetworkUnavailable,
            "WebDAV 网络请求失败",
            true,
            Some("retry"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_root_uses_application_default() {
        assert_eq!(normalize_remote_root(" ").unwrap(), "see-see");
        assert_eq!(normalize_remote_root("/foo/bar/").unwrap(), "foo/bar");
    }

    #[test]
    fn traversal_and_ambiguous_roots_are_rejected() {
        for root in [".", "..", "foo/../bar", "foo\\bar", "foo?bar", "foo#bar"] {
            assert!(normalize_remote_root(root).is_err(), "accepted {root}");
        }
    }

    #[test]
    fn webdav_endpoint_requires_safe_url() {
        assert!(normalize_endpoint("https://example.test/dav").is_ok());
        assert!(normalize_endpoint("http://localhost:8080/dav").is_ok());
        assert!(normalize_endpoint("http://example.test/dav").is_err());
        assert!(normalize_endpoint("https://example.test/dav?x=1").is_err());
    }
}
