use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Manager,
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

fn build_menu(
    app: &tauri::AppHandle,
    status: &RuntimeStatus,
) -> Result<tauri::menu::Menu<tauri::Wry>, AppError> {
    let state_text = match status.engine_state {
        EngineState::Idle => "语音链路：未就绪",
        EngineState::Running => "语音链路：已就绪",
        EngineState::Error => "语音链路：错误",
    };

    let engine_status = MenuItem::with_id(app, "engine_status", state_text, false, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;

    let show_main = MenuItem::with_id(app, "show_main", "显示主界面", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let restart_engine = MenuItem::with_id(
        app,
        "restart_engine",
        "重新初始化语音链路",
        true,
        None::<&str>,
    )
    .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| AppError::System(format!("创建菜单失败: {e}")))?;

    MenuBuilder::new(app)
        .item(&engine_status)
        .separator()
        .item(&show_main)
        .item(&restart_engine)
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
        "restart_engine" => {
            state.inner().stop_engine();
            if let Err(e) = state.inner().start_engine() {
                log::error!("托盘重新初始化语音链路失败: {e}");
                state
                    .inner()
                    .set_last_error(format!("托盘重新初始化失败: {e}"));
            }
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
