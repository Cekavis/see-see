use crate::{
    analysis::ActiveAnalysis,
    capture::CaptureSession,
    credentials::CredentialStore,
    database::Database,
    error::{AppError, ErrorCode},
    providers::client,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureState {
    Preparing,
    Selecting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisState {
    Submitting,
    Streaming,
    Completed,
    Failed,
    Cancelled,
}

impl AnalysisState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Default)]
pub struct RuntimeState {
    pub capture: Option<CaptureSession>,
    pub capture_reservation: Option<String>,
    pub analysis: HashMap<String, Arc<ActiveAnalysis>>,
    pub result_window_position: Option<ResultWindowPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultWindowPosition {
    pub x: i32,
    pub y: i32,
}

impl ResultWindowPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl RuntimeState {
    pub fn capture_is_active(&self) -> bool {
        self.capture.is_some() || self.capture_reservation.is_some()
    }

    pub fn reserve_capture(&mut self, id: String) -> Result<(), AppError> {
        if self.capture_is_active() {
            return Err(AppError::new(
                ErrorCode::AlreadyRunning,
                "截图正在进行",
                false,
                Some("focus_active"),
            ));
        }
        self.capture_reservation = Some(id);
        Ok(())
    }

    pub fn release_capture(&mut self, id: &str) {
        if self.capture_reservation.as_deref() == Some(id) {
            self.capture_reservation = None;
        }
    }

    pub fn take_capture(&mut self, session_id: &str) -> Result<CaptureSession, AppError> {
        if self
            .capture
            .as_ref()
            .is_none_or(|session| session.id != session_id)
        {
            return Err(AppError::new(
                ErrorCode::NotFound,
                "截图会话不存在",
                false,
                None,
            ));
        }
        Ok(self.capture.take().expect("capture checked above"))
    }

    pub fn take_analysis(&mut self, run_id: &str) -> Option<Arc<ActiveAnalysis>> {
        self.analysis.remove(run_id)
    }

    pub fn remember_result_window_position(&mut self, position: ResultWindowPosition) {
        self.result_window_position = Some(position);
    }
}

pub struct AppState {
    pub database: Database,
    pub http: Mutex<Client>,
    pub runtime: Mutex<RuntimeState>,
}

impl AppState {
    pub fn new(
        database: Database,
        credentials: Arc<dyn CredentialStore>,
    ) -> Result<Self, crate::error::AppError> {
        crate::settings::migrate_model_credentials(&database, credentials.as_ref())?;
        Ok(Self {
            database,
            http: Mutex::new(client()?),
            runtime: Mutex::new(RuntimeState::default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisState, ResultWindowPosition, RuntimeState};
    use crate::{analysis::ActiveAnalysis, capture::CaptureSession};
    use std::{collections::HashMap, sync::Arc};

    #[test]
    fn wrong_capture_id_does_not_discard_active_session() {
        let mut runtime = RuntimeState {
            capture: Some(CaptureSession {
                id: "active".into(),
                prompt_id: "prompt".into(),
                monitors: vec![],
                selection: None,
            }),
            capture_reservation: None,
            analysis: HashMap::new(),
            result_window_position: None,
        };

        assert!(runtime.take_capture("stale").is_err());
        assert_eq!(
            runtime.capture.as_ref().map(|session| session.id.as_str()),
            Some("active")
        );
        assert_eq!(runtime.take_capture("active").unwrap().id, "active");
        assert!(runtime.capture.is_none());
    }

    #[test]
    fn capture_reservation_blocks_duplicates_and_only_owner_releases_it() {
        let mut runtime = RuntimeState::default();
        runtime.reserve_capture("native".into()).unwrap();

        assert!(runtime.capture_is_active());
        assert!(runtime.reserve_capture("duplicate".into()).is_err());
        runtime.release_capture("stale");
        assert_eq!(runtime.capture_reservation.as_deref(), Some("native"));
        runtime.release_capture("native");
        assert!(!runtime.capture_is_active());
    }

    #[test]
    fn result_windows_are_tracked_and_removed_by_run_id() {
        let first = Arc::new(ActiveAnalysis::new(
            "first",
            vec![],
            "模型配置",
            "提示词配置",
        ));
        let second = Arc::new(ActiveAnalysis::new(
            "second",
            vec![],
            "另一个模型配置",
            "另一个提示词配置",
        ));
        let mut runtime = RuntimeState {
            analysis: HashMap::from([
                (String::from("first"), first.clone()),
                (String::from("second"), second.clone()),
            ]),
            ..RuntimeState::default()
        };

        assert!(runtime.take_analysis("completed-old-run").is_none());
        assert!(
            runtime
                .analysis
                .get("first")
                .is_some_and(|value| Arc::ptr_eq(value, &first))
        );
        assert!(runtime.take_analysis("first").is_some());
        assert!(
            runtime
                .analysis
                .get("second")
                .is_some_and(|value| Arc::ptr_eq(value, &second))
        );
        assert_eq!(runtime.analysis.len(), 1);
    }

    #[test]
    fn result_window_position_defaults_to_empty_and_latest_move_wins() {
        let mut runtime = RuntimeState::default();
        assert_eq!(runtime.result_window_position, None);

        runtime.remember_result_window_position(ResultWindowPosition::new(100, 200));
        assert_eq!(
            runtime.result_window_position,
            Some(ResultWindowPosition::new(100, 200))
        );

        runtime.remember_result_window_position(ResultWindowPosition::new(-40, 80));
        assert_eq!(
            runtime.result_window_position,
            Some(ResultWindowPosition::new(-40, 80))
        );
    }

    #[test]
    fn removing_a_result_window_does_not_clear_the_remembered_position() {
        let active = Arc::new(ActiveAnalysis::new(
            "position-owner",
            vec![],
            "模型配置",
            "提示词配置",
        ));
        let mut runtime = RuntimeState {
            analysis: HashMap::from([(String::from("position-owner"), active)]),
            result_window_position: Some(ResultWindowPosition::new(320, 240)),
            ..RuntimeState::default()
        };

        assert!(runtime.take_analysis("position-owner").is_some());
        assert_eq!(
            runtime.result_window_position,
            Some(ResultWindowPosition::new(320, 240))
        );
    }

    #[test]
    fn terminal_analysis_states_close_finished_result_windows() {
        assert!(!AnalysisState::Submitting.is_terminal());
        assert!(!AnalysisState::Streaming.is_terminal());
        assert!(AnalysisState::Completed.is_terminal());
        assert!(AnalysisState::Failed.is_terminal());
        assert!(AnalysisState::Cancelled.is_terminal());
    }
}
