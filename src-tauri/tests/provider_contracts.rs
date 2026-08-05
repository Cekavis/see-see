use secrecy::SecretString;
use see_see_lib::providers::{
    ProviderEvent, ProviderProtocol, ProviderRequest, build_http_request, connection_test_png,
    parse_model_list, parse_stream_event, stream_text, test_connection,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn request(protocol: ProviderProtocol) -> ProviderRequest {
    ProviderRequest {
        protocol,
        base_url: match protocol {
            ProviderProtocol::OpenAi => "https://api.openai.com/v1",
            ProviderProtocol::Anthropic => "https://api.anthropic.com/v1",
            ProviderProtocol::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        }
        .into(),
        model_id: "vision-model".into(),
        api_key: Some(SecretString::from("test-key")),
        prompt: "解释图片".into(),
        image_png: vec![1, 2, 3],
        stream: true,
    }
}

#[test]
fn provider_requests_match_contracts_without_exposing_keys_in_json() {
    for protocol in [
        ProviderProtocol::OpenAi,
        ProviderProtocol::Anthropic,
        ProviderProtocol::Gemini,
    ] {
        let prepared = build_http_request(&request(protocol)).unwrap();
        let body = prepared.body.to_string();
        assert!(body.contains("vision-model") || prepared.url.contains("vision-model"));
        assert!(body.contains("解释图片"));
        assert!(!body.contains("test-key"));
        assert!(
            prepared
                .headers
                .values()
                .any(|value| value.contains("test-key"))
        );
    }

    let gemini = build_http_request(&request(ProviderProtocol::Gemini)).unwrap();
    assert_eq!(
        gemini.body["generationConfig"]["thinkingConfig"]["includeThoughts"],
        true
    );
}

#[test]
fn stream_events_are_normalized_to_text_deltas() {
    let openai = parse_stream_event(
        ProviderProtocol::OpenAi,
        None,
        r#"{"choices":[{"delta":{"content":"旅行"}}]}"#,
    )
    .unwrap();
    assert_eq!(openai, vec![ProviderEvent::TextDelta("旅行".into())]);

    let anthropic = parse_stream_event(
        ProviderProtocol::Anthropic,
        Some("content_block_delta"),
        r#"{"delta":{"type":"text_delta","text":"旅行"}}"#,
    )
    .unwrap();
    assert_eq!(anthropic, vec![ProviderEvent::TextDelta("旅行".into())]);

    let gemini = parse_stream_event(
        ProviderProtocol::Gemini,
        None,
        r#"{"candidates":[{"content":{"parts":[{"text":"旅行"}]}}]}"#,
    )
    .unwrap();
    assert_eq!(gemini, vec![ProviderEvent::TextDelta("旅行".into())]);
}

#[test]
fn provider_thinking_events_are_normalized_separately() {
    let openai = parse_stream_event(
        ProviderProtocol::OpenAi,
        None,
        r#"{"choices":[{"delta":{"reasoning_content":"先分析","reasoning_details":[{"text":"再确认"}],"content":"答案"}}]}"#,
    )
    .unwrap();
    assert_eq!(
        openai,
        vec![
            ProviderEvent::ThinkingDelta("先分析".into()),
            ProviderEvent::ThinkingDelta("再确认".into()),
            ProviderEvent::TextDelta("答案".into()),
        ]
    );

    let anthropic = parse_stream_event(
        ProviderProtocol::Anthropic,
        Some("content_block_delta"),
        r#"{"delta":{"type":"thinking_delta","thinking":"分析"}}"#,
    )
    .unwrap();
    assert_eq!(anthropic, vec![ProviderEvent::ThinkingDelta("分析".into())]);

    let gemini = parse_stream_event(
        ProviderProtocol::Gemini,
        None,
        r#"{"candidates":[{"content":{"parts":[{"thought":true,"text":"分析"},{"text":"答案"}]}}]}"#,
    )
    .unwrap();
    assert_eq!(
        gemini,
        vec![
            ProviderEvent::ThinkingDelta("分析".into()),
            ProviderEvent::TextDelta("答案".into()),
        ]
    );
}

#[tokio::test]
async fn leading_think_tags_are_split_across_stream_chunks() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "data: {\"choices\":[{\"delta\":{\"content\":\" \\n<thi\"}}]}\n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"nk>先分析</th\"}}]}\n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"ink>\"}}]}\n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"\\r\\n\\n最终答案\"}}]}\n\n",
                    "data: [DONE]\n\n",
                )),
        )
        .mount(&server)
        .await;

    let mut events = Vec::new();
    let answer = stream_text(
        &see_see_lib::providers::client().unwrap(),
        &ProviderRequest {
            protocol: ProviderProtocol::OpenAi,
            base_url: format!("{}/v1", server.uri()),
            model_id: "vision-model".into(),
            api_key: None,
            prompt: "OK".into(),
            image_png: connection_test_png(),
            stream: true,
        },
        |event| events.push(event),
    )
    .await
    .unwrap();

    assert_eq!(answer, "最终答案");
    assert_eq!(
        events,
        vec![
            ProviderEvent::ThinkingDelta("先分析".into()),
            ProviderEvent::TextDelta("最终答案".into()),
            ProviderEvent::Completed,
        ]
    );
}

#[test]
fn provider_completion_and_empty_output_are_distinct() {
    assert_eq!(
        parse_stream_event(ProviderProtocol::OpenAi, None, "[DONE]").unwrap(),
        vec![ProviderEvent::Completed]
    );
    assert!(parse_stream_event(ProviderProtocol::OpenAi, None, "not-json").is_err());
}

#[test]
fn model_lists_are_normalized() {
    let openai = parse_model_list(
        ProviderProtocol::OpenAi,
        r#"{"data":[{"id":"gpt-vision"}]}"#,
    )
    .unwrap();
    assert_eq!(openai[0].id, "gpt-vision");

    let anthropic = parse_model_list(
        ProviderProtocol::Anthropic,
        r#"{"data":[{"id":"claude-vision","display_name":"Claude Vision"}]}"#,
    )
    .unwrap();
    assert_eq!(anthropic[0].name, "Claude Vision");

    let gemini = parse_model_list(
        ProviderProtocol::Gemini,
        r#"{"models":[{"name":"models/gemini-vision","displayName":"Gemini Vision","supportedGenerationMethods":["generateContent"]},{"name":"models/embed","supportedGenerationMethods":["embedContent"]}]}"#,
    )
    .unwrap();
    assert_eq!(gemini.len(), 1);
    assert_eq!(gemini[0].id, "gemini-vision");
}

#[tokio::test]
async fn connection_errors_are_classified_without_automatic_retry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let result = test_connection(
        &see_see_lib::providers::client().unwrap(),
        ProviderRequest {
            protocol: ProviderProtocol::OpenAi,
            base_url: format!("{}/v1", server.uri()),
            model_id: "vision-model".into(),
            api_key: Some(SecretString::from("wrong")),
            prompt: "OK".into(),
            image_png: connection_test_png(),
            stream: true,
        },
    )
    .await;
    assert_eq!(result.error.unwrap().code.as_str(), "auth_failed");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn image_capability_errors_have_a_stable_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(400).set_body_string("This model does not support image inputs"),
        )
        .mount(&server)
        .await;
    let result = test_connection(
        &see_see_lib::providers::client().unwrap(),
        ProviderRequest {
            protocol: ProviderProtocol::OpenAi,
            base_url: format!("{}/v1", server.uri()),
            model_id: "text-only".into(),
            api_key: None,
            prompt: "OK".into(),
            image_png: connection_test_png(),
            stream: true,
        },
    )
    .await;
    let error = result.error.unwrap();
    assert_eq!(error.code.as_str(), "image_not_supported");
    let details = error.details.unwrap();
    assert!(details.contains("HTTP 400"));
    assert!(details.contains("This model does not support image inputs"));
}

#[tokio::test]
async fn provider_response_details_are_bounded_and_redact_sensitive_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "message": "upstream failed",
                "api_key": "sk-secret",
                "echo": "credential sk-leaked",
                "context": "x".repeat(5_000)
            }
        })))
        .mount(&server)
        .await;

    let error = test_connection(
        &see_see_lib::providers::client().unwrap(),
        ProviderRequest {
            protocol: ProviderProtocol::OpenAi,
            base_url: format!("{}/v1", server.uri()),
            model_id: "vision-model".into(),
            api_key: None,
            prompt: "OK".into(),
            image_png: connection_test_png(),
            stream: true,
        },
    )
    .await
    .error
    .unwrap();

    let details = error.details.unwrap();
    assert!(details.contains("HTTP 500"));
    assert!(details.contains("upstream failed"));
    assert!(details.contains("[REDACTED]"));
    assert!(!details.contains("sk-secret"));
    assert!(!details.contains("sk-leaked"));
    assert!(details.contains("已截断"));
}
