use tauri::Emitter;

use crate::{
    app_state::AppState,
    audio,
    error::AppError,
    types::{AppConfig, AudioRouteConfig, HotkeyConfig, RuntimeStatus},
};

#[tauri::command]
pub fn list_audio_devices() -> Result<crate::types::DeviceList, AppError> {
    audio::list_devices()
}

#[tauri::command]
pub fn get_app_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, AppError> {
    Ok(state.inner().config())
}

#[tauri::command]
pub fn save_audio_route(
    state: tauri::State<'_, AppState>,
    config: AudioRouteConfig,
) -> Result<(), AppError> {
    state.inner().set_route(config)
}

#[tauri::command]
pub fn set_hotkey(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: HotkeyConfig,
) -> Result<(), AppError> {
    state.inner().set_hotkey_config(config.clone())?;
    state
        .inner()
        .hotkey
        .apply(&app, &config, state.inner().gate.clone())?;
    Ok(())
}

#[tauri::command]
pub fn start_engine(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    state.inner().start_engine()?;
    app.emit("engine_state_changed", state.inner().runtime_status())
        .map_err(|e| AppError::System(format!("发射事件失败: {e}")))?;
    Ok(())
}

#[tauri::command]
pub fn stop_engine(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    state.inner().stop_engine();
    app.emit("engine_state_changed", state.inner().runtime_status())
        .map_err(|e| AppError::System(format!("发射事件失败: {e}")))?;
    Ok(())
}

#[tauri::command]
pub fn set_mic_gate(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    open: bool,
    source: String,
) -> Result<(), AppError> {
    state.inner().gate.set_open(open, &source);
    app.emit("gate_state_changed", state.inner().gate_snapshot())
        .map_err(|e| AppError::System(format!("发射事件失败: {e}")))?;
    Ok(())
}

#[tauri::command]
pub fn get_runtime_status(
    state: tauri::State<'_, AppState>,
) -> Result<RuntimeStatus, AppError> {
    Ok(state.inner().runtime_status())
}

#[tauri::command]
pub fn set_launch_on_startup(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), AppError> {
    use tauri_plugin_autostart::ManagerExt;

    if enabled {
        app.autolaunch()
            .enable()
            .map_err(|e| AppError::System(format!("启用自启动失败: {e}")))?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|e| AppError::System(format!("关闭自启动失败: {e}")))?;
    }

    state.inner().set_launch_on_startup(enabled)
}

#[tauri::command]
pub fn set_minimize_to_tray(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), AppError> {
    state.inner().set_minimize_to_tray(enabled)
}
