use see_see_lib::{
    analysis::{AnalysisEvent, AnalysisRun, AnalysisSnapshot},
    error::ErrorCode,
    state::AnalysisState,
};

#[test]
fn analysis_has_one_active_run_and_one_terminal_event() {
    let mut run = AnalysisRun::new("run-1");
    assert_eq!(
        run.snapshot(),
        AnalysisSnapshot::new("run-1", AnalysisState::Submitting)
    );
    assert!(matches!(
        run.push_delta("你"),
        Ok(AnalysisEvent::Delta { .. })
    ));
    assert!(matches!(
        run.push_delta("好"),
        Ok(AnalysisEvent::Delta { .. })
    ));
    let completed = run.complete(false).unwrap();
    assert_eq!(
        completed,
        AnalysisEvent::Completed {
            run_id: "run-1".into(),
            text: "你好".into(),
            saved_to_history: false
        }
    );
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
    completed.push_delta("仍可复制").unwrap();
    completed.complete(false).unwrap();
    assert_eq!(completed.snapshot().text, "仍可复制");
    assert!(!completed.snapshot().saved_to_history);
}
