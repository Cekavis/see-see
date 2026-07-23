pub mod analysis;
pub mod capture;
pub mod commands;
pub mod credentials;
pub mod database;
pub mod error;
pub mod history;
pub mod providers;
pub mod settings;
pub mod state;

use credentials::SystemCredentialStore;
use database::Database;
use state::AppState;
use std::sync::Arc;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
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
            let database = Database::open(&directory.join("see-see.sqlite3"))
                .map_err(Box::<dyn std::error::Error>::from)?;
            let state = AppState::new(database, Arc::new(SystemCredentialStore))
                .map_err(Box::<dyn std::error::Error>::from)?;
            app.manage(state);
            let snapshot = settings::load_app_snapshot(&app.state::<AppState>().database)
                .map_err(Box::<dyn std::error::Error>::from)?;
            commands::register_capture_shortcut(app.handle(), &snapshot.settings.capture_shortcut)?;
            if let Ok(actual) = app.autolaunch().is_enabled()
                && actual != snapshot.settings.autostart
            {
                let _ =
                    settings::set_autostart_with(&app.state::<AppState>().database, actual, |_| {
                        Ok::<(), ()>(())
                    });
            }
            let show = MenuItem::with_id(app, "show", "打开 See See", true, None::<&str>)?;
            let history = MenuItem::with_id(app, "history", "历史记录", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &history, &settings, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("See See")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "history" | "settings" => {
                        tauri::async_runtime::spawn(commands::open_view(
                            app.clone(),
                            event.id.as_ref().to_owned(),
                        ));
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
                if matches!(
                    window.label(),
                    "main" | "settings" | "history" | "prompts" | "onboarding"
                ) {
                    api.prevent_close();
                    let _ = window.hide();
                } else if window.label() == "result" {
                    if let Ok(mut runtime) = window.app_handle().state::<AppState>().runtime.lock()
                        && let Some(active) = runtime.analysis.take()
                    {
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
            commands::open_view,
            commands::begin_capture,
            commands::get_capture_frame,
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
            commands::complete_onboarding,
            commands::open_screen_permission_settings,
            commands::export_sanitized_logs,
            commands::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run See See");
}
