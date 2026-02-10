#[cfg(target_os = "windows")]
use std::path::Path;

use crate::{error::AppError, types::VirtualMicStatus};

pub fn initialize() -> Result<VirtualMicStatus, AppError> {
    #[cfg(target_os = "windows")]
    {
        let driver_scaffold_exists = Path::new("driver").exists();
        let detail = if driver_scaffold_exists {
            "已检测到 driver/ 骨架目录。当前版本仅完成驱动工程骨架与接口协议，未产出可被系统识别的内核虚拟麦端点。"
        } else {
            "未检测到 driver/ 骨架目录，虚拟麦驱动尚未初始化。"
        };

        return Ok(VirtualMicStatus {
            backend: "windows-kernel-driver-skeleton".to_string(),
            ready: false,
            detail: detail.to_string(),
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
