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
    pub analysis: Option<Arc<ActiveAnalysis>>,
}

impl RuntimeState {
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
}

pub struct AppState {
    pub database: Database,
    pub credentials: Arc<dyn CredentialStore>,
    pub http: Client,
    pub runtime: Mutex<RuntimeState>,
}

impl AppState {
    pub fn new(
        database: Database,
        credentials: Arc<dyn CredentialStore>,
    ) -> Result<Self, crate::error::AppError> {
        Ok(Self {
            database,
            credentials,
            http: client()?,
            runtime: Mutex::new(RuntimeState::default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeState;
    use crate::capture::CaptureSession;

    #[test]
    fn wrong_capture_id_does_not_discard_active_session() {
        let mut runtime = RuntimeState {
            capture: Some(CaptureSession {
                id: "active".into(),
                monitors: vec![],
                selection: None,
            }),
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
}
