use std::{fs, path::PathBuf};

use crate::{error::AppError, types::AppConfig};

const CONFIG_FILE_NAME: &str = "config.json";

fn config_dir() -> Result<PathBuf, AppError> {
    let base = dirs::config_dir().ok_or_else(|| AppError::Config("无法定位配置目录".to_string()))?;
    Ok(base.join("windows-mic-ctrl"))
}

pub fn load_config() -> Result<AppConfig, AppError> {
    let dir = config_dir()?;
    let path = dir.join(CONFIG_FILE_NAME);
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::Config(format!("读取配置文件失败: {e}")))?;
    serde_json::from_str::<AppConfig>(&content)
        .map_err(|e| AppError::Config(format!("解析配置文件失败: {e}")))
}

pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    let dir = config_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::Config(format!("创建配置目录失败: {e}")))?;
    }

    let path = dir.join(CONFIG_FILE_NAME);
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("序列化配置失败: {e}")))?;
    fs::write(path, content).map_err(|e| AppError::Config(format!("写入配置失败: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.hotkey.accelerator, "Ctrl+Shift+V");
    }
}
