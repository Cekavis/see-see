use secrecy::{ExposeSecret, SecretString};
use see_see_lib::{
    analysis::{
        ActiveAnalysis, AnalysisEvent, AnalysisInput, AnalysisRun, AnalysisSnapshot,
        PerformanceMetrics,
    },
    error::ErrorCode,
    providers::{ProviderProtocol, ReasoningEffort, StreamTiming},
    settings::{ModelSnapshot, PromptSnapshot},
    state::AnalysisState,
};
use std::sync::Arc;
use tauri::ipc::Channel;

#[test]
fn analysis_run_has_one_terminal_event() {
    let mut run = AnalysisRun::new("run-1", "模型配置", "提示词配置");
    assert_eq!(
        run.snapshot(),
        AnalysisSnapshot::new("run-1", AnalysisState::Submitting, "模型配置", "提示词配置")
    );
    assert_eq!(
        run.started(),
        AnalysisEvent::Started {
            run_id: "run-1".into(),
            model_config_name: "模型配置".into(),
            prompt_config_name: "提示词配置".into(),
        }
    );
    assert!(matches!(
        run.push_thinking("先判断"),
        Ok(AnalysisEvent::ThinkingDelta { .. })
    ));
    assert!(matches!(
        run.push_text("你"),
        Ok(AnalysisEvent::Delta { .. })
    ));
    assert!(matches!(
        run.push_text("好"),
        Ok(AnalysisEvent::Delta { .. })
    ));
    assert_eq!(
        run.push_usage(Some(123), None).unwrap(),
        AnalysisEvent::Usage {
            run_id: "run-1".into(),
            input_tokens: Some(123),
            output_tokens: None,
        }
    );
    assert_eq!(
        run.push_usage(None, Some(45)).unwrap(),
        AnalysisEvent::Usage {
            run_id: "run-1".into(),
            input_tokens: Some(123),
            output_tokens: Some(45),
        }
    );
    let completed = run.complete(false).unwrap();
    assert_eq!(
        completed,
        AnalysisEvent::Completed {
            run_id: "run-1".into(),
            thinking: "先判断".into(),
            text: "你好".into(),
            input_tokens: Some(123),
            output_tokens: Some(45),
            saved_to_history: false,
            metrics: PerformanceMetrics::default(),
        }
    );
    assert_eq!(run.snapshot().thinking, "先判断");
    assert_eq!(run.snapshot().text, "你好");
    assert_eq!(run.snapshot().state, AnalysisState::Completed);
    assert_eq!(run.cancel().unwrap_err().code, ErrorCode::AlreadyRunning);
}

#[test]
fn performance_metrics_accept_late_usage_without_estimating_missing_or_zero_duration() {
    let mut run = AnalysisRun::new("run-metrics", "模型配置", "提示词配置");
    run.push_timing(StreamTiming::FirstToken { ttft_ms: 17 })
        .unwrap();
    run.push_timing(StreamTiming::Completed {
        generation_ms: Some(250),
    })
    .unwrap();
    assert_eq!(run.snapshot().metrics.tps, None);

    run.push_usage(None, Some(50)).unwrap();
    assert_eq!(run.snapshot().metrics.tps, Some(200.0));
    let completed = run.complete(false).unwrap();
    assert!(matches!(
        completed,
        AnalysisEvent::Completed {
            metrics: PerformanceMetrics {
                ttft_ms: Some(17),
                generation_ms: Some(250),
                tps: Some(value),
            },
            ..
        } if (value - 200.0).abs() < f64::EPSILON
    ));

    let mut missing_usage = AnalysisRun::new("run-missing-usage", "模型配置", "提示词配置");
    missing_usage
        .push_timing(StreamTiming::Completed {
            generation_ms: Some(250),
        })
        .unwrap();
    missing_usage.complete(false).unwrap();
    assert_eq!(missing_usage.snapshot().metrics.tps, None);

    let mut zero_duration = AnalysisRun::new("run-zero-duration", "模型配置", "提示词配置");
    zero_duration.push_usage(None, Some(50)).unwrap();
    zero_duration
        .push_timing(StreamTiming::Completed {
            generation_ms: Some(0),
        })
        .unwrap();
    zero_duration.complete(false).unwrap();
    assert_eq!(zero_duration.snapshot().metrics.tps, None);
}

#[test]
fn failed_analysis_preserves_ttft_and_retry_clears_all_metrics() {
    let active = ActiveAnalysis::new("run-failed-metrics", vec![], "模型配置", "提示词配置");
    active
        .push_timing(StreamTiming::FirstToken { ttft_ms: 23 })
        .unwrap();
    active
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
            false,
        )
        .unwrap();
    assert_eq!(
        active.snapshot().unwrap().metrics,
        PerformanceMetrics {
            ttft_ms: Some(23),
            generation_ms: None,
            tps: None,
        }
    );

    active
        .reset_for_retry("重试模型配置", "重试提示词配置")
        .unwrap();
    assert_eq!(
        active.snapshot().unwrap().metrics,
        PerformanceMetrics::default()
    );
}

#[test]
fn late_subscriber_receives_the_terminal_snapshot_with_metrics() {
    let active = ActiveAnalysis::new("run-late-subscribe", vec![], "模型配置", "提示词配置");
    active
        .push_timing(StreamTiming::FirstToken { ttft_ms: 11 })
        .unwrap();
    active.push_usage(None, Some(8)).unwrap();
    active
        .push_timing(StreamTiming::Completed {
            generation_ms: Some(400),
        })
        .unwrap();
    active.complete(false).unwrap();

    let channel = Channel::<AnalysisEvent>::new(|_| Ok(()));
    let snapshot = active.subscribe(channel).unwrap();
    assert_eq!(snapshot.state, AnalysisState::Completed);
    assert_eq!(snapshot.metrics.ttft_ms, Some(11));
    assert_eq!(snapshot.metrics.generation_ms, Some(400));
    assert_eq!(snapshot.metrics.tps, Some(20.0));
}

#[test]
fn concurrent_analyses_keep_run_ids_and_streams_independent() {
    let first = ActiveAnalysis::new("run-first", vec![], "模型一", "提示词一");
    let second = ActiveAnalysis::new("run-second", vec![], "模型二", "提示词二");

    first.started().unwrap();
    second.started().unwrap();
    first.push_text("第一路").unwrap();
    second.push_text("第二路").unwrap();
    assert_eq!(first.snapshot().unwrap().text, "第一路");
    assert_eq!(second.snapshot().unwrap().text, "第二路");
    assert_eq!(first.snapshot().unwrap().run_id, "run-first");
    assert_eq!(second.snapshot().unwrap().run_id, "run-second");
}

#[test]
fn cancellation_is_terminal_and_never_claims_history_persistence() {
    let mut run = AnalysisRun::new("run-2", "模型配置", "提示词配置");
    run.push_timing(StreamTiming::FirstToken { ttft_ms: 19 })
        .unwrap();
    assert_eq!(
        run.cancel().unwrap(),
        AnalysisEvent::Cancelled {
            run_id: "run-2".into()
        }
    );
    let snapshot = run.snapshot();
    assert_eq!(snapshot.state, AnalysisState::Cancelled);
    assert!(!snapshot.saved_to_history);
    assert_eq!(snapshot.metrics, PerformanceMetrics::default());
    assert!(
        run.fail(see_see_lib::error::AppError::invalid("late"), true)
            .is_err()
    );
}

#[test]
fn failed_requests_are_not_retried_and_storage_failure_keeps_result_available() {
    let mut failed = AnalysisRun::new("run-3", "模型配置", "提示词配置");
    failed.push_usage(Some(18), None).unwrap();
    let event = failed
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
            false,
        )
        .unwrap();
    assert!(matches!(
        event,
        AnalysisEvent::Failed {
            input_tokens: Some(18),
            output_tokens: None,
            saved_to_history: false,
            ..
        }
    ));

    let mut completed = AnalysisRun::new("run-4", "模型配置", "提示词配置");
    completed.push_thinking("内部分析").unwrap();
    completed.push_text("仍可复制").unwrap();
    completed.push_usage(Some(18), None).unwrap();
    completed.complete(false).unwrap();
    assert_eq!(completed.snapshot().thinking, "内部分析");
    assert_eq!(completed.snapshot().text, "仍可复制");
    assert_eq!(completed.snapshot().input_tokens, Some(18));
    assert_eq!(completed.snapshot().output_tokens, None);
    assert!(!completed.snapshot().saved_to_history);
}

#[test]
fn retry_resets_all_failures_and_keeps_the_source_image() {
    let active = Arc::new(ActiveAnalysis::new(
        "run-5",
        vec![1, 2, 3],
        "原模型配置",
        "原提示词配置",
    ));
    active
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::AuthFailed, "认证失败", false),
            false,
        )
        .unwrap();

    active
        .reset_for_retry("重试模型配置", "重试提示词配置")
        .unwrap();
    let snapshot = active.snapshot().unwrap();
    assert_eq!(snapshot.state, AnalysisState::Submitting);
    assert_eq!(snapshot.model_config_name, "重试模型配置");
    assert_eq!(snapshot.prompt_config_name, "重试提示词配置");
    assert!(snapshot.thinking.is_empty());
    assert!(snapshot.text.is_empty());
    assert_eq!(snapshot.input_tokens, None);
    assert_eq!(snapshot.output_tokens, None);
    assert_eq!(active.image_png(), vec![1, 2, 3]);

    let terminal = ActiveAnalysis::new("run-6", vec![], "模型配置", "提示词配置");
    terminal.complete(false).unwrap();
    assert!(
        terminal
            .reset_for_retry("重试模型配置", "重试提示词配置")
            .is_err()
    );
}

#[test]
fn retry_uses_the_original_request_snapshot_after_configuration_changes() {
    let input = AnalysisInput {
        image_png: vec![9, 8, 7],
        prompt: PromptSnapshot {
            id: "prompt-original".into(),
            name: "原提示词".into(),
            body: "请按原提示词回答".into(),
        },
        model: ModelSnapshot {
            id: "model-original".into(),
            name: "原模型".into(),
            protocol: ProviderProtocol::Anthropic,
            base_url: "https://original.example/v1".into(),
            model_id: "vision-original".into(),
            reasoning_effort: None,
        },
        api_key: Some(SecretString::from("original-secret")),
        save_history: true,
        started_at: "2026-08-31T00:00:00Z".into(),
    };
    let active = Arc::new(ActiveAnalysis::new_with_input("run-snapshot", &input));
    active
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
            true,
        )
        .unwrap();

    let retry = active.retry_input().unwrap();
    assert_eq!(retry.image_png, input.image_png);
    assert_eq!(retry.prompt, input.prompt);
    assert_eq!(retry.model, input.model);
    assert_eq!(
        retry.api_key.as_ref().unwrap().expose_secret(),
        "original-secret"
    );
    assert_eq!(retry.save_history, input.save_history);
    assert_ne!(retry.started_at, input.started_at);

    active
        .reset_for_retry(retry.model.name.clone(), retry.prompt.name.clone())
        .unwrap();
    assert_eq!(active.snapshot().unwrap().run_id, "run-snapshot");
    assert_eq!(active.snapshot().unwrap().model_config_name, "原模型");
    assert_eq!(active.snapshot().unwrap().prompt_config_name, "原提示词");
}

fn model_retry_source_input(save_history: bool) -> AnalysisInput {
    AnalysisInput {
        image_png: vec![9, 8, 7],
        prompt: PromptSnapshot {
            id: "prompt-original".into(),
            name: "原提示词".into(),
            body: "请按原提示词回答".into(),
        },
        model: ModelSnapshot {
            id: "model-original".into(),
            name: "原模型".into(),
            protocol: ProviderProtocol::Anthropic,
            base_url: "https://original.example/v1".into(),
            model_id: "vision-original".into(),
            reasoning_effort: None,
        },
        api_key: Some(SecretString::from("source-test-key")),
        save_history,
        started_at: "2026-08-31T00:00:00Z".into(),
    }
}

fn selected_retry_model() -> ModelSnapshot {
    ModelSnapshot {
        id: "model-selected".into(),
        name: "所选模型".into(),
        protocol: ProviderProtocol::OpenAiResponses,
        base_url: "https://selected.example/v1".into(),
        model_id: "vision-selected".into(),
        reasoning_effort: Some(ReasoningEffort::High),
    }
}

#[test]
fn model_retry_preserves_frozen_input_and_substitutes_only_model_credentials_and_time() {
    for save_history in [false, true] {
        let mut input = model_retry_source_input(save_history);
        let original_prompt = input.prompt.clone();
        let active = ActiveAnalysis::new_with_input("run-source", &input);
        active
            .fail(
                see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
                save_history,
            )
            .unwrap();
        let source_snapshot = active.snapshot().unwrap();
        input.prompt.body = "已经修改的提示词".into();

        let selected = selected_retry_model();
        let retry = active
            .retry_input_with_model(
                selected.clone(),
                Some(SecretString::from("selected-test-key")),
            )
            .unwrap();

        assert_eq!(retry.image_png, input.image_png);
        assert_eq!(retry.prompt, original_prompt);
        assert_eq!(retry.model, selected);
        assert_eq!(
            retry.api_key.as_ref().unwrap().expose_secret(),
            "selected-test-key"
        );
        assert_eq!(retry.save_history, save_history);
        assert_ne!(retry.started_at, input.started_at);
        assert_eq!(active.snapshot().unwrap(), source_snapshot);

        let source_request = active.retry_input().unwrap();
        assert_eq!(source_request.model, input.model);
        assert_eq!(source_request.prompt, original_prompt);
        assert_eq!(
            source_request.api_key.as_ref().unwrap().expose_secret(),
            "source-test-key"
        );
    }
}

#[test]
fn model_retry_does_not_reuse_source_credentials_when_selected_model_has_no_key() {
    let active = ActiveAnalysis::new_with_input("run-source", &model_retry_source_input(false));
    let retry = active
        .retry_input_with_model(selected_retry_model(), None)
        .unwrap();

    assert!(retry.api_key.is_none());
}

#[test]
fn model_retry_accepts_every_source_state_and_keeps_new_run_and_cancellation_independent() {
    for state in [
        AnalysisState::Submitting,
        AnalysisState::Streaming,
        AnalysisState::Completed,
        AnalysisState::Failed,
        AnalysisState::Cancelled,
    ] {
        let active = ActiveAnalysis::new_with_input("run-source", &model_retry_source_input(false));
        let source_cancellation = active.cancel_receiver();
        match state {
            AnalysisState::Submitting => {}
            AnalysisState::Streaming => {
                active.push_thinking("原分析思考").unwrap();
                active.push_text("原分析内容").unwrap();
                active.push_usage(Some(10), Some(4)).unwrap();
            }
            AnalysisState::Completed => {
                active.push_text("原分析内容").unwrap();
                active.complete(false).unwrap();
            }
            AnalysisState::Failed => active
                .fail(
                    see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
                    false,
                )
                .unwrap(),
            AnalysisState::Cancelled => active.cancel().unwrap(),
        }
        let source_snapshot = active.snapshot().unwrap();
        let was_cancelled = *source_cancellation.borrow();
        let input = active
            .retry_input_with_model(selected_retry_model(), None)
            .unwrap();
        let retried = ActiveAnalysis::new_with_input("run-selected-model", &input);
        let retried_cancellation = retried.cancel_receiver();

        assert_eq!(source_snapshot.state, state);
        assert_ne!(retried.snapshot().unwrap().run_id, source_snapshot.run_id);
        assert_eq!(retried.snapshot().unwrap().state, AnalysisState::Submitting);
        assert!(!*retried_cancellation.borrow());
        retried.push_text("新分析内容").unwrap();
        retried.cancel().unwrap();

        assert_eq!(active.snapshot().unwrap(), source_snapshot);
        assert_eq!(*source_cancellation.borrow(), was_cancelled);
        assert!(*retried_cancellation.borrow());
        if !state.is_terminal() {
            active.push_text("继续原分析").unwrap();
            assert!(active.snapshot().unwrap().text.ends_with("继续原分析"));
        }
    }
}

#[test]
fn model_retry_rejects_missing_original_request_without_mutating_source() {
    let active = ActiveAnalysis::new("run-source", vec![1, 2, 3], "原模型", "原提示词");
    let snapshot = active.snapshot().unwrap();
    let error = active
        .retry_input_with_model(selected_retry_model(), None)
        .err()
        .unwrap();

    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert_eq!(error.message, "原始分析配置不可用");
    assert_eq!(active.snapshot().unwrap(), snapshot);
}
