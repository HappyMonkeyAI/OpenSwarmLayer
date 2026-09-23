use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent,
};

#[derive(Clone, serde::Serialize)]
struct DesktopSession {
    base_url: String,
    auth_token: String,
}

struct LocalEngine {
    config: ts_daemon::RuntimeConfig,
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

fn spawn_engine(
    config: ts_daemon::RuntimeConfig,
    previous: Option<tauri::async_runtime::JoinHandle<()>>,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        if let Some(previous) = previous {
            previous.abort();
            let _ = previous.await;
        }
        if let Err(error) = ts_daemon::serve_runtime(config).await {
            eprintln!("TensorSwarm local engine stopped: {error:#}");
        }
    })
}

#[tauri::command]
fn desktop_session(session: tauri::State<'_, DesktopSession>) -> DesktopSession {
    session.inner().clone()
}

#[tauri::command]
fn restart_local_engine(engine: State<'_, LocalEngine>) -> Result<(), String> {
    let mut task = engine
        .task
        .lock()
        .map_err(|_| "Local engine supervisor is unavailable".to_string())?;
    let previous = task.take();
    *task = Some(spawn_engine(engine.config.clone(), previous));
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            desktop_session,
            restart_local_engine
        ])
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let store_root = app_data.join("cache");
            std::fs::create_dir_all(&store_root)?;
            let session = DesktopSession {
                base_url: "http://127.0.0.1:9090".into(),
                auth_token: uuid::Uuid::new_v4().to_string(),
            };
            app.manage(session.clone());
            let runtime_config = ts_daemon::RuntimeConfig {
                control_bind: "127.0.0.1:9090".into(),
                proxy_bind: "127.0.0.1:9091".into(),
                manifest_path: None,
                store_root,
                origin_url: "https://localhost/".into(),
                auth_token: session.auth_token,
            };
            app.manage(LocalEngine {
                config: runtime_config.clone(),
                task: Mutex::new(Some(spawn_engine(runtime_config, None))),
            });

            let show = MenuItem::with_id(app, "show", "Show TensorSwarm", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("TensorSwarm")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running TensorSwarm desktop application");
}
