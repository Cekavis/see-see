pub mod analysis;
pub mod autostart;
pub mod capture;
pub mod commands;
pub mod credentials;
pub mod database;
pub mod error;
pub mod history;
pub mod providers;
pub mod settings;
pub mod state;
pub mod windowing;

use credentials::SystemCredentialStore;
use database::Database;
use state::AppState;
use std::sync::Arc;
use tauri::{
    AppHandle, Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_log::{Target, TargetKind};

fn should_hide_on_close(label: &str) -> bool {
    label == "main"
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(unix)]
fn restrict_local_storage_permissions(directory: &std::path::Path, database: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700));
    let _ = std::fs::set_permissions(database, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_local_storage_permissions(_: &std::path::Path, _: &std::path::Path) {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::LogDir { file_name: None })])
                .build(),
        )
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let database_path = directory.join("see-see.sqlite3");
            let database =
                Database::open(&database_path).map_err(Box::<dyn std::error::Error>::from)?;
            restrict_local_storage_permissions(&directory, &database_path);
            let state = AppState::new(database, Arc::new(SystemCredentialStore))
                .map_err(Box::<dyn std::error::Error>::from)?;
            app.manage(state);
            let snapshot = settings::load_app_snapshot(&app.state::<AppState>().database)
                .map_err(Box::<dyn std::error::Error>::from)?;
            commands::register_capture_shortcut(app.handle(), &snapshot.settings.capture_shortcut)?;
            match autostart::reconcile_on_startup(app.handle(), snapshot.settings.autostart) {
                Ok(actual) if actual != snapshot.settings.autostart => {
                    let _ = settings::set_autostart_with(
                        &app.state::<AppState>().database,
                        actual,
                        |_| Ok::<(), ()>(()),
                    );
                }
                Err(error) => log::warn!("无法同步开机启动状态：{error}"),
                _ => {}
            }
            let capture = MenuItem::with_id(app, "capture", "开始截图", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "打开 See See", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&capture, &show, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("See See")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "capture" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(error) = commands::begin_capture_action(app.clone()).await {
                                commands::report_capture_failure(&app, "tray", &error);
                            }
                        });
                    }
                    "show" => {
                        show_main_window(app);
                    }
                    "quit" => commands::quit_app(app.clone()),
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if should_hide_on_close(window.label()) {
                    api.prevent_close();
                    let _ = window.hide();
                } else if let Some(run_id) = windowing::result_run_id(window.label()) {
                    let active = window
                        .app_handle()
                        .state::<AppState>()
                        .runtime
                        .lock()
                        .ok()
                        .and_then(|mut runtime| runtime.take_analysis(run_id));
                    if let Some(active) = active {
                        let _ = active.cancel();
                    }
                } else if window.label().starts_with("capture-")
                    && let Ok(mut runtime) = window.app_handle().state::<AppState>().runtime.lock()
                {
                    runtime.capture = None;
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::begin_capture,
            commands::get_capture_frame,
            commands::show_capture_overlay,
            commands::update_capture_selection,
            commands::finish_capture,
            commands::cancel_capture,
            commands::attach_analysis,
            commands::cancel_analysis,
            commands::close_result,
            commands::set_result_always_on_top,
            commands::copy_text,
            commands::list_model_configs,
            commands::save_model_config,
            commands::delete_model_config,
            commands::duplicate_model_config,
            commands::set_active_model_config,
            commands::list_remote_models,
            commands::test_model_config,
            commands::list_prompt_presets,
            commands::save_prompt_preset,
            commands::duplicate_prompt_preset,
            commands::delete_prompt_preset,
            commands::set_active_prompt,
            commands::query_history,
            commands::get_history_entry,
            commands::get_history_image,
            commands::resubmit_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::set_save_history,
            commands::get_settings,
            commands::set_capture_shortcut,
            commands::set_autostart,
            commands::open_login_items_settings,
            commands::complete_onboarding,
            commands::request_screen_permission,
            commands::open_screen_permission_settings,
            commands::export_sanitized_logs,
            commands::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run See See");
}

#[cfg(test)]
mod tests {
    use super::should_hide_on_close;

    #[test]
    fn only_the_main_management_window_hides_on_close() {
        assert!(should_hide_on_close("main"));
        for label in [
            "settings",
            "prompts",
            "history",
            "onboarding",
            "result-run-id",
            "result-",
            "capture-1",
        ] {
            assert!(
                !should_hide_on_close(label),
                "unexpected hidden window: {label}"
            );
        }
    }
}
