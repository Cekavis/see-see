use secrecy::{ExposeSecret, SecretString};
use see_see_lib::{
    analysis::{ActiveAnalysis, AnalysisEvent, AnalysisInput, AnalysisRun, AnalysisSnapshot},
    error::ErrorCode,
    providers::ProviderProtocol,
    settings::{ModelSnapshot, PromptSnapshot},
    state::AnalysisState,
};
use std::sync::Arc;

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
    let completed = run.complete(false).unwrap();
    assert_eq!(
        completed,
        AnalysisEvent::Completed {
            run_id: "run-1".into(),
            thinking: "先判断".into(),
            text: "你好".into(),
            saved_to_history: false
        }
    );
    assert_eq!(run.snapshot().thinking, "先判断");
    assert_eq!(run.snapshot().text, "你好");
    assert_eq!(run.snapshot().state, AnalysisState::Completed);
    assert_eq!(run.cancel().unwrap_err().code, ErrorCode::AlreadyRunning);
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
    assert_eq!(
        run.cancel().unwrap(),
        AnalysisEvent::Cancelled {
            run_id: "run-2".into()
        }
    );
    let snapshot = run.snapshot();
    assert_eq!(snapshot.state, AnalysisState::Cancelled);
    assert!(!snapshot.saved_to_history);
    assert!(
        run.fail(see_see_lib::error::AppError::invalid("late"), true)
            .is_err()
    );
}

#[test]
fn failed_requests_are_not_retried_and_storage_failure_keeps_result_available() {
    let mut failed = AnalysisRun::new("run-3", "模型配置", "提示词配置");
    let event = failed
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
            false,
        )
        .unwrap();
    assert!(matches!(
        event,
        AnalysisEvent::Failed {
            saved_to_history: false,
            ..
        }
    ));

    let mut completed = AnalysisRun::new("run-4", "模型配置", "提示词配置");
    completed.push_thinking("内部分析").unwrap();
    completed.push_text("仍可复制").unwrap();
    completed.complete(false).unwrap();
    assert_eq!(completed.snapshot().thinking, "内部分析");
    assert_eq!(completed.snapshot().text, "仍可复制");
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
