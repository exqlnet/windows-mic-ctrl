use std::{str::FromStr, sync::Arc};

use parking_lot::RwLock;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{
    error::AppError,
    gate::GateController,
    types::{GateMode, HotkeyConfig},
};

#[derive(Default)]
pub struct HotkeyManager {
    current: RwLock<Option<String>>,
}

impl HotkeyManager {
    pub fn apply(
        &self,
        app: &tauri::AppHandle,
        config: &HotkeyConfig,
        gate: Arc<GateController>,
    ) -> Result<(), AppError> {
        let manager = app.global_shortcut();

        if let Some(current) = self.current.read().clone() {
            if let Ok(old_shortcut) = Shortcut::from_str(&current) {
                let _ = manager.unregister(old_shortcut);
            }
        }

        let shortcut = Shortcut::from_str(&config.accelerator)
            .map_err(|e| AppError::Hotkey(format!("快捷键格式错误: {e}")))?;

        manager
            .on_shortcut(shortcut, {
                let app = app.clone();
                let mode = config.mode.clone();
                move |_app, _shortcut, event| {
                    handle_event(&app, &gate, &mode, event.state);
                }
            })
            .map_err(|e| AppError::Hotkey(format!("注册快捷键失败: {e}")))?;

        *self.current.write() = Some(config.accelerator.clone());
        Ok(())
    }
}

fn handle_event(app: &tauri::AppHandle, gate: &GateController, mode: &GateMode, state: ShortcutState) {
    match mode {
        GateMode::Ptt => {
            if state == ShortcutState::Pressed {
                gate.set_open(true, "hotkey");
            } else if state == ShortcutState::Released {
                gate.set_open(false, "hotkey");
            }
        }
        GateMode::Toggle => {
            if state == ShortcutState::Pressed {
                gate.toggle("hotkey");
            }
        }
        GateMode::Hybrid => {
            if state == ShortcutState::Pressed {
                gate.toggle("hotkey");
            }
        }
    }

    let _ = app.emit("gate_state_changed", gate.snapshot());
}
