use see_see_lib::{
    database::Database,
    settings::{
        load_app_snapshot, replace_shortcut, sanitize_log_line, set_autostart_with,
        set_capture_shortcut_value,
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
fn exported_logs_remove_secrets_and_provider_payloads() {
    let sanitized =
        sanitize_log_line("Authorization: Bearer sk-secret api_key=abc raw_response={private}");
    assert!(!sanitized.contains("sk-secret"));
    assert!(!sanitized.contains("abc"));
    assert!(!sanitized.contains("private"));
}
