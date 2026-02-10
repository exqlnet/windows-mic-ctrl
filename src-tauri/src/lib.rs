mod app_state;
mod audio;
mod commands;
mod config;
mod error;
mod gate;
mod hotkey;
mod mouse_hook;
mod tray;
mod types;
mod virtual_mic;

use tauri::{Emitter, Manager};

use app_state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let state = AppState::new().map_err(|e| e.to_string())?;

            if let Err(e) = state.validate_route_exists() {
                log::warn!("历史设备校验失败: {e}");
            }

            let cfg = state.config();
            state
                .hotkey
                .apply(app.handle(), &cfg.hotkey, state.gate.clone())
                .map_err(|e| e.to_string())?;

            if let Err(e) = state.start_engine() {
                log::warn!("启动自动初始化失败: {e}");
            }

            app.manage(state);
            tray::create_tray(app.handle()).map_err(|e| e.to_string())?;

            if cfg.launch_on_startup {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.handle().autolaunch().enable();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.inner().config().minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    {
                        let _ = window.set_skip_taskbar(true);
                    }
                    let _ = window.app_handle().emit("window_hidden_to_tray", true);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_audio_devices,
            commands::get_app_config,
            commands::save_audio_route,
            commands::set_hotkey,
            commands::start_engine,
            commands::stop_engine,
            commands::set_mic_gate,
            commands::get_runtime_status,
            commands::get_virtual_mic_status,
            commands::set_launch_on_startup,
            commands::set_minimize_to_tray,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
