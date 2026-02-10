use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    audio::{list_devices, EngineRuntime},
    config,
    error::AppError,
    gate::GateController,
    hotkey::HotkeyManager,
    types::{AppConfig, AudioRouteConfig, EngineState, GateState, RuntimeStatus},
};

pub struct AppState {
    pub gate: Arc<GateController>,
    pub hotkey: HotkeyManager,
    config: Mutex<AppConfig>,
    engine: Mutex<Option<EngineRuntime>>,
    last_error: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let cfg = config::load_config()?;
        let gate = Arc::new(GateController::new(cfg.hotkey.mode.clone()));
        Ok(Self {
            gate,
            hotkey: HotkeyManager::default(),
            config: Mutex::new(cfg),
            engine: Mutex::new(None),
            last_error: Mutex::new(None),
        })
    }

    pub fn config(&self) -> AppConfig {
        self.config.lock().clone()
    }

    pub fn set_route(&self, route: AudioRouteConfig) -> Result<(), AppError> {
        let mut cfg = self.config.lock();
        cfg.route = route;
        config::save_config(&cfg)
    }

    pub fn set_hotkey_config(&self, hotkey: crate::types::HotkeyConfig) -> Result<(), AppError> {
        let mut cfg = self.config.lock();
        cfg.hotkey = hotkey;
        self.gate.set_mode(cfg.hotkey.mode.clone());
        config::save_config(&cfg)
    }

    pub fn set_launch_on_startup(&self, enabled: bool) -> Result<(), AppError> {
        let mut cfg = self.config.lock();
        cfg.launch_on_startup = enabled;
        config::save_config(&cfg)
    }

    pub fn set_minimize_to_tray(&self, enabled: bool) -> Result<(), AppError> {
        let mut cfg = self.config.lock();
        cfg.minimize_to_tray = enabled;
        config::save_config(&cfg)
    }

    pub fn start_engine(&self) -> Result<(), AppError> {
        if self.engine.lock().is_some() {
            return Ok(());
        }

        let cfg = self.config.lock().clone();
        if cfg.route.input_device_id.is_empty() || cfg.route.bridge_output_device_id.is_empty() {
            return Err(AppError::InvalidArgument(
                "请先选择输入设备与桥接输出设备".to_string(),
            ));
        }

        let runtime = EngineRuntime::start(
            &cfg.route.input_device_id,
            &cfg.route.bridge_output_device_id,
            self.gate.clone(),
        )?;
        *self.engine.lock() = Some(runtime);
        *self.last_error.lock() = None;

        Ok(())
    }

    pub fn stop_engine(&self) {
        let _ = self.engine.lock().take();
    }

    pub fn set_last_error(&self, msg: impl Into<String>) {
        *self.last_error.lock() = Some(msg.into());
    }

    pub fn gate_snapshot(&self) -> GateState {
        self.gate.snapshot()
    }

    pub fn runtime_status(&self) -> RuntimeStatus {
        let gate_state = self.gate.snapshot();
        if let Some(engine) = self.engine.lock().as_ref() {
            return engine.status(gate_state);
        }

        RuntimeStatus {
            engine_state: if self.last_error.lock().is_some() {
                EngineState::Error
            } else {
                EngineState::Idle
            },
            buffer_level_ms: 0,
            xruns: 0,
            last_error: self.last_error.lock().clone(),
            gate_state,
        }
    }

    pub fn validate_route_exists(&self) -> Result<(), AppError> {
        let cfg = self.config.lock().clone();
        let devices = list_devices()?;
        let in_ok = devices.inputs.iter().any(|d| d.id == cfg.route.input_device_id);
        let out_ok = devices
            .outputs
            .iter()
            .any(|d| d.id == cfg.route.bridge_output_device_id);

        if !in_ok || !out_ok {
            return Err(AppError::DeviceNotFound(
                "历史配置中的音频设备已不可用，请重新选择".to_string(),
            ));
        }

        Ok(())
    }
}
