use secrecy::SecretString;
use see_see_lib::providers::{
    ProviderEvent, ProviderProtocol, ProviderRequest, ReasoningEffort, build_http_request,
    connection_test_png, parse_model_list, parse_stream_event, stream_text, test_connection,
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
            ProviderProtocol::OpenAiResponses => "https://api.openai.com/v1",
            ProviderProtocol::Anthropic => "https://api.anthropic.com/v1",
            ProviderProtocol::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        }
        .into(),
        model_id: "vision-model".into(),
        reasoning_effort: Some(ReasoningEffort::Low),
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
        ProviderProtocol::OpenAiResponses,
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
    let openai = build_http_request(&request(ProviderProtocol::OpenAi)).unwrap();
    assert_eq!(openai.body["stream_options"]["include_usage"], true);
    assert_eq!(openai.body["reasoning_effort"], "low");

    let responses = build_http_request(&request(ProviderProtocol::OpenAiResponses)).unwrap();
    assert_eq!(responses.url, "https://api.openai.com/v1/responses");
    assert_eq!(responses.body["store"], false);
    assert_eq!(responses.body["reasoning"]["summary"], "auto");
    assert_eq!(responses.body["reasoning"]["effort"], "low");
    assert_eq!(
        responses.body["input"][0]["content"][0]["type"],
        "input_image"
    );
    assert_eq!(
        responses.body["input"][0]["content"][1]["type"],
        "input_text"
    );
}

#[test]
fn reasoning_effort_is_supported_by_both_openai_protocols_and_ignored_elsewhere() {
    for (effort, expected) in [
        (ReasoningEffort::None, "none"),
        (ReasoningEffort::Minimal, "minimal"),
        (ReasoningEffort::Low, "low"),
        (ReasoningEffort::Medium, "medium"),
        (ReasoningEffort::High, "high"),
        (ReasoningEffort::XHigh, "xhigh"),
        (ReasoningEffort::Max, "max"),
    ] {
        let mut chat_request = request(ProviderProtocol::OpenAi);
        chat_request.reasoning_effort = Some(effort);
        let prepared = build_http_request(&chat_request).unwrap();
        assert_eq!(prepared.body["reasoning_effort"], expected);

        let mut responses_request = request(ProviderProtocol::OpenAiResponses);
        responses_request.reasoning_effort = Some(effort);
        let prepared = build_http_request(&responses_request).unwrap();
        assert_eq!(prepared.body["reasoning"]["effort"], expected);
    }

    let mut openai = request(ProviderProtocol::OpenAi);
    openai.reasoning_effort = None;
    assert!(
        build_http_request(&openai)
            .unwrap()
            .body
            .get("reasoning_effort")
            .is_none()
    );

    let mut responses = request(ProviderProtocol::OpenAiResponses);
    responses.reasoning_effort = None;
    let responses = build_http_request(&responses).unwrap();
    assert!(responses.body["reasoning"].get("effort").is_none());
    assert_eq!(responses.body["reasoning"]["summary"], "auto");

    let anthropic = build_http_request(&request(ProviderProtocol::Anthropic)).unwrap();
    assert!(anthropic.body.get("reasoning_effort").is_none());
    let gemini = build_http_request(&request(ProviderProtocol::Gemini)).unwrap();
    assert!(gemini.body.get("reasoning_effort").is_none());
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

    let responses = parse_stream_event(
        ProviderProtocol::OpenAiResponses,
        Some("response.output_text.delta"),
        r#"{"type":"response.output_text.delta","delta":"旅行"}"#,
    )
    .unwrap();
    assert_eq!(responses, vec![ProviderEvent::TextDelta("旅行".into())]);

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

    let responses = parse_stream_event(
        ProviderProtocol::OpenAiResponses,
        Some("response.reasoning_summary_text.delta"),
        r#"{"type":"response.reasoning_summary_text.delta","delta":"分析"}"#,
    )
    .unwrap();
    assert_eq!(responses, vec![ProviderEvent::ThinkingDelta("分析".into())]);

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

#[test]
fn provider_usage_events_are_normalized_separately() {
    let openai = parse_stream_event(
        ProviderProtocol::OpenAi,
        None,
        r#"{"choices":[],"usage":{"prompt_tokens":123,"completion_tokens":45}}"#,
    )
    .unwrap();
    assert_eq!(
        openai,
        vec![ProviderEvent::Usage {
            input_tokens: Some(123),
            output_tokens: Some(45),
        }]
    );

    let responses = parse_stream_event(
        ProviderProtocol::OpenAiResponses,
        Some("response.completed"),
        r#"{"type":"response.completed","response":{"usage":{"input_tokens":123,"output_tokens":45}}}"#,
    )
    .unwrap();
    assert_eq!(
        responses,
        vec![
            ProviderEvent::Usage {
                input_tokens: Some(123),
                output_tokens: Some(45),
            },
            ProviderEvent::Completed,
        ]
    );

    assert!(
        parse_stream_event(
            ProviderProtocol::OpenAiResponses,
            Some("response.failed"),
            r#"{"type":"response.failed","response":{"error":{"message":"upstream failed"}}}"#,
        )
        .is_err()
    );
    assert!(parse_stream_event(
        ProviderProtocol::OpenAiResponses,
        Some("response.incomplete"),
        r#"{"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"}}}"#,
    )
    .is_err());

    let anthropic_start = parse_stream_event(
        ProviderProtocol::Anthropic,
        Some("message_start"),
        r#"{"message":{"usage":{"input_tokens":67,"output_tokens":0}}}"#,
    )
    .unwrap();
    assert_eq!(
        anthropic_start,
        vec![ProviderEvent::Usage {
            input_tokens: Some(67),
            output_tokens: Some(0),
        }]
    );

    let anthropic_delta = parse_stream_event(
        ProviderProtocol::Anthropic,
        Some("message_delta"),
        r#"{"usage":{"output_tokens":89}}"#,
    )
    .unwrap();
    assert_eq!(
        anthropic_delta,
        vec![ProviderEvent::Usage {
            input_tokens: None,
            output_tokens: Some(89),
        }]
    );

    let gemini = parse_stream_event(
        ProviderProtocol::Gemini,
        None,
        r#"{"usageMetadata":{"promptTokenCount":11,"candidatesTokenCount":20,"thoughtsTokenCount":10}}"#,
    )
    .unwrap();
    assert_eq!(
        gemini,
        vec![ProviderEvent::Usage {
            input_tokens: Some(11),
            output_tokens: Some(30),
        }]
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
            reasoning_effort: Some(ReasoningEffort::Low),
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

#[tokio::test]
async fn responses_stream_separates_summary_answer_and_usage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "event: response.reasoning_summary_text.delta\n",
                    "data: {\"type\":\"response.reasoning_summary_text.delta\",\"delta\":\"先分析\"}\n\n",
                    "event: response.output_text.delta\n",
                    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"最终答案\"}\n\n",
                    "event: response.completed\n",
                    "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":12,\"output_tokens\":34}}}\n\n",
                )),
        )
        .mount(&server)
        .await;

    let mut events = Vec::new();
    let answer = stream_text(
        &see_see_lib::providers::client().unwrap(),
        &ProviderRequest {
            protocol: ProviderProtocol::OpenAiResponses,
            base_url: format!("{}/v1", server.uri()),
            model_id: "vision-model".into(),
            reasoning_effort: Some(ReasoningEffort::Medium),
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
            ProviderEvent::Usage {
                input_tokens: Some(12),
                output_tokens: Some(34),
            },
            ProviderEvent::Completed,
        ]
    );
}

#[tokio::test]
async fn truncated_stream_keeps_partial_text_without_inventing_usage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"部分结果\"}}]}\n\n",
                ),
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
            reasoning_effort: Some(ReasoningEffort::Low),
            api_key: None,
            prompt: "OK".into(),
            image_png: connection_test_png(),
            stream: true,
        },
        |event| events.push(event),
    )
    .await
    .unwrap();

    assert_eq!(answer, "部分结果");
    assert_eq!(events, vec![ProviderEvent::TextDelta("部分结果".into())]);
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

    let responses = parse_model_list(
        ProviderProtocol::OpenAiResponses,
        r#"{"data":[{"id":"gpt-responses"}]}"#,
    )
    .unwrap();
    assert_eq!(responses[0].id, "gpt-responses");

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
            reasoning_effort: Some(ReasoningEffort::Low),
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
            reasoning_effort: Some(ReasoningEffort::Low),
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
            reasoning_effort: Some(ReasoningEffort::Low),
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
