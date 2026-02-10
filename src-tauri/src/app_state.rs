use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use parking_lot::Mutex;

use crate::{
    audio::{complete_route_defaults, list_devices, EngineRuntime},
    config,
    error::AppError,
    gate::GateController,
    hotkey::HotkeyManager,
    types::{AppConfig, AudioRouteConfig, EngineState, GateState, RuntimeStatus, VirtualMicStatus},
    virtual_mic,
};

struct EngineWorker {
    stop_tx: mpsc::Sender<()>,
    join_handle: Option<thread::JoinHandle<()>>,
    snapshot: Arc<Mutex<RuntimeStatus>>,
}

impl EngineWorker {
    fn new(
        input_device_id: String,
        output_device_id: String,
        gate: Arc<GateController>,
    ) -> Result<Self, AppError> {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (started_tx, started_rx) = mpsc::sync_channel::<Result<(), String>>(1);
        let snapshot = Arc::new(Mutex::new(RuntimeStatus {
            engine_state: EngineState::Idle,
            buffer_level_ms: 0,
            xruns: 0,
            last_error: None,
            gate_state: gate.snapshot(),
        }));

        let snapshot_for_thread = snapshot.clone();
        let join_handle = thread::spawn(move || {
            let runtime =
                match EngineRuntime::start(&input_device_id, &output_device_id, gate.clone()) {
                    Ok(runtime) => {
                        {
                            let mut status = snapshot_for_thread.lock();
                            status.engine_state = EngineState::Running;
                            status.last_error = None;
                            status.gate_state = gate.snapshot();
                        }
                        let _ = started_tx.send(Ok(()));
                        runtime
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        {
                            let mut status = snapshot_for_thread.lock();
                            status.engine_state = EngineState::Error;
                            status.last_error = Some(msg.clone());
                            status.gate_state = gate.snapshot();
                        }
                        let _ = started_tx.send(Err(msg));
                        return;
                    }
                };

            loop {
                {
                    let mut status = snapshot_for_thread.lock();
                    *status = runtime.status(gate.snapshot());
                }

                match stop_rx.recv_timeout(Duration::from_millis(150)) {
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }

            let mut status = snapshot_for_thread.lock();
            status.engine_state = EngineState::Idle;
        });

        match started_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self {
                stop_tx,
                join_handle: Some(join_handle),
                snapshot,
            }),
            Ok(Err(msg)) => {
                let _ = join_handle.join();
                Err(AppError::Audio(msg))
            }
            Err(_) => {
                let _ = stop_tx.send(());
                let _ = join_handle.join();
                Err(AppError::Audio("启动音频引擎超时".to_string()))
            }
        }
    }

    fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }

    fn status(&self, gate_state: GateState) -> RuntimeStatus {
        let mut status = self.snapshot.lock().clone();
        status.gate_state = gate_state;
        status
    }
}

pub struct AppState {
    pub gate: Arc<GateController>,
    pub hotkey: HotkeyManager,
    config: Mutex<AppConfig>,
    engine: Mutex<Option<EngineWorker>>,
    last_error: Mutex<Option<String>>,
    virtual_mic_status: Mutex<VirtualMicStatus>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let cfg = config::load_config()?;
        let gate = Arc::new(GateController::new(cfg.hotkey.mode.clone()));
        let vm_status = virtual_mic::initialize().unwrap_or_else(|e| VirtualMicStatus {
            backend: "windows-kernel-driver-skeleton".to_string(),
            ready: false,
            detail: format!("虚拟麦后端初始化失败: {e}"),
        });

        let state = Self {
            gate,
            hotkey: HotkeyManager::default(),
            config: Mutex::new(cfg),
            engine: Mutex::new(None),
            last_error: Mutex::new(None),
            virtual_mic_status: Mutex::new(vm_status),
        };

        if let Err(e) = state.ensure_route_defaults() {
            state.set_last_error(format!("初始化设备失败: {e}"));
        }

        Ok(state)
    }

    pub fn config(&self) -> AppConfig {
        self.config.lock().clone()
    }

    pub fn set_route(&self, mut route: AudioRouteConfig) -> Result<(), AppError> {
        let mut cfg = self.config.lock();

        if route.bridge_output_device_id.is_empty() {
            route.bridge_output_device_id = cfg.route.bridge_output_device_id.clone();
        }

        cfg.route = route;
        complete_route_defaults(&mut cfg.route)?;
        config::save_config(&cfg)
    }

    pub fn ensure_route_defaults(&self) -> Result<(), AppError> {
        let mut cfg = self.config.lock();
        let before = cfg.route.clone();
        complete_route_defaults(&mut cfg.route)?;
        if cfg.route != before {
            config::save_config(&cfg)?;
        }
        Ok(())
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

        let mut cfg = self.config.lock().clone();
        complete_route_defaults(&mut cfg.route)?;

        if cfg.route.input_device_id.is_empty() || cfg.route.bridge_output_device_id.is_empty() {
            return Err(AppError::InvalidArgument(
                "请先选择物理麦克风输入设备，并确保虚拟麦端点可用".to_string(),
            ));
        }

        let worker = EngineWorker::new(
            cfg.route.input_device_id.clone(),
            cfg.route.bridge_output_device_id.clone(),
            self.gate.clone(),
        )?;
        *self.engine.lock() = Some(worker);
        *self.last_error.lock() = None;

        Ok(())
    }

    pub fn stop_engine(&self) {
        if let Some(worker) = self.engine.lock().take() {
            worker.stop();
        }
    }

    pub fn set_last_error(&self, msg: impl Into<String>) {
        *self.last_error.lock() = Some(msg.into());
    }

    pub fn gate_snapshot(&self) -> GateState {
        self.gate.snapshot()
    }

    pub fn virtual_mic_status(&self) -> VirtualMicStatus {
        self.virtual_mic_status.lock().clone()
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
        if cfg.route.input_device_id.is_empty() && cfg.route.bridge_output_device_id.is_empty() {
            return Ok(());
        }

        let devices = list_devices()?;
        let in_ok = devices
            .inputs
            .iter()
            .any(|d| d.id == cfg.route.input_device_id);
        let out_ok = devices
            .outputs
            .iter()
            .any(|d| d.id == cfg.route.bridge_output_device_id);

        if !in_ok || !out_ok {
            return Err(AppError::DeviceNotFound(
                "历史配置中的音频设备已不可用，请重新选择输入设备".to_string(),
            ));
        }

        Ok(())
    }
}
