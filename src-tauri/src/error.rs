use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    ShortcutConflict,
    ScreenPermissionDenied,
    CaptureFailed,
    NoActiveModel,
    NoActivePrompt,
    InsecureEndpoint,
    TlsFailed,
    NetworkUnavailable,
    Timeout,
    AuthFailed,
    RateLimited,
    ModelNotFound,
    ImageNotSupported,
    ImageTooLarge,
    ProviderError,
    EmptyResponse,
    StorageFull,
    StorageFailed,
    ClipboardFailed,
    AutostartApprovalRequired,
    AlreadyRunning,
    NotFound,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::ShortcutConflict => "shortcut_conflict",
            Self::ScreenPermissionDenied => "screen_permission_denied",
            Self::CaptureFailed => "capture_failed",
            Self::NoActiveModel => "no_active_model",
            Self::NoActivePrompt => "no_active_prompt",
            Self::InsecureEndpoint => "insecure_endpoint",
            Self::TlsFailed => "tls_failed",
            Self::NetworkUnavailable => "network_unavailable",
            Self::Timeout => "timeout",
            Self::AuthFailed => "auth_failed",
            Self::RateLimited => "rate_limited",
            Self::ModelNotFound => "model_not_found",
            Self::ImageNotSupported => "image_not_supported",
            Self::ImageTooLarge => "image_too_large",
            Self::ProviderError => "provider_error",
            Self::EmptyResponse => "empty_response",
            Self::StorageFull => "storage_full",
            Self::StorageFailed => "storage_failed",
            Self::ClipboardFailed => "clipboard_failed",
            Self::AutostartApprovalRequired => "autostart_approval_required",
            Self::AlreadyRunning => "already_running",
            Self::NotFound => "not_found",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

impl AppError {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        retryable: bool,
        action: Option<&str>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            retryable,
            action: action.map(str::to_owned),
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn message_with_details(&self) -> String {
        match self.details.as_deref() {
            Some(details) => format!("{}\n\n{details}", self.message),
            None => self.message.clone(),
        }
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::StorageFailed, message, false, Some("retry"))
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message, false, None)
    }

    pub fn provider(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self::new(code, message, retryable, Some("retry"))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}
