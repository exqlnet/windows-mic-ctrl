use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

use crate::{
    app_state::AppState,
    error::AppError,
    types::{EngineState, RuntimeStatus},
};

pub fn create_tray(app: &tauri::AppHandle) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let status = state.inner().runtime_status();
    let menu = build_menu(app, &status)?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .build(app)
        .map_err(|e| AppError::System(format!("创建托盘失败: {e}")))?;

    Ok(())
}

fn build_menu(app: &tauri::AppHandle, status: &RuntimeStatus) -> Result<tauri::menu::Menu<tauri::Wry>, AppError> {
    let gate_text = if status.gate_state.is_open { "当前状态：开麦" } else { "当前状态：闭麦" };
    let state_text = match status.engine_state {
        EngineState::Idle => "引擎：未启动",
        EngineState::Running => "引擎：运行中",
        EngineState::Error => "引擎：错误",
    };

    let gate_status = MenuItem::with_id(app, "gate_status", gate_text, false, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let engine_status = MenuItem::with_id(app, "engine_status", state_text, false, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;

    let show_main = MenuItem::with_id(app, "show_main", "显示主界面", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let toggle_gate = MenuItem::with_id(app, "toggle_gate", "切换开麦/闭麦", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let start_engine = MenuItem::with_id(app, "start_engine", "启动桥接", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let stop_engine = MenuItem::with_id(app, "stop_engine", "停止桥接", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;

    MenuBuilder::new(app)
        .item(&gate_status)
        .item(&engine_status)
        .separator()
        .item(&show_main)
        .item(&toggle_gate)
        .item(&start_engine)
        .item(&stop_engine)
        .separator()
        .item(&quit)
        .build()
        .map_err(|e| AppError::System(format!("构建托盘菜单失败: {e}")))
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let state = app.state::<AppState>();
    match event.id.0.as_str() {
        "show_main" => {
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    let _ = window.set_skip_taskbar(false);
                }
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "toggle_gate" => {
            state.inner().gate.toggle("tray");
            let _ = app.emit("gate_state_changed", state.inner().gate.snapshot());
        }
        "start_engine" => {
            if let Err(e) = state.inner().start_engine() {
                log::error!("托盘启动引擎失败: {e}");
            }
        }
        "stop_engine" => {
            state.inner().stop_engine();
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }

    let latest = state.inner().runtime_status();
    if let Ok(menu) = build_menu(app, &latest) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
