use see_see_lib::{
    error::{AppError, ErrorCode},
    providers::validate_endpoint,
};

#[test]
fn error_serialization_is_stable_and_redacted() {
    let error = AppError::new(
        ErrorCode::AuthFailed,
        "API Key 无效或无权访问该模型",
        false,
        Some("edit_model_config"),
    );

    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("auth_failed"));
    assert!(!json.contains("sk-secret"));
    assert!(!json.contains("raw_response"));
}

#[test]
fn endpoint_validation_allows_https_and_loopback_http_only() {
    assert!(validate_endpoint("https://api.example.com/v1").is_ok());
    assert!(validate_endpoint("http://localhost:11434/v1").is_ok());
    assert!(validate_endpoint("http://127.0.0.1:8080/v1").is_ok());
    assert!(validate_endpoint("http://[::1]:8080/v1").is_ok());
    assert!(validate_endpoint("http://api.example.com/v1").is_err());
    assert!(validate_endpoint("https://user:pass@example.com/v1").is_err());
    assert!(validate_endpoint("https://example.com/v1#secret").is_err());
}
