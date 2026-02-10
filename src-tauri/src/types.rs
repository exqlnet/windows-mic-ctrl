use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub direction: DeviceDirection,
    pub is_default: bool,
    pub is_virtual_candidate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceList {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GateMode {
    Ptt,
    Toggle,
    Hybrid,
}

impl Default for GateMode {
    fn default() -> Self {
        Self::Ptt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub accelerator: String,
    pub mode: GateMode,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            accelerator: "Ctrl+Shift+V".to_string(),
            mode: GateMode::Ptt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioRouteConfig {
    pub input_device_id: String,
    pub bridge_output_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub route: AudioRouteConfig,
    pub hotkey: HotkeyConfig,
    pub launch_on_startup: bool,
    pub minimize_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            route: AudioRouteConfig::default(),
            hotkey: HotkeyConfig::default(),
            launch_on_startup: false,
            minimize_to_tray: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateState {
    pub is_open: bool,
    pub mode: GateMode,
    pub last_source: String,
    pub changed_at: DateTime<Utc>,
}

impl Default for GateState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: GateMode::Ptt,
            last_source: "system".to_string(),
            changed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EngineState {
    Idle,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub engine_state: EngineState,
    pub buffer_level_ms: u32,
    pub xruns: u64,
    pub last_error: Option<String>,
    pub gate_state: GateState,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            engine_state: EngineState::Idle,
            buffer_level_ms: 0,
            xruns: 0,
            last_error: None,
            gate_state: GateState::default(),
        }
    }
}
