use cpal::traits::{DeviceTrait, HostTrait};

use crate::{
    error::AppError,
    types::{DeviceDirection, DeviceInfo, DeviceList},
};

fn host() -> cpal::Host {
    cpal::default_host()
}

fn is_virtual_candidate(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ["cable", "vb-audio", "virtual", "voiceemeeter"]
        .iter()
        .any(|k| lower.contains(k))
}

fn make_device_id(direction: &DeviceDirection, index: usize, name: &str) -> String {
    let d = match direction {
        DeviceDirection::Input => "in",
        DeviceDirection::Output => "out",
    };
    format!("{}#{}#{}", d, index, name)
}

pub fn list_devices() -> Result<DeviceList, AppError> {
    let host = host();

    let default_input_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();
    let default_output_name = host
        .default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let mut inputs = Vec::new();
    for (index, dev) in host
        .input_devices()
        .map_err(|e| AppError::Audio(format!("获取输入设备失败: {e}")))?
        .enumerate()
    {
        let name = dev.name().unwrap_or_else(|_| "未知输入设备".to_string());
        inputs.push(DeviceInfo {
            id: make_device_id(&DeviceDirection::Input, index, &name),
            name: name.clone(),
            direction: DeviceDirection::Input,
            is_default: name == default_input_name,
            is_virtual_candidate: is_virtual_candidate(&name),
        });
    }

    let mut outputs = Vec::new();
    for (index, dev) in host
        .output_devices()
        .map_err(|e| AppError::Audio(format!("获取输出设备失败: {e}")))?
        .enumerate()
    {
        let name = dev.name().unwrap_or_else(|_| "未知输出设备".to_string());
        outputs.push(DeviceInfo {
            id: make_device_id(&DeviceDirection::Output, index, &name),
            name: name.clone(),
            direction: DeviceDirection::Output,
            is_default: name == default_output_name,
            is_virtual_candidate: is_virtual_candidate(&name),
        });
    }

    Ok(DeviceList { inputs, outputs })
}

#[cfg(target_os = "windows")]
fn resolve_device(device_id: &str, direction: DeviceDirection) -> Result<cpal::Device, AppError> {
    let host = host();
    let parts: Vec<&str> = device_id.split('#').collect();
    if parts.len() < 3 {
        return Err(AppError::InvalidArgument(format!(
            "设备 ID 格式非法: {device_id}"
        )));
    }

    let idx: usize = parts[1]
        .parse()
        .map_err(|_| AppError::InvalidArgument(format!("设备 ID 索引非法: {device_id}")))?;
    let name = parts[2..].join("#");

    let mut devices = match direction {
        DeviceDirection::Input => host
            .input_devices()
            .map_err(|e| AppError::Audio(format!("读取输入设备失败: {e}")))?
            .collect::<Vec<_>>(),
        DeviceDirection::Output => host
            .output_devices()
            .map_err(|e| AppError::Audio(format!("读取输出设备失败: {e}")))?
            .collect::<Vec<_>>(),
    };

    if idx < devices.len() {
        let candidate = devices.swap_remove(idx);
        let candidate_name = candidate.name().unwrap_or_default();
        if candidate_name == name {
            return Ok(candidate);
        }
    }

    match devices
        .into_iter()
        .find(|d| d.name().map(|n| n == name).unwrap_or(false))
    {
        Some(device) => Ok(device),
        None => Err(AppError::DeviceNotFound(name)),
    }
}

#[cfg(target_os = "windows")]
mod runtime_impl {
    use std::{
        collections::VecDeque,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
    };

    use cpal::{
        traits::{DeviceTrait, StreamTrait},
        Device, SampleFormat, Stream, StreamConfig,
    };
    use parking_lot::Mutex;

    use crate::{
        audio::resolve_device,
        error::AppError,
        gate::{apply_envelope, GateController},
        types::{DeviceDirection, EngineState, RuntimeStatus},
    };

    fn choose_input_config(input: &Device) -> Result<cpal::SupportedStreamConfig, AppError> {
        let configs = input
            .supported_input_configs()
            .map_err(|e| AppError::Audio(format!("读取输入配置失败: {e}")))?;
        configs
            .filter(|cfg| cfg.sample_format() == SampleFormat::F32)
            .max_by_key(|cfg| cfg.max_sample_rate().0)
            .map(|cfg| cfg.with_max_sample_rate())
            .ok_or_else(|| AppError::Audio("输入设备不支持 F32 格式".to_string()))
    }

    fn choose_output_config(
        output: &Device,
        in_cfg: &cpal::SupportedStreamConfig,
    ) -> Result<cpal::SupportedStreamConfig, AppError> {
        let target_rate = in_cfg.sample_rate().0;
        let target_channels = in_cfg.channels();

        let configs: Vec<_> = output
            .supported_output_configs()
            .map_err(|e| AppError::Audio(format!("读取输出配置失败: {e}")))?
            .collect();

        let exact = configs.iter().find(|cfg| {
            cfg.sample_format() == SampleFormat::F32
                && cfg.channels() == target_channels
                && cfg.min_sample_rate().0 <= target_rate
                && cfg.max_sample_rate().0 >= target_rate
        });

        if let Some(cfg) = exact {
            return Ok(cfg.with_sample_rate(cpal::SampleRate(target_rate)));
        }

        configs
            .into_iter()
            .find(|cfg| cfg.sample_format() == SampleFormat::F32)
            .map(|cfg| cfg.with_max_sample_rate())
            .ok_or_else(|| AppError::Audio("输出设备不支持 F32 格式".to_string()))
    }

    pub struct EngineRuntime {
        input_stream: Stream,
        output_stream: Stream,
        buffer: Arc<Mutex<VecDeque<f32>>>,
        sample_rate: u32,
        channels: u16,
        xruns: Arc<AtomicU64>,
        last_error: Arc<Mutex<Option<String>>>,
    }

    impl EngineRuntime {
        pub fn start(
            input_id: &str,
            output_id: &str,
            gate: Arc<GateController>,
        ) -> Result<Self, AppError> {
            let input = resolve_device(input_id, DeviceDirection::Input)?;
            let output = resolve_device(output_id, DeviceDirection::Output)?;

            let in_cfg = choose_input_config(&input)?;
            let out_cfg = choose_output_config(&output, &in_cfg)?;

            if in_cfg.channels() != out_cfg.channels() {
                return Err(AppError::Audio(format!(
                    "输入/输出声道不一致: {} vs {}",
                    in_cfg.channels(),
                    out_cfg.channels()
                )));
            }

            let sample_rate = out_cfg.sample_rate().0;
            let channels = out_cfg.channels();

            let buffer = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(sample_rate as usize)));
            let xruns = Arc::new(AtomicU64::new(0));
            let last_error = Arc::new(Mutex::new(None::<String>));

            let in_buffer = buffer.clone();
            let in_last_error = last_error.clone();
            let input_config: StreamConfig = in_cfg.config();
            let input_stream = input
                .build_input_stream(
                    &input_config,
                    move |data: &[f32], _| {
                        let mut queue = in_buffer.lock();
                        for s in data {
                            if queue.len() > 96_000 {
                                let _ = queue.pop_front();
                            }
                            queue.push_back(*s);
                        }
                    },
                    move |err| {
                        *in_last_error.lock() = Some(format!("输入流错误: {err}"));
                    },
                    None,
                )
                .map_err(|e| AppError::Audio(format!("创建输入流失败: {e}")))?;

            let out_buffer = buffer.clone();
            let out_gate = gate.clone();
            let out_xruns = xruns.clone();
            let out_last_error = last_error.clone();
            let out_sample_rate = sample_rate;
            let output_config: StreamConfig = out_cfg.config();
            let output_stream = output
                .build_output_stream(
                    &output_config,
                    move |data: &mut [f32], _| {
                        let open = out_gate.is_open();
                        let mut queue = out_buffer.lock();
                        let mut gain = if open { 1.0 } else { 0.0 };
                        apply_envelope(&mut gain, open, data.len(), out_sample_rate);

                        for sample in data.iter_mut() {
                            if open {
                                if let Some(v) = queue.pop_front() {
                                    *sample = v * gain;
                                } else {
                                    *sample = 0.0;
                                    out_xruns.fetch_add(1, Ordering::Relaxed);
                                }
                            } else {
                                *sample = 0.0;
                            }
                        }
                    },
                    move |err| {
                        *out_last_error.lock() = Some(format!("输出流错误: {err}"));
                    },
                    None,
                )
                .map_err(|e| AppError::Audio(format!("创建输出流失败: {e}")))?;

            input_stream
                .play()
                .map_err(|e| AppError::Audio(format!("启动输入流失败: {e}")))?;
            output_stream
                .play()
                .map_err(|e| AppError::Audio(format!("启动输出流失败: {e}")))?;

            Ok(Self {
                input_stream,
                output_stream,
                buffer,
                sample_rate,
                channels,
                xruns,
                last_error,
            })
        }

        pub fn status(&self, gate_state: crate::types::GateState) -> RuntimeStatus {
            let queue_len = self.buffer.lock().len() as u64;
            let samples_per_ms = (self.sample_rate as u64 * self.channels as u64) / 1000;
            let buffer_level_ms = if samples_per_ms == 0 {
                0
            } else {
                (queue_len / samples_per_ms) as u32
            };

            RuntimeStatus {
                engine_state: EngineState::Running,
                buffer_level_ms,
                xruns: self.xruns.load(Ordering::Relaxed),
                last_error: self.last_error.lock().clone(),
                gate_state,
            }
        }
    }

    impl Drop for EngineRuntime {
        fn drop(&mut self) {
            let _ = self.input_stream.pause();
            let _ = self.output_stream.pause();
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod runtime_impl {
    use std::sync::Arc;

    use crate::{
        error::AppError,
        gate::GateController,
        types::{EngineState, RuntimeStatus},
    };

    pub struct EngineRuntime;

    impl EngineRuntime {
        pub fn start(
            _input_id: &str,
            _output_id: &str,
            _gate: Arc<GateController>,
        ) -> Result<Self, AppError> {
            Err(AppError::System(
                "当前平台仅提供开发桩实现，真实音频桥接仅支持 Windows".to_string(),
            ))
        }

        pub fn status(&self, gate_state: crate::types::GateState) -> RuntimeStatus {
            RuntimeStatus {
                engine_state: EngineState::Error,
                buffer_level_ms: 0,
                xruns: 0,
                last_error: Some("非 Windows 平台未启用音频桥接".to_string()),
                gate_state,
            }
        }
    }
}

pub use runtime_impl::EngineRuntime;
