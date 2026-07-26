use crate::error::AppError;
use tauri::WebviewWindow;

const RESULT_DEFAULT_WIDTH: f64 = 460.0;
const RESULT_DEFAULT_HEIGHT: f64 = 500.0;
const RESULT_MIN_WIDTH: f64 = 420.0;
const RESULT_MIN_HEIGHT: f64 = 360.0;

const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
const MOVE_TO_ACTIVE_SPACE: usize = 1 << 1;
const STATIONARY: usize = 1 << 4;
const IGNORES_CYCLE: usize = 1 << 6;
const FULL_SCREEN_AUXILIARY: usize = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRole {
    CaptureOverlay,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPolicy {
    pub collection_behavior: usize,
    pub elevated_overlay_level: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResultWindowSize {
    pub width: f64,
    pub height: f64,
    pub min_width: f64,
    pub min_height: f64,
}

pub fn result_window_size() -> ResultWindowSize {
    ResultWindowSize {
        width: RESULT_DEFAULT_WIDTH,
        height: RESULT_DEFAULT_HEIGHT,
        min_width: RESULT_MIN_WIDTH,
        min_height: RESULT_MIN_HEIGHT,
    }
}

pub fn policy_for(role: WindowRole) -> WindowPolicy {
    match role {
        WindowRole::CaptureOverlay => WindowPolicy {
            collection_behavior: CAN_JOIN_ALL_SPACES
                | FULL_SCREEN_AUXILIARY
                | STATIONARY
                | IGNORES_CYCLE,
            elevated_overlay_level: true,
        },
        WindowRole::Result => WindowPolicy {
            collection_behavior: MOVE_TO_ACTIVE_SPACE | FULL_SCREEN_AUXILIARY,
            elevated_overlay_level: false,
        },
    }
}

pub fn joins_all_spaces(policy: WindowPolicy) -> bool {
    policy.collection_behavior & CAN_JOIN_ALL_SPACES != 0
}

pub fn moves_to_active_space(policy: WindowPolicy) -> bool {
    policy.collection_behavior & MOVE_TO_ACTIVE_SPACE != 0
}

pub fn supports_full_screen_space(policy: WindowPolicy) -> bool {
    policy.collection_behavior & FULL_SCREEN_AUXILIARY != 0
}

pub fn is_stationary(policy: WindowPolicy) -> bool {
    policy.collection_behavior & STATIONARY != 0
}

pub fn ignores_window_cycle(policy: WindowPolicy) -> bool {
    policy.collection_behavior & IGNORES_CYCLE != 0
}

#[cfg(target_os = "macos")]
fn apply_macos_policy(native_window: usize, policy: WindowPolicy) {
    use objc2_app_kit::{NSPopUpMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};

    // SAFETY: Tauri owns this NSWindow for at least as long as the cloned
    // WebviewWindow captured by the main-thread callback. AppKit access is
    // confined to that callback.
    let native_window = unsafe { &*(native_window as *const NSWindow) };
    native_window.setCollectionBehavior(NSWindowCollectionBehavior::from_bits_retain(
        policy.collection_behavior,
    ));
    if policy.elevated_overlay_level {
        native_window.setLevel(NSPopUpMenuWindowLevel);
    }
}

#[cfg(target_os = "macos")]
fn native_window(window: &WebviewWindow) -> Result<usize, AppError> {
    window
        .ns_window()
        .map(|value| value as usize)
        .map_err(|_| AppError::invalid("无法访问 macOS 原生窗口"))
}

pub fn show_capture_window(window: &WebviewWindow) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        let native_window = native_window(window)?;
        let window = window.clone();
        window
            .clone()
            .run_on_main_thread(move || {
                apply_macos_policy(native_window, policy_for(WindowRole::CaptureOverlay));
                let _ = window.show();
            })
            .map_err(|_| AppError::invalid("无法配置 macOS 截图窗口"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        window
            .show()
            .map_err(|_| AppError::invalid("无法显示截图窗口"))
    }
}

pub fn present_result_window(window: &WebviewWindow) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        let native_window = native_window(window)?;
        let window = window.clone();
        window
            .clone()
            .run_on_main_thread(move || {
                apply_macos_policy(native_window, policy_for(WindowRole::Result));
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
            })
            .map_err(|_| AppError::invalid("无法配置 macOS 结果窗口"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        window
            .center()
            .and_then(|_| window.show())
            .and_then(|_| window.set_focus())
            .map_err(|_| AppError::invalid("无法显示结果窗口"))
    }
}
