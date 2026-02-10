export type DeviceDirection = 'input' | 'output';

export interface DeviceInfo {
  id: string;
  name: string;
  direction: DeviceDirection;
  is_default: boolean;
  is_virtual_candidate: boolean;
}

export interface DeviceList {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
}

export type GateMode = 'ptt' | 'toggle' | 'hybrid';

export interface HotkeyConfig {
  accelerator: string;
  mode: GateMode;
}

export interface AudioRouteConfig {
  input_device_id: string;
  bridge_output_device_id: string;
}

export interface GateState {
  is_open: boolean;
  mode: GateMode;
  last_source: string;
  changed_at: string;
}

export type EngineState = 'idle' | 'running' | 'error';

export interface RuntimeStatus {
  engine_state: EngineState;
  buffer_level_ms: number;
  xruns: number;
  last_error: string | null;
  gate_state: GateState;
}

export interface VirtualMicStatus {
  backend: string;
  ready: boolean;
  detail: string;
}

export interface AppConfig {
  route: AudioRouteConfig;
  hotkey: HotkeyConfig;
  launch_on_startup: boolean;
  minimize_to_tray: boolean;
}
