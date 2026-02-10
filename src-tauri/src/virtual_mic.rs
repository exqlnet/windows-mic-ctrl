use crate::{error::AppError, types::VirtualMicStatus};

pub fn initialize() -> Result<VirtualMicStatus, AppError> {
    #[cfg(target_os = "windows")]
    {
        return Ok(VirtualMicStatus {
            backend: "embedded-virtual-mic-bootstrap".to_string(),
            ready: true,
            detail: "虚拟麦初始化流程已启动（当前版本为驱动接入过渡阶段）。".to_string(),
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
