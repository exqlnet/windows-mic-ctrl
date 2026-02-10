#[cfg(target_os = "windows")]
use std::process::Command;

#[cfg(target_os = "windows")]
use cpal::traits::{DeviceTrait, HostTrait};

use crate::{error::AppError, types::VirtualMicStatus};

#[cfg(target_os = "windows")]
const VIRTUAL_MIC_NAME_HINTS: &[&str] = &[
    "windows mic ctrl virtual mic",
    "windowsmicctrl virtual mic",
    "wmc virtual mic",
];

#[cfg(target_os = "windows")]
const DRIVER_SERVICE_NAME: &str = "windows_mic_ctrl_virtual_mic";

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DriverServiceState {
    Running,
    Stopped,
    NotFound,
    Unknown,
}

#[cfg(target_os = "windows")]
fn detect_virtual_capture_device() -> Result<Option<String>, AppError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| AppError::Audio(format!("枚举录制设备失败: {e}")))?;

    for device in devices {
        let name = device.name().unwrap_or_else(|_| "未知录制设备".to_string());
        let lower = name.to_ascii_lowercase();
        if VIRTUAL_MIC_NAME_HINTS
            .iter()
            .any(|hint| lower.contains(hint))
        {
            return Ok(Some(name));
        }
    }

    Ok(None)
}

#[cfg(target_os = "windows")]
fn query_driver_service_state() -> (DriverServiceState, String) {
    match Command::new("sc")
        .args(["query", DRIVER_SERVICE_NAME])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
            let text = format!("{stdout}\n{stderr}");

            if text.contains("failed 1060") || text.contains("does not exist") {
                return (DriverServiceState::NotFound, "未安装".to_string());
            }

            if text.contains("running") {
                return (DriverServiceState::Running, "运行中".to_string());
            }

            if text.contains("stopped") || text.contains("stop pending") {
                return (DriverServiceState::Stopped, "已安装但未运行".to_string());
            }

            if output.status.success() {
                return (
                    DriverServiceState::Unknown,
                    "已注册（状态未知）".to_string(),
                );
            }

            (
                DriverServiceState::Unknown,
                "查询失败（请检查管理员权限）".to_string(),
            )
        }
        Err(e) => (DriverServiceState::Unknown, format!("查询失败：{e}")),
    }
}

#[cfg(target_os = "windows")]
fn query_test_signing_enabled() -> Result<bool, String> {
    match Command::new("bcdedit")
        .args(["/enum", "{current}"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            Ok(text.contains("testsigning") && text.contains("yes"))
        }
        Ok(output) => Err(format!(
            "bcdedit 返回非零状态（{}）",
            output.status.code().unwrap_or(-1)
        )),
        Err(e) => Err(format!("调用 bcdedit 失败：{e}")),
    }
}

pub fn initialize() -> Result<VirtualMicStatus, AppError> {
    #[cfg(target_os = "windows")]
    {
        let endpoint = detect_virtual_capture_device()?;
        let (service_state, service_text) = query_driver_service_state();
        let testsigning = query_test_signing_enabled();

        if let Some(name) = endpoint {
            let test_mode = match testsigning {
                Ok(true) => "已启用",
                Ok(false) => "未启用",
                Err(_) => "未知",
            };

            return Ok(VirtualMicStatus {
                backend: "windows-kernel-driver".to_string(),
                ready: true,
                detail: format!(
                    "已检测到虚拟麦录制端点：{name}。驱动服务：{service_text}。Test Mode：{test_mode}。"
                ),
            });
        }

        let mut hints = Vec::new();
        hints.push("系统录制设备中未检测到“Windows Mic Ctrl Virtual Mic”端点。".to_string());

        match service_state {
            DriverServiceState::NotFound => {
                hints.push(format!("驱动服务 {DRIVER_SERVICE_NAME} 未安装。"));
            }
            DriverServiceState::Stopped => {
                hints.push(format!("驱动服务 {DRIVER_SERVICE_NAME} 已安装但未运行。"));
            }
            DriverServiceState::Running => {
                hints.push(format!(
                    "驱动服务 {DRIVER_SERVICE_NAME} 运行中，但仍未暴露录制端点；请检查 SysVAD 派生代码与 INF。"
                ));
            }
            DriverServiceState::Unknown => {
                hints.push(format!(
                    "驱动服务 {DRIVER_SERVICE_NAME} 状态未知：{service_text}。"
                ));
            }
        }

        match testsigning {
            Ok(true) => {}
            Ok(false) => {
                hints.push("若使用测试签名驱动，请先启用 Test Mode 并重启系统。".to_string())
            }
            Err(e) => hints.push(format!("无法确认 Test Mode 状态：{e}。")),
        }

        hints.push(
            "建议在管理员 PowerShell 依次执行：driver/windows/scripts/dev-bootstrap.ps1 -> driver/windows/scripts/build-driver.ps1 -> driver/windows/scripts/install-driver-test.ps1。"
                .to_string(),
        );

        return Ok(VirtualMicStatus {
            backend: "windows-kernel-driver".to_string(),
            ready: false,
            detail: hints.join(" "),
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(VirtualMicStatus {
            backend: "unsupported-platform".to_string(),
            ready: false,
            detail: "当前平台不支持 Windows 虚拟麦驱动。".to_string(),
        })
    }
}
