use see_see_lib::{
    analysis::{ActiveAnalysis, AnalysisEvent, AnalysisRun, AnalysisSnapshot},
    error::ErrorCode,
    state::AnalysisState,
};
use std::sync::Arc;

#[test]
fn analysis_has_one_active_run_and_one_terminal_event() {
    let mut run = AnalysisRun::new("run-1");
    assert_eq!(
        run.snapshot(),
        AnalysisSnapshot::new("run-1", AnalysisState::Submitting)
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
fn cancellation_is_terminal_and_never_claims_history_persistence() {
    let mut run = AnalysisRun::new("run-2");
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
    let mut failed = AnalysisRun::new("run-3");
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

    let mut completed = AnalysisRun::new("run-4");
    completed.push_thinking("内部分析").unwrap();
    completed.push_text("仍可复制").unwrap();
    completed.complete(false).unwrap();
    assert_eq!(completed.snapshot().thinking, "内部分析");
    assert_eq!(completed.snapshot().text, "仍可复制");
    assert!(!completed.snapshot().saved_to_history);
}

#[test]
fn retry_resets_only_retryable_failures_and_keeps_the_source_image() {
    let active = Arc::new(ActiveAnalysis::new("run-5", vec![1, 2, 3]));
    active
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::Timeout, "超时", true),
            false,
        )
        .unwrap();

    active.reset_for_retry().unwrap();
    let snapshot = active.snapshot().unwrap();
    assert_eq!(snapshot.state, AnalysisState::Submitting);
    assert!(snapshot.thinking.is_empty());
    assert!(snapshot.text.is_empty());
    assert_eq!(active.image_png(), vec![1, 2, 3]);

    let terminal = ActiveAnalysis::new("run-6", vec![]);
    terminal
        .fail(
            see_see_lib::error::AppError::provider(ErrorCode::AuthFailed, "认证失败", false),
            false,
        )
        .unwrap();
    assert!(terminal.reset_for_retry().is_err());
}
