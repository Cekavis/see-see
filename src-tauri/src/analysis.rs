use crate::{
    error::{AppError, ErrorCode},
    history::{HistoryInput, HistoryStatus, save_history},
    providers::{ProviderEvent, ProviderRequest, stream_text},
    settings::{ModelSnapshot, PromptSnapshot},
    state::{AnalysisState, AppState},
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, ipc::Channel};
use tokio::sync::watch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AnalysisEvent {
    Started {
        #[serde(rename = "runId")]
        run_id: String,
    },
    Delta {
        #[serde(rename = "runId")]
        run_id: String,
        text: String,
    },
    Completed {
        #[serde(rename = "runId")]
        run_id: String,
        text: String,
        #[serde(rename = "savedToHistory")]
        saved_to_history: bool,
    },
    Failed {
        #[serde(rename = "runId")]
        run_id: String,
        error: AppError,
        #[serde(rename = "savedToHistory")]
        saved_to_history: bool,
    },
    Cancelled {
        #[serde(rename = "runId")]
        run_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSnapshot {
    pub run_id: String,
    pub state: AnalysisState,
    pub text: String,
    pub saved_to_history: bool,
    pub error: Option<AppError>,
}

impl AnalysisSnapshot {
    pub fn new(run_id: impl Into<String>, state: AnalysisState) -> Self {
        Self {
            run_id: run_id.into(),
            state,
            text: String::new(),
            saved_to_history: false,
            error: None,
        }
    }
}

pub struct AnalysisRun {
    snapshot: AnalysisSnapshot,
    terminal: bool,
}

impl AnalysisRun {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            snapshot: AnalysisSnapshot::new(run_id, AnalysisState::Submitting),
            terminal: false,
        }
    }

    pub fn snapshot(&self) -> AnalysisSnapshot {
        self.snapshot.clone()
    }

    pub fn started(&self) -> AnalysisEvent {
        AnalysisEvent::Started {
            run_id: self.snapshot.run_id.clone(),
        }
    }

    pub fn push_delta(&mut self, text: impl Into<String>) -> Result<AnalysisEvent, AppError> {
        self.ensure_active()?;
        let text = text.into();
        self.snapshot.state = AnalysisState::Streaming;
        self.snapshot.text.push_str(&text);
        Ok(AnalysisEvent::Delta {
            run_id: self.snapshot.run_id.clone(),
            text,
        })
    }

    pub fn complete(&mut self, saved_to_history: bool) -> Result<AnalysisEvent, AppError> {
        self.ensure_active()?;
        self.terminal = true;
        self.snapshot.state = AnalysisState::Completed;
        self.snapshot.saved_to_history = saved_to_history;
        Ok(AnalysisEvent::Completed {
            run_id: self.snapshot.run_id.clone(),
            text: self.snapshot.text.clone(),
            saved_to_history,
        })
    }

    pub fn fail(
        &mut self,
        error: AppError,
        saved_to_history: bool,
    ) -> Result<AnalysisEvent, AppError> {
        self.ensure_active()?;
        self.terminal = true;
        self.snapshot.state = AnalysisState::Failed;
        self.snapshot.saved_to_history = saved_to_history;
        self.snapshot.error = Some(error.clone());
        Ok(AnalysisEvent::Failed {
            run_id: self.snapshot.run_id.clone(),
            error,
            saved_to_history,
        })
    }

    pub fn cancel(&mut self) -> Result<AnalysisEvent, AppError> {
        self.ensure_active()?;
        self.terminal = true;
        self.snapshot.state = AnalysisState::Cancelled;
        self.snapshot.text.clear();
        Ok(AnalysisEvent::Cancelled {
            run_id: self.snapshot.run_id.clone(),
        })
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn ensure_active(&self) -> Result<(), AppError> {
        if self.terminal {
            Err(AppError::new(
                ErrorCode::AlreadyRunning,
                "分析已经结束",
                false,
                None,
            ))
        } else {
            Ok(())
        }
    }
}

pub struct ActiveAnalysis {
    run: Mutex<AnalysisRun>,
    listeners: Mutex<Vec<Channel<AnalysisEvent>>>,
    cancel: watch::Sender<bool>,
}

impl ActiveAnalysis {
    pub fn new(run_id: impl Into<String>) -> Self {
        let (cancel, _) = watch::channel(false);
        Self {
            run: Mutex::new(AnalysisRun::new(run_id)),
            listeners: Mutex::new(Vec::new()),
            cancel,
        }
    }

    pub fn snapshot(&self) -> Result<AnalysisSnapshot, AppError> {
        Ok(self.lock_run()?.snapshot())
    }

    pub fn subscribe(&self, channel: Channel<AnalysisEvent>) -> Result<AnalysisSnapshot, AppError> {
        let snapshot = self.snapshot()?;
        if !matches!(
            snapshot.state,
            AnalysisState::Completed | AnalysisState::Failed | AnalysisState::Cancelled
        ) {
            self.listeners
                .lock()
                .map_err(|_| AppError::storage("分析订阅状态不可用"))?
                .push(channel);
        }
        Ok(snapshot)
    }

    pub fn started(&self) -> Result<(), AppError> {
        let event = self.lock_run()?.started();
        self.emit(event)
    }

    pub fn push_delta(&self, text: impl Into<String>) -> Result<(), AppError> {
        let event = self.lock_run()?.push_delta(text)?;
        self.emit(event)
    }

    pub fn complete(&self, saved_to_history: bool) -> Result<(), AppError> {
        let event = self.lock_run()?.complete(saved_to_history)?;
        self.emit(event)
    }

    pub fn fail(&self, error: AppError, saved_to_history: bool) -> Result<(), AppError> {
        let event = self.lock_run()?.fail(error, saved_to_history)?;
        self.emit(event)
    }

    pub fn cancel(&self) -> Result<(), AppError> {
        let _ = self.cancel.send(true);
        if !self.lock_run()?.is_terminal() {
            let event = self.lock_run()?.cancel()?;
            self.emit(event)?;
        }
        Ok(())
    }

    pub fn cancel_receiver(&self) -> watch::Receiver<bool> {
        self.cancel.subscribe()
    }

    fn lock_run(&self) -> Result<std::sync::MutexGuard<'_, AnalysisRun>, AppError> {
        self.run
            .lock()
            .map_err(|_| AppError::storage("分析状态不可用"))
    }

    fn emit(&self, event: AnalysisEvent) -> Result<(), AppError> {
        self.listeners
            .lock()
            .map_err(|_| AppError::storage("分析订阅状态不可用"))?
            .retain(|channel| channel.send(event.clone()).is_ok());
        Ok(())
    }
}

pub struct AnalysisInput {
    pub image_png: Vec<u8>,
    pub prompt: PromptSnapshot,
    pub model: ModelSnapshot,
    pub api_key: Option<SecretString>,
    pub save_history: bool,
    pub started_at: String,
}

pub fn start_network_analysis(app: AppHandle, active: Arc<ActiveAnalysis>, input: AnalysisInput) {
    tauri::async_runtime::spawn(async move {
        let _ = active.started();
        let request = ProviderRequest {
            protocol: input.model.protocol,
            base_url: input.model.base_url.clone(),
            model_id: input.model.model_id.clone(),
            api_key: input.api_key,
            prompt: input.prompt.body.clone(),
            image_png: input.image_png.clone(),
            stream: true,
        };
        let state = app.state::<AppState>();
        let mut cancelled = active.cancel_receiver();
        let stream = stream_text(&state.http, &request, |event| {
            if let ProviderEvent::TextDelta(text) = event {
                let _ = active.push_delta(text);
            }
        });
        tokio::pin!(stream);
        let result = tokio::select! {
            _ = cancelled.changed() => None,
            result = &mut stream => Some(result),
        };
        let Some(result) = result else {
            let _ = active.cancel();
            return;
        };
        let completed_at = now();
        match result {
            Ok(text) => {
                let history = HistoryInput {
                    id: active
                        .snapshot()
                        .map(|snapshot| snapshot.run_id)
                        .unwrap_or_default(),
                    status: HistoryStatus::Success,
                    result_text: Some(text),
                    error_code: None,
                    error_message: None,
                    prompt_name: input.prompt.name,
                    prompt_body: input.prompt.body,
                    model_config_name: input.model.name,
                    protocol: input.model.protocol.as_str().into(),
                    model_id: input.model.model_id,
                    started_at: input.started_at,
                    completed_at,
                };
                let saved = save_history(
                    &state.database,
                    input.save_history,
                    &history,
                    Some(&input.image_png),
                )
                .unwrap_or(false);
                let _ = active.complete(saved);
            }
            Err(error) => {
                let history = HistoryInput {
                    id: active
                        .snapshot()
                        .map(|snapshot| snapshot.run_id)
                        .unwrap_or_default(),
                    status: HistoryStatus::Failed,
                    result_text: None,
                    error_code: Some(error.code.as_str().into()),
                    error_message: Some(error.message.clone()),
                    prompt_name: input.prompt.name,
                    prompt_body: input.prompt.body,
                    model_config_name: input.model.name,
                    protocol: input.model.protocol.as_str().into(),
                    model_id: input.model.model_id,
                    started_at: input.started_at,
                    completed_at,
                };
                let saved = save_history(
                    &state.database,
                    input.save_history,
                    &history,
                    Some(&input.image_png),
                )
                .unwrap_or(false);
                let _ = active.fail(error, saved);
            }
        }
    });
}

pub fn now() -> String {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}
