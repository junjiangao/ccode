use std::fmt;

/// 应用程序错误类型
#[derive(Debug)]
pub enum AppError {
    /// 配置文件相关错误
    Config(String),
    /// IO操作错误
    Io(std::io::Error),
    /// JSON序列化/反序列化错误
    Json(serde_json::Error),
    /// 配置文件不存在
    ConfigNotFound,
    /// 指定的配置项不存在
    ProfileNotFound(String),
    /// 无效的配置格式
    InvalidConfig(String),
    /// 命令执行错误
    CommandExecution(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "配置错误: {msg}"),
            AppError::Io(err) => write!(f, "文件操作错误: {err}"),
            AppError::Json(err) => write!(f, "JSON格式错误: {err}"),
            AppError::ConfigNotFound => {
                write!(f, "配置文件不存在，请使用 'ccode add <name>' 添加配置")
            }
            AppError::ProfileNotFound(name) => {
                write!(f, "配置 '{name}' 不存在，请使用 'ccode list' 查看可用配置")
            }
            AppError::InvalidConfig(msg) => write!(f, "无效配置: {msg}"),
            AppError::CommandExecution(msg) => write!(f, "命令执行失败: {msg}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err)
    }
}

/// 应用程序结果类型别名
pub type AppResult<T> = Result<T, AppError>;
