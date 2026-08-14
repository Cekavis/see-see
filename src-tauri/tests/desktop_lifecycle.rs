use see_see_lib::{
    autostart::{SystemAutostartStatus, confirm_enabled},
    capture::PhysicalRect,
    commands::{AnalysisStarted, finish_capture, open_main_window, resubmit_history},
    database::{DEFAULT_CAPTURE_SHORTCUT, Database},
    error::{AppError, ErrorCode},
    settings::{
        load_app_snapshot, replace_shortcut, sanitize_log_line, set_autostart_with,
        set_capture_shortcut_value,
    },
    windowing::{
        WindowRole, ignores_window_cycle, is_stationary, joins_all_spaces, policy_for,
        result_run_id, result_window_label, result_window_size, supports_full_screen_space,
    },
};
use std::cell::RefCell;
use std::future::Future;
use tauri::AppHandle;

#[test]
fn result_window_creation_stays_out_of_synchronous_windows_commands() {
    fn assert_async<F, Fut>(_: F)
    where
        F: Fn(AppHandle, String, PhysicalRect) -> Fut,
        Fut: Future<Output = Result<AnalysisStarted, AppError>>,
    {
    }

    assert_async(finish_capture);

    fn assert_resubmit_async<F, Fut>(_: F)
    where
        F: Fn(AppHandle, String, String, String) -> Fut,
        Fut: Future<Output = Result<AnalysisStarted, AppError>>,
    {
    }

    assert_resubmit_async(resubmit_history);
}

#[test]
fn result_navigation_uses_the_run_specific_window_command() {
    fn assert_command<F>(_: F)
    where
        F: Fn(AppHandle, String) -> Result<(), AppError>,
    {
    }

    assert_command(open_main_window);
}

#[test]
fn result_navigation_closes_only_after_a_terminal_state_check() {
    let commands = include_str!("../src/commands.rs");
    let navigation = commands
        .split_once("pub fn open_main_window(")
        .unwrap()
        .1
        .split_once("pub fn set_result_always_on_top(")
        .unwrap()
        .0;
    let focus = navigation.find("focus_main(&app)?").unwrap();
    let terminal = navigation.find(".is_terminal()").unwrap();
    let close = navigation.find(".close()").unwrap();

    assert!(focus < terminal);
    assert!(terminal < close);
}

#[test]
fn capture_overlay_disables_undecorated_window_shadow() {
    let commands = include_str!("../src/commands.rs");
    let capture_windows = commands
        .split_once("fn create_capture_windows(")
        .unwrap()
        .1
        .split_once("fn create_result_window(")
        .unwrap()
        .0;
    assert!(capture_windows.contains(".decorations(false)"));
    assert!(capture_windows.contains(".shadow(false)"));
}

#[test]
fn windows_capture_overlay_disables_show_transitions() {
    let windowing = include_str!("../src/windowing.rs");
    let capture_show = windowing
        .split_once("fn disable_capture_window_transitions")
        .unwrap()
        .1
        .split_once("pub fn present_result_window")
        .unwrap()
        .0;
    let disable = capture_show
        .find("DWMWA_TRANSITIONS_FORCEDISABLED")
        .unwrap();
    let show = capture_show.find(".show()").unwrap();
    assert!(disable < show);
}

#[test]
fn windows_capture_overlay_is_raised_and_focused_after_show() {
    let windowing = include_str!("../src/windowing.rs");
    let capture_show = windowing
        .split_once("fn raise_capture_window")
        .unwrap()
        .1
        .split_once("pub fn present_result_window")
        .unwrap()
        .0;
    assert!(capture_show.contains("SetWindowPos"));
    assert!(capture_show.contains("HWND_TOPMOST"));

    let show = capture_show.find(".show()").unwrap();
    let raise = capture_show.find("raise_capture_window(window)").unwrap();
    let focus = capture_show.find(".set_focus()").unwrap();
    assert!(show < raise);
    assert!(raise < focus);
}

#[test]
fn capture_overlay_waits_for_frontend_frame_readiness() {
    let commands = include_str!("../src/commands.rs");
    let create_windows = commands
        .split_once("fn create_capture_windows(")
        .unwrap()
        .1
        .split_once("fn create_result_window(")
        .unwrap()
        .0;
    assert!(!create_windows.contains("show_capture_window"));

    let show_ready = commands
        .split_once("pub fn show_capture_overlay(")
        .unwrap()
        .1
        .split_once("pub fn update_capture_selection(")
        .unwrap()
        .0;
    assert!(show_ready.contains("show_capture_window"));
}

#[test]
fn shortcut_replacement_registers_new_before_removing_old_and_rolls_back_on_conflict() {
    let calls = RefCell::new(Vec::new());
    replace_shortcut(
        "Alt+Shift+A",
        "Ctrl+Shift+X",
        |value| {
            calls.borrow_mut().push(format!("register:{value}"));
            Ok::<(), ()>(())
        },
        |value| {
            calls.borrow_mut().push(format!("unregister:{value}"));
            Ok::<(), ()>(())
        },
    )
    .unwrap();
    assert_eq!(
        calls.into_inner(),
        ["register:Ctrl+Shift+X", "unregister:Alt+Shift+A"]
    );

    let failed = replace_shortcut("Alt+Shift+A", "Taken", |_| Err(()), |_| Ok(()));
    assert!(failed.is_err());
}

#[test]
fn desktop_settings_only_persist_after_system_success() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(
        load_app_snapshot(&db).unwrap().settings.capture_shortcut,
        DEFAULT_CAPTURE_SHORTCUT
    );
    assert!(set_autostart_with(&db, true, |_| Err(())).is_err());
    assert!(!load_app_snapshot(&db).unwrap().settings.autostart);
    assert!(
        set_autostart_with(&db, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .autostart
    );
    assert_eq!(
        set_capture_shortcut_value(&db, "Ctrl+Shift+X")
            .unwrap()
            .capture_shortcut,
        "Ctrl+Shift+X"
    );
}

#[test]
fn macos_login_item_status_requires_explicit_approval() {
    assert_eq!(
        SystemAutostartStatus::from_raw(0),
        SystemAutostartStatus::NotRegistered
    );
    assert!(SystemAutostartStatus::from_raw(1).is_enabled());
    assert_eq!(
        SystemAutostartStatus::from_raw(3),
        SystemAutostartStatus::NotFound
    );
    assert_eq!(
        SystemAutostartStatus::from_raw(9),
        SystemAutostartStatus::Unknown(9)
    );

    let error = confirm_enabled(SystemAutostartStatus::RequiresApproval).unwrap_err();
    assert_eq!(error.code, ErrorCode::AutostartApprovalRequired);
    assert_eq!(error.action.as_deref(), Some("open_login_items"));
    assert!(error.message.contains("登录项与扩展"));
}

#[test]
fn macos_capture_and_result_windows_use_distinct_space_policies() {
    let capture = policy_for(WindowRole::CaptureOverlay);
    assert!(joins_all_spaces(capture));
    assert!(supports_full_screen_space(capture));
    assert!(is_stationary(capture));
    assert!(ignores_window_cycle(capture));
    assert!(capture.elevated_overlay_level);

    let result = policy_for(WindowRole::Result);
    assert!(joins_all_spaces(result));
    assert!(supports_full_screen_space(result));
    assert!(!is_stationary(result));
    assert!(!ignores_window_cycle(result));
    assert!(!result.elevated_overlay_level);
}

#[test]
fn result_window_defaults_are_compact_and_keep_accessible_minimums() {
    let size = result_window_size();
    assert!(size.width >= size.min_width);
    assert!(size.height >= size.min_height);
    assert!(size.width <= 480.0);
    assert!(size.height <= 520.0);
    assert_eq!((size.min_width, size.min_height), (420.0, 360.0));
}

#[test]
fn result_windows_use_unique_run_labels() {
    assert_eq!(result_window_label("first"), "result-first");
    assert_eq!(result_window_label("second"), "result-second");
    assert_eq!(result_run_id("result-first"), Some("first"));
    assert_eq!(result_run_id("result-"), None);
    assert_eq!(result_run_id("main"), None);
}

#[test]
fn exported_logs_remove_secrets_and_provider_payloads() {
    let sanitized =
        sanitize_log_line("Authorization: Bearer sk-secret api_key=abc raw_response={private}");
    assert!(!sanitized.contains("sk-secret"));
    assert!(!sanitized.contains("abc"));
    assert!(!sanitized.contains("private"));
}
