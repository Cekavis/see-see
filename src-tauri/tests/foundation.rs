use see_see_lib::{
    commands::ModelConnectionInput,
    error::{AppError, ErrorCode},
    providers::{ProviderProtocol, validate_endpoint},
    settings::ModelConfigInput,
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
fn provider_protocol_json_matches_the_ipc_contract() {
    for (protocol, json) in [
        (ProviderProtocol::OpenAi, "\"openai\""),
        (ProviderProtocol::Anthropic, "\"anthropic\""),
        (ProviderProtocol::Gemini, "\"gemini\""),
    ] {
        assert_eq!(serde_json::to_string(&protocol).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<ProviderProtocol>(json).unwrap(),
            protocol,
        );
    }

    assert!(serde_json::from_str::<ProviderProtocol>("\"open_ai\"").is_err());

    assert!(
        serde_json::from_value::<ModelConnectionInput>(serde_json::json!({
            "protocol": "openai",
            "baseUrl": "https://api.openai.com/v1",
            "modelId": "gpt-vision"
        }))
        .is_ok(),
    );
    assert!(
        serde_json::from_value::<ModelConfigInput>(serde_json::json!({
            "name": "OpenAI",
            "protocol": "openai",
            "baseUrl": "https://api.openai.com/v1",
            "modelId": "gpt-vision"
        }))
        .is_ok(),
    );
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
