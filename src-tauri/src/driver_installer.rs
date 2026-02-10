#[cfg(target_os = "windows")]
mod imp {
    use std::{
        path::{Path, PathBuf},
        process::Command,
        thread,
        time::Duration,
    };

    use cpal::traits::{DeviceTrait, HostTrait};
    use tauri::{path::BaseDirectory, AppHandle, Manager};

    use crate::error::AppError;

    const VIRTUAL_MIC_NAME_HINTS: &[&str] = &[
        "windows mic ctrl virtual mic",
        "windowsmicctrl virtual mic",
        "wmc virtual mic",
    ];

    fn has_virtual_capture_endpoint() -> Result<bool, AppError> {
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
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn candidate_driver_dirs(app: &AppHandle) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Ok(path) = app
            .path()
            .resolve("drivers/windows", BaseDirectory::Resource)
        {
            dirs.push(path);
        }

        if let Ok(current) = std::env::current_dir() {
            dirs.push(current.join("src-tauri").join("drivers").join("windows"));
            dirs.push(current.join("drivers").join("windows"));
            dirs.push(
                current
                    .join("..")
                    .join("src-tauri")
                    .join("drivers")
                    .join("windows"),
            );
        }

        let mut uniq = Vec::new();
        for path in dirs {
            if uniq.iter().all(|p: &PathBuf| p != &path) {
                uniq.push(path);
            }
        }

        uniq
    }

    fn resolve_bundled_inf(app: &AppHandle) -> Result<PathBuf, AppError> {
        for directory in candidate_driver_dirs(app) {
            if !directory.exists() {
                continue;
            }

            let mut inf_candidates = Vec::new();
            let entries = std::fs::read_dir(&directory).map_err(|e| {
                AppError::System(format!("读取驱动目录失败({}): {e}", directory.display()))
            })?;

            for entry in entries.flatten() {
                let path = entry.path();
                let is_inf = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("inf"))
                    .unwrap_or(false);

                if is_inf {
                    inf_candidates.push(path);
                }
            }

            inf_candidates.sort();

            if let Some(preferred) = inf_candidates.iter().find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_ascii_lowercase().contains("windows-mic-ctrl"))
                    .unwrap_or(false)
            }) {
                return Ok(preferred.clone());
            }

            if let Some(first) = inf_candidates.into_iter().next() {
                return Ok(first);
            }
        }

        Err(AppError::System(
            "未在安装包资源中找到可用驱动 INF（drivers/windows/*.inf）".to_string(),
        ))
    }

    fn install_driver_with_uac(inf_path: &Path) -> Result<(), AppError> {
        let inf = inf_path
            .to_string_lossy()
            .replace('"', "`\"")
            .replace('`', "``");

        let script = format!(
            "$ErrorActionPreference='Stop'; \
             $inf=\"{inf}\"; \
             if (-not (Test-Path $inf)) {{ throw \"INF not found: $inf\" }}; \
             $p=Start-Process -FilePath pnputil -ArgumentList @('/add-driver',$inf,'/install') -Verb RunAs -Wait -PassThru; \
             if ($null -eq $p) {{ throw 'pnputil 未返回进程对象' }}; \
             exit $p.ExitCode"
        );

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .output()
            .map_err(|e| AppError::System(format!("调用 powershell 失败: {e}")))?;

        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(AppError::System(format!(
            "驱动安装失败（可能拒绝了 UAC）：exit={:?}, stdout={}, stderr={}",
            output.status.code(),
            stdout,
            stderr
        )))
    }

    pub fn ensure_driver_installed(app: &AppHandle) -> Result<String, AppError> {
        if has_virtual_capture_endpoint()? {
            return Ok("检测到虚拟麦已安装，跳过安装。".to_string());
        }

        let inf_path = resolve_bundled_inf(app)?;
        install_driver_with_uac(&inf_path)?;

        for _ in 0..10 {
            thread::sleep(Duration::from_millis(700));
            if has_virtual_capture_endpoint()? {
                return Ok(format!(
                    "已通过安装包内驱动完成安装：{}",
                    inf_path.display()
                ));
            }
        }

        Err(AppError::System(
            "驱动安装命令已执行，但仍未检测到虚拟麦录制端点（可能需要重启系统）".to_string(),
        ))
    }
}

#[cfg(target_os = "windows")]
pub use imp::ensure_driver_installed;

#[cfg(not(target_os = "windows"))]
pub fn ensure_driver_installed(_app: &tauri::AppHandle) -> Result<String, crate::error::AppError> {
    Err(crate::error::AppError::System(
        "当前平台不支持自动安装 Windows 虚拟麦驱动".to_string(),
    ))
}
