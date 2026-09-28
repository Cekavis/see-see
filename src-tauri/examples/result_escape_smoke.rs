//! Windows native-key regression check. Start `npm run dev` first, then run:
//! cargo run --manifest-path src-tauri/Cargo.toml --example result_escape_smoke
//! Uses in-memory data and an isolated WebView profile; never calls a provider.

#[cfg(target_os = "windows")]
mod smoke {
    use see_see_lib::{
        analysis::ActiveAnalysis,
        commands,
        credentials::MemoryCredentialStore,
        database::Database,
        providers::ProviderProtocol,
        settings::{self, ModelConfigInput},
        state::AppState,
        windowing,
    };
    use std::{
        collections::HashSet,
        path::PathBuf,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };
    use tauri::{
        AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
    };
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
            VK_CONTROL, VK_ESCAPE, VK_W,
        },
        WindowsAndMessaging::GetForegroundWindow,
    };

    struct SmokeState {
        expected_close: Mutex<HashSet<String>>,
        escape_held: AtomicBool,
        completed: Arc<AtomicBool>,
        profile: PathBuf,
    }

    fn key_input(key: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    dwFlags: if up {
                        KEYEVENTF_KEYUP
                    } else {
                        Default::default()
                    },
                    ..Default::default()
                },
            },
        }
    }

    #[tauri::command]
    async fn smoke_press_key(
        window: WebviewWindow,
        key: String,
        expect_close: bool,
    ) -> Result<(), String> {
        if !window.label().starts_with("result-native-escape-") {
            return Err("Refusing keyboard input for a non-test window".into());
        }
        window.set_focus().map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        if unsafe { GetForegroundWindow() } != hwnd {
            return Err("Test window is not foreground; no input was sent".into());
        }
        let inputs = match key.as_str() {
            "Escape" => vec![key_input(VK_ESCAPE, false), key_input(VK_ESCAPE, true)],
            "EscapeDown" => vec![key_input(VK_ESCAPE, false)],
            "EscapeRepeatAndUp" => vec![key_input(VK_ESCAPE, false), key_input(VK_ESCAPE, true)],
            "Ctrl+W" => vec![
                key_input(VK_CONTROL, false),
                key_input(VK_W, false),
                key_input(VK_W, true),
                key_input(VK_CONTROL, true),
            ],
            _ => return Err("Unknown smoke-test key".into()),
        };
        if expect_close {
            window
                .state::<SmokeState>()
                .expected_close
                .lock()
                .unwrap()
                .insert(window.label().into());
        }
        let count = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        let state = window.state::<SmokeState>();
        if key == "EscapeDown" && count == 1 {
            state.escape_held.store(true, Ordering::SeqCst);
        } else if key == "EscapeRepeatAndUp" && count == 2 {
            state.escape_held.store(false, Ordering::SeqCst);
        }
        if count != inputs.len() as u32 {
            return Err(format!("Sent {count} of {} test key events", inputs.len()));
        }
        Ok(())
    }

    #[tauri::command]
    fn smoke_report(app: AppHandle, message: String, failed: bool) {
        println!("{}: {message}", if failed { "FAIL" } else { "PASS" });
        if failed {
            app.exit(1);
        }
    }

    fn open_scenario(app: &AppHandle, scenario: usize) -> Result<(), Box<dyn std::error::Error>> {
        let run_id = format!("native-escape-{scenario}");
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1).write_to(&mut png, image::ImageFormat::Png)?;
        let analysis = Arc::new(ActiveAnalysis::new(
            &run_id,
            png.into_inner(),
            "Native test model",
            "Native test prompt",
        ));
        analysis.push_text("Native Escape smoke result")?;
        analysis.complete(false)?;
        app.state::<AppState>()
            .runtime
            .lock()
            .unwrap()
            .analysis
            .insert(run_id.clone(), analysis);
        let window = WebviewWindowBuilder::new(
            app,
            windowing::result_window_label(&run_id),
            WebviewUrl::App(
                format!("tests/native/result-escape.html?run={run_id}&scenario={scenario}").into(),
            ),
        )
        .title("See See · Native Escape test")
        .inner_size(460.0, 540.0)
        .data_directory(app.state::<SmokeState>().profile.join(&run_id))
        .build()?;
        windowing::install_native_close_shortcuts(&window, true)?;
        window.set_focus()?;
        Ok(())
    }

    pub fn run() -> Result<i32, Box<dyn std::error::Error>> {
        let build_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../build");
        std::fs::create_dir_all(&build_dir)?;
        let profile = tempfile::Builder::new()
            .prefix("native-escape-")
            .tempdir_in(build_dir)?;
        let state = AppState::new(
            Database::open_in_memory()?,
            Arc::new(MemoryCredentialStore::default()),
        )?;
        settings::save_model_config(
            &state.database,
            ModelConfigInput {
                id: None,
                name: "Native test model".into(),
                protocol: ProviderProtocol::OpenAi,
                base_url: "https://example.test/v1".into(),
                model_id: "native-test".into(),
                api_key: None,
                reasoning_effort: None,
                clear_api_key: false,
            },
        )?;
        let mut context = tauri::generate_context!();
        context.config_mut().app.windows.clear();
        let completed = Arc::new(AtomicBool::new(false));
        let app = tauri::Builder::default()
            .manage(state)
            .manage(SmokeState {
                expected_close: Mutex::new(HashSet::new()),
                escape_held: AtomicBool::new(false),
                completed: Arc::clone(&completed),
                profile: profile.path().into(),
            })
            .invoke_handler(tauri::generate_handler![
                commands::get_app_snapshot,
                commands::get_analysis_image,
                commands::attach_analysis,
                commands::list_model_configs,
                commands::set_result_model_chooser_open,
                smoke_press_key,
                smoke_report,
            ])
            .setup(|app| {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = open_scenario(&handle, 0) {
                        eprintln!("FAIL: {error}");
                        handle.exit(1);
                    }
                });
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(45)).await;
                    eprintln!("FAIL: native Escape smoke timed out");
                    handle.exit(1);
                });
                Ok(())
            })
            .on_window_event(|window, event| {
                if let WindowEvent::Destroyed = event {
                    let app = window.app_handle();
                    let expected = app
                        .state::<SmokeState>()
                        .expected_close
                        .lock()
                        .unwrap()
                        .remove(window.label());
                    if !expected {
                        eprintln!("FAIL: unexpected window close: {}", window.label());
                        app.exit(1);
                        return;
                    }
                    let run_id = windowing::result_run_id(window.label()).unwrap();
                    app.state::<AppState>()
                        .runtime
                        .lock()
                        .unwrap()
                        .take_analysis(run_id);
                    let scenario = run_id
                        .strip_prefix("native-escape-")
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    println!("PASS: native close for scenario {scenario}");
                    if scenario == 2 {
                        app.state::<SmokeState>()
                            .completed
                            .store(true, Ordering::SeqCst);
                        app.exit(0);
                    } else {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(error) = open_scenario(&app, scenario + 1) {
                                eprintln!("FAIL: {error}");
                                app.exit(1);
                            }
                        });
                    }
                }
            })
            .build(context)?;
        app.run_return(|app, event| {
            if let RunEvent::ExitRequested {
                code: None, api, ..
            } = &event
            {
                api.prevent_exit();
            }
            if matches!(event, RunEvent::Exit)
                && app
                    .state::<SmokeState>()
                    .escape_held
                    .swap(false, Ordering::SeqCst)
            {
                // Release only the key held by this test, including failure exits.
                unsafe {
                    SendInput(
                        &[key_input(VK_ESCAPE, true)],
                        std::mem::size_of::<INPUT>() as i32,
                    )
                };
            }
        });
        Ok(if completed.load(Ordering::SeqCst) {
            0
        } else {
            1
        })
    }
}

#[cfg(target_os = "windows")]
fn main() {
    match smoke::run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("FAIL: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("Native Escape smoke requires Windows and WebView2.");
    std::process::exit(1);
}
