use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("参数无效: {0}")]
    InvalidArgument(String),
    #[error("未找到设备: {0}")]
    DeviceNotFound(String),
    #[error("音频错误: {0}")]
    Audio(String),
    #[error("配置错误: {0}")]
    Config(String),
    #[error("快捷键错误: {0}")]
    Hotkey(String),
    #[error("系统错误: {0}")]
    System(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
