use crate::{
    analysis::{self, ActiveAnalysis, AnalysisEvent, AnalysisInput, AnalysisSnapshot},
    capture::{CaptureSession, CaptureSessionSummary, PhysicalRect, compose_selection},
    error::{AppError, ErrorCode},
    history::{self, HistoryEntryDetail, HistoryImageVariant, HistoryPage, HistoryQuery},
    providers::{self, ProviderProtocol, ProviderRequest, RemoteModel},
    settings::{self, ModelConfigInput, ModelConfigSummary, PromptPreset, PromptPresetInput},
    state::AppState,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    ipc::{Channel, Response},
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
#[cfg(target_os = "macos")]
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStarted {
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearHistoryResult {
    pub deleted_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub exported: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConnectionInput {
    pub id: Option<String>,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
}

#[tauri::command]
pub fn get_app_snapshot(app: AppHandle) -> Result<settings::AppSnapshot, AppError> {
    settings::load_app_snapshot(&app.state::<AppState>().database)
}

#[tauri::command]
pub fn open_view(app: AppHandle, view: String) -> Result<(), AppError> {
    let label = match view.as_str() {
        "history" | "prompts" | "settings" => view,
        _ => return Err(AppError::invalid("未知界面")),
    };
    if let Some(window) = app.get_webview_window(&label) {
        window
            .show()
            .map_err(|_| AppError::invalid("无法显示窗口"))?;
        window
            .set_focus()
            .map_err(|_| AppError::invalid("无法聚焦窗口"))?;
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
        .title("See See")
        .inner_size(1024.0, 720.0)
        .build()
        .map_err(|_| AppError::invalid("无法创建窗口"))?;
    Ok(())
}

#[tauri::command]
pub async fn begin_capture(app: AppHandle) -> Result<CaptureSessionSummary, AppError> {
    begin_capture_action(app).await
}

pub async fn begin_capture_action(app: AppHandle) -> Result<CaptureSessionSummary, AppError> {
    let state = app.state::<AppState>();
    {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        if runtime.capture.is_some() {
            return Err(already_running("截图正在进行"));
        }
        if let Some(active) = &runtime.analysis
            && !active.snapshot()?.state.is_terminal()
        {
            focus_result(&app);
            return Err(already_running("已有分析正在进行"));
        }
        runtime.analysis = None;
    }
    require_active_configuration(&state)?;

    let session_id = Uuid::new_v4().to_string();
    let session =
        tauri::async_runtime::spawn_blocking(move || CaptureSession::capture_all(session_id))
            .await
            .map_err(|_| {
                AppError::new(
                    ErrorCode::CaptureFailed,
                    "截图任务异常结束",
                    false,
                    Some("retry"),
                )
            })??;
    let summary = session.summary();
    {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        if runtime.capture.is_some() || runtime.analysis.is_some() {
            return Err(already_running("已有任务正在进行"));
        }
        runtime.capture = Some(session);
    }
    if let Err(error) = create_capture_windows(&app, &summary) {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        runtime.capture = None;
        close_capture_windows(&app, &summary);
        return Err(error);
    }
    Ok(summary)
}

#[tauri::command]
pub fn get_capture_frame(
    app: AppHandle,
    session_id: String,
    monitor_id: String,
) -> Result<Response, AppError> {
    let state = app.state::<AppState>();
    let runtime = state
        .runtime
        .lock()
        .map_err(|_| AppError::storage("运行状态不可用"))?;
    let session = runtime
        .capture
        .as_ref()
        .filter(|session| session.id == session_id)
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "截图会话不存在", false, None))?;
    Ok(Response::new(session.frame(&monitor_id)?))
}

#[tauri::command]
pub fn update_capture_selection(
    app: AppHandle,
    session_id: String,
    selection: PhysicalRect,
) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let labels = {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        let session = runtime
            .capture
            .as_mut()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "截图会话不存在", false, None))?;
        session.update_selection(selection)?;
        session
            .monitors
            .iter()
            .map(|monitor| capture_label(&monitor.summary.id))
            .collect::<Vec<_>>()
    };
    for label in labels {
        let _ = app.emit_to(label, "capture-selection", selection);
    }
    Ok(())
}

#[tauri::command]
pub fn finish_capture(
    app: AppHandle,
    session_id: String,
    selection: PhysicalRect,
) -> Result<AnalysisStarted, AppError> {
    let state = app.state::<AppState>();
    let session = {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        let session = runtime
            .capture
            .as_mut()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "截图会话不存在", false, None))?;
        session.update_selection(selection)?;
        runtime.take_capture(&session_id)?
    };
    let summary = session.summary();
    close_capture_windows(&app, &summary);
    let image_png = compose_selection(&session.monitors, selection)?;
    start_analysis_with_image(app, image_png)
}

fn start_analysis_with_image(
    app: AppHandle,
    image_png: Vec<u8>,
) -> Result<AnalysisStarted, AppError> {
    let state = app.state::<AppState>();
    let (model, prompt) = require_active_configuration(&state)?;
    let api_key = model
        .credential_ref
        .as_deref()
        .map(|reference| state.credentials.get(reference))
        .transpose()?
        .flatten();
    let save_history = settings::load_app_snapshot(&state.database)?
        .settings
        .save_history;
    let run_id = Uuid::new_v4().to_string();
    let active = Arc::new(ActiveAnalysis::new(run_id.clone()));
    {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        if let Some(current) = &runtime.analysis
            && !current.snapshot()?.state.is_terminal()
        {
            focus_result(&app);
            return Err(already_running("已有分析正在进行"));
        }
        runtime.analysis = Some(active.clone());
    }
    if let Err(error) = create_result_window(&app, &run_id) {
        state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?
            .analysis = None;
        return Err(error);
    }
    analysis::start_network_analysis(
        app,
        active,
        AnalysisInput {
            image_png,
            prompt,
            model,
            api_key,
            save_history,
            started_at: analysis::now(),
        },
    );
    Ok(AnalysisStarted { run_id })
}

#[tauri::command]
pub fn cancel_capture(app: AppHandle, session_id: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let session = {
        let mut runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        runtime.take_capture(&session_id)?
    };
    close_capture_windows(&app, &session.summary());
    Ok(())
}

#[tauri::command]
pub fn attach_analysis(
    app: AppHandle,
    run_id: String,
    on_event: Channel<AnalysisEvent>,
) -> Result<AnalysisSnapshot, AppError> {
    active_analysis(&app, &run_id)?.subscribe(on_event)
}

#[tauri::command]
pub fn cancel_analysis(app: AppHandle, run_id: String) -> Result<(), AppError> {
    active_analysis(&app, &run_id)?.cancel()
}

#[tauri::command]
pub fn close_result(app: AppHandle, run_id: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let active = active_analysis(&app, &run_id)?;
    if !active.snapshot()?.state.is_terminal() {
        active.cancel()?;
    }
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| AppError::storage("运行状态不可用"))?;
    if runtime.analysis.as_ref().is_some_and(|current| {
        current
            .snapshot()
            .is_ok_and(|snapshot| snapshot.run_id == run_id)
    }) {
        runtime.analysis = None;
    }
    Ok(())
}

#[tauri::command]
pub fn set_result_always_on_top(app: AppHandle, value: bool) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("result") {
        window
            .set_always_on_top(value)
            .map_err(|_| AppError::invalid("无法更新窗口置顶状态"))?;
    }
    app.state::<AppState>().database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET result_always_on_top = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![value, analysis::now()],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), AppError> {
    if text.is_empty() {
        return Err(AppError::invalid("没有可复制的文字"));
    }
    app.clipboard().write_text(text).map_err(|_| {
        AppError::new(
            ErrorCode::ClipboardFailed,
            "无法写入剪贴板",
            true,
            Some("retry"),
        )
    })
}

#[tauri::command]
pub fn list_model_configs(app: AppHandle) -> Result<Vec<ModelConfigSummary>, AppError> {
    let state = app.state::<AppState>();
    settings::list_model_configs(&state.database, state.credentials.as_ref())
}

#[tauri::command]
pub fn save_model_config(
    app: AppHandle,
    input: ModelConfigInput,
) -> Result<ModelConfigSummary, AppError> {
    let state = app.state::<AppState>();
    settings::save_model_config(&state.database, state.credentials.as_ref(), input)
}

#[tauri::command]
pub fn delete_model_config(app: AppHandle, id: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    settings::delete_model_config(&state.database, state.credentials.as_ref(), &id)
}

#[tauri::command]
pub fn set_active_model_config(app: AppHandle, id: String) -> Result<(), AppError> {
    settings::set_active_model_config(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub async fn list_remote_models(
    app: AppHandle,
    draft: ModelConnectionInput,
) -> Result<Vec<RemoteModel>, AppError> {
    let state = app.state::<AppState>();
    let key = connection_key(&state, &draft)?;
    providers::list_models(&state.http, draft.protocol, &draft.base_url, key.as_ref()).await
}

#[tauri::command]
pub async fn test_model_config(
    app: AppHandle,
    draft: ModelConnectionInput,
) -> Result<providers::ConnectionTestResult, AppError> {
    providers::validate_endpoint(&draft.base_url)?;
    if draft.model_id.trim().is_empty() {
        return Err(AppError::invalid("模型 ID 不能为空"));
    }
    let state = app.state::<AppState>();
    let key = connection_key(&state, &draft)?;
    let result = providers::test_connection(
        &state.http,
        ProviderRequest {
            protocol: draft.protocol,
            base_url: draft.base_url,
            model_id: draft.model_id,
            api_key: key,
            prompt: "请只回复 OK。".into(),
            image_png: providers::connection_test_png(),
            stream: true,
        },
    )
    .await;
    if let Some(id) = draft.id.as_deref() {
        settings::record_model_test(
            &state.database,
            id,
            result.passed,
            result.error.as_ref().map(|error| error.code.as_str()),
        )?;
    }
    Ok(result)
}

#[tauri::command]
pub fn list_prompt_presets(app: AppHandle) -> Result<Vec<PromptPreset>, AppError> {
    settings::list_prompt_presets(&app.state::<AppState>().database)
}

#[tauri::command]
pub fn save_prompt_preset(
    app: AppHandle,
    input: PromptPresetInput,
) -> Result<PromptPreset, AppError> {
    settings::save_prompt_preset(&app.state::<AppState>().database, input)
}

#[tauri::command]
pub fn duplicate_prompt_preset(app: AppHandle, id: String) -> Result<PromptPreset, AppError> {
    settings::duplicate_prompt_preset(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub fn delete_prompt_preset(app: AppHandle, id: String) -> Result<(), AppError> {
    settings::delete_prompt_preset(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub fn set_active_prompt(app: AppHandle, id: String) -> Result<(), AppError> {
    settings::set_active_prompt(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub fn query_history(app: AppHandle, query: HistoryQuery) -> Result<HistoryPage, AppError> {
    history::query_history(&app.state::<AppState>().database, query)
}

#[tauri::command]
pub fn get_history_entry(app: AppHandle, id: String) -> Result<HistoryEntryDetail, AppError> {
    history::get_history_detail(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub fn get_history_image(
    app: AppHandle,
    id: String,
    variant: HistoryImageVariant,
) -> Result<Response, AppError> {
    Ok(Response::new(history::get_history_image(
        &app.state::<AppState>().database,
        &id,
        variant,
    )?))
}

#[tauri::command]
pub fn resubmit_history(app: AppHandle, id: String) -> Result<AnalysisStarted, AppError> {
    {
        let state = app.state::<AppState>();
        let runtime = state
            .runtime
            .lock()
            .map_err(|_| AppError::storage("运行状态不可用"))?;
        if runtime.capture.is_some() {
            return Err(already_running("截图正在进行"));
        }
    }
    let image = history::get_history_image(
        &app.state::<AppState>().database,
        &id,
        HistoryImageVariant::Original,
    )?;
    start_analysis_with_image(app, image)
}

#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: String) -> Result<(), AppError> {
    history::delete_history_entry(&app.state::<AppState>().database, &id)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<ClearHistoryResult, AppError> {
    Ok(ClearHistoryResult {
        deleted_count: history::clear_history(&app.state::<AppState>().database)?,
    })
}

#[tauri::command]
pub fn set_save_history(app: AppHandle, value: bool) -> Result<settings::AppSettings, AppError> {
    settings::set_save_history(&app.state::<AppState>().database, value)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<settings::AppSettings, AppError> {
    Ok(settings::load_app_snapshot(&app.state::<AppState>().database)?.settings)
}

#[tauri::command]
pub fn set_capture_shortcut(
    app: AppHandle,
    shortcut: String,
) -> Result<settings::AppSettings, AppError> {
    shortcut
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|_| AppError::invalid("快捷键格式无效"))?;
    let state = app.state::<AppState>();
    let old = settings::load_app_snapshot(&state.database)?
        .settings
        .capture_shortcut;
    settings::replace_shortcut(
        &old,
        &shortcut,
        |value| register_capture_shortcut(&app, value),
        |value| app.global_shortcut().unregister(value),
    )?;
    match settings::set_capture_shortcut_value(&state.database, &shortcut) {
        Ok(settings) => Ok(settings),
        Err(error) => {
            let _ = register_capture_shortcut(&app, &old);
            let _ = app.global_shortcut().unregister(shortcut.as_str());
            Err(error)
        }
    }
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, value: bool) -> Result<settings::AppSettings, AppError> {
    let state = app.state::<AppState>();
    settings::set_autostart_with(&state.database, value, |enabled| {
        if enabled {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        }
    })
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle) -> Result<(), AppError> {
    settings::complete_onboarding(&app.state::<AppState>().database)
}

#[tauri::command]
pub fn open_screen_permission_settings(app: AppHandle) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    app.opener()
        .open_url(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            None::<&str>,
        )
        .map_err(|_| AppError::invalid("无法打开屏幕录制权限设置"))?;
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    Ok(())
}

#[tauri::command]
pub async fn export_sanitized_logs(app: AppHandle) -> Result<ExportResult, AppError> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|_| AppError::storage("无法定位日志目录"))?;
    let path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_file_name("see-see-diagnostics.txt")
            .blocking_save_file()
    })
    .await
    .map_err(|_| AppError::storage("日志导出对话框异常结束"))?;
    let Some(path) = path else {
        return Ok(ExportResult { exported: false });
    };
    let path = path
        .into_path()
        .map_err(|_| AppError::storage("导出路径无效"))?;
    let mut output = String::new();
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|value| value.to_str()) != Some("log") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    output.push_str(&settings::sanitize_log_line(line));
                    output.push('\n');
                }
            }
        }
    }
    std::fs::write(path, output).map_err(|_| AppError::storage("无法写入诊断日志"))?;
    Ok(ExportResult { exported: true })
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    if let Ok(runtime) = app.state::<AppState>().runtime.lock()
        && let Some(active) = &runtime.analysis
    {
        let _ = active.cancel();
    }
    app.exit(0);
}

fn require_active_configuration(
    state: &AppState,
) -> Result<(settings::ModelSnapshot, settings::PromptSnapshot), AppError> {
    let model = settings::load_active_model(&state.database)?.ok_or_else(|| {
        AppError::new(
            ErrorCode::NoActiveModel,
            "请先选择并测试一个模型配置",
            false,
            Some("edit_model_config"),
        )
    })?;
    let prompt = settings::load_active_prompt(&state.database)?.ok_or_else(|| {
        AppError::new(
            ErrorCode::NoActivePrompt,
            "请先选择一个提示词",
            false,
            Some("edit_prompt"),
        )
    })?;
    Ok((model, prompt))
}

fn connection_key(
    state: &AppState,
    draft: &ModelConnectionInput,
) -> Result<Option<SecretString>, AppError> {
    if let Some(api_key) = draft.api_key.as_ref() {
        return Ok(Some(SecretString::from(api_key.clone())));
    }
    let Some(id) = draft.id.as_deref() else {
        return Ok(None);
    };
    let model = settings::load_model(&state.database, id)?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "模型配置不存在", false, None))?;
    model
        .credential_ref
        .as_deref()
        .map(|reference| state.credentials.get(reference))
        .transpose()
        .map(Option::flatten)
}

fn active_analysis(app: &AppHandle, run_id: &str) -> Result<Arc<ActiveAnalysis>, AppError> {
    let state = app.state::<AppState>();
    let runtime = state
        .runtime
        .lock()
        .map_err(|_| AppError::storage("运行状态不可用"))?;
    runtime
        .analysis
        .as_ref()
        .filter(|active| {
            active
                .snapshot()
                .is_ok_and(|snapshot| snapshot.run_id == run_id)
        })
        .cloned()
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "分析任务不存在", false, None))
}

fn create_capture_windows(
    app: &AppHandle,
    summary: &CaptureSessionSummary,
) -> Result<(), AppError> {
    for monitor in &summary.monitors {
        let label = capture_label(&monitor.id);
        let url = format!(
            "index.html?session={}&monitor={}&x={}&y={}&scale={}",
            summary.session_id,
            monitor.id,
            monitor.bounds.x,
            monitor.bounds.y,
            monitor.scale_factor
        );
        let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
            .title("See See Capture")
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false)
            .build()
            .map_err(|_| {
                AppError::new(
                    ErrorCode::CaptureFailed,
                    "无法创建截图遮罩",
                    false,
                    Some("retry"),
                )
            })?;
        window
            .set_position(PhysicalPosition::new(monitor.bounds.x, monitor.bounds.y))
            .and_then(|_| {
                window.set_size(PhysicalSize::new(
                    monitor.bounds.width,
                    monitor.bounds.height,
                ))
            })
            .and_then(|_| window.show())
            .map_err(|_| {
                AppError::new(
                    ErrorCode::CaptureFailed,
                    "无法显示截图遮罩",
                    false,
                    Some("retry"),
                )
            })?;
    }
    Ok(())
}

fn create_result_window(app: &AppHandle, run_id: &str) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("result") {
        let _ = window.destroy();
    }
    let always_on_top = settings::load_app_snapshot(&app.state::<AppState>().database)?
        .settings
        .result_always_on_top;
    WebviewWindowBuilder::new(
        app,
        "result",
        WebviewUrl::App(format!("index.html?run={run_id}").into()),
    )
    .title("See See · 识别结果")
    .inner_size(620.0, 720.0)
    .min_inner_size(420.0, 360.0)
    .always_on_top(always_on_top)
    .build()
    .map_err(|_| AppError::invalid("无法创建结果窗口"))?;
    Ok(())
}

fn close_capture_windows(app: &AppHandle, summary: &CaptureSessionSummary) {
    for monitor in &summary.monitors {
        if let Some(window) = app.get_webview_window(&capture_label(&monitor.id)) {
            let _ = window.destroy();
        }
    }
}

fn capture_label(monitor_id: &str) -> String {
    format!("capture-{monitor_id}")
}

fn focus_result(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("result") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn already_running(message: &str) -> AppError {
    AppError::new(
        ErrorCode::AlreadyRunning,
        message,
        false,
        Some("focus_active"),
    )
}

pub fn register_capture_shortcut(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = begin_capture_action(app).await {
                        log::warn!("capture shortcut failed: {}", error.code.as_str());
                    }
                });
            }
        })
}
