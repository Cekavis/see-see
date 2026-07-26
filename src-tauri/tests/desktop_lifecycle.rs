use see_see_lib::{
    autostart::{SystemAutostartStatus, confirm_enabled},
    database::{DEFAULT_CAPTURE_SHORTCUT, Database},
    error::ErrorCode,
    settings::{
        load_app_snapshot, replace_shortcut, sanitize_log_line, set_autostart_with,
        set_capture_shortcut_value,
    },
    windowing::{
        WindowRole, ignores_window_cycle, is_stationary, joins_all_spaces, moves_to_active_space,
        policy_for, result_window_size, supports_full_screen_space,
    },
};
use std::cell::RefCell;

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
    assert!(!moves_to_active_space(capture));
    assert!(supports_full_screen_space(capture));
    assert!(is_stationary(capture));
    assert!(ignores_window_cycle(capture));
    assert!(capture.elevated_overlay_level);

    let result = policy_for(WindowRole::Result);
    assert!(!joins_all_spaces(result));
    assert!(moves_to_active_space(result));
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
fn exported_logs_remove_secrets_and_provider_payloads() {
    let sanitized =
        sanitize_log_line("Authorization: Bearer sk-secret api_key=abc raw_response={private}");
    assert!(!sanitized.contains("sk-secret"));
    assert!(!sanitized.contains("abc"));
    assert!(!sanitized.contains("private"));
}
