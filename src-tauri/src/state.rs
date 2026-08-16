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
use std::sync::{Arc, Mutex};

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
    pub analysis: Option<Arc<ActiveAnalysis>>,
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
        let is_current = self.analysis.as_ref().is_some_and(|active| {
            active
                .snapshot()
                .is_ok_and(|snapshot| snapshot.run_id == run_id)
        });
        if is_current {
            self.analysis.take()
        } else {
            None
        }
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
    use super::{AnalysisState, RuntimeState};
    use crate::{analysis::ActiveAnalysis, capture::CaptureSession};
    use std::sync::Arc;

    #[test]
    fn wrong_capture_id_does_not_discard_active_session() {
        let mut runtime = RuntimeState {
            capture: Some(CaptureSession {
                id: "active".into(),
                monitors: vec![],
                selection: None,
            }),
            capture_reservation: None,
            analysis: None,
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
    fn stale_result_window_cannot_take_current_analysis() {
        let active = Arc::new(ActiveAnalysis::new(
            "current",
            vec![],
            "模型配置",
            "提示词配置",
        ));
        let mut runtime = RuntimeState {
            analysis: Some(active.clone()),
            ..RuntimeState::default()
        };

        assert!(runtime.take_analysis("completed-old-run").is_none());
        assert!(
            runtime
                .analysis
                .as_ref()
                .is_some_and(|value| Arc::ptr_eq(value, &active))
        );
        assert!(runtime.take_analysis("current").is_some());
        assert!(runtime.analysis.is_none());
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
