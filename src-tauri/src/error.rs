//! Centralized error handling for the application

use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 解析失败: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("系统命令执行失败: {0}")]
    Command(String),

    #[error("热键错误: {0}")]
    Hotkey(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("键盘配置错误: {0}")]
    Keyboard(String),

    #[error("验证失败: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("此功能不支持当前平台: {0}")]
    PlatformNotSupported(String),

    #[error("权限不足: {0}。请以管理员身份运行程序")]
    PermissionDenied(String),
}

impl AppError {
    /// Create a message error
    pub fn message<T: Into<String>>(msg: T) -> Self {
        AppError::Message(msg.into())
    }

    /// Create an error with context
    pub fn with_context<E: Display, T: Into<String>>(err: E, msg: T) -> Self {
        AppError::Message(format!("{}: {}", msg.into(), err))
    }

    /// Create a validation error
    pub fn validation<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        AppError::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a platform not supported error
    pub fn platform_not_supported<T: Into<String>>(feature: T) -> Self {
        AppError::PlatformNotSupported(feature.into())
    }

    /// Create a permission denied error
    pub fn permission_denied<T: Into<String>>(action: T) -> Self {
        AppError::PermissionDenied(action.into())
    }

    /// Get error code for frontend handling
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Message(_) => "MESSAGE",
            AppError::Io(_) => "IO_ERROR",
            AppError::SerdeJson(_) => "JSON_ERROR",
            AppError::Command(_) => "COMMAND_ERROR",
            AppError::Hotkey(_) => "HOTKEY_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::Keyboard(_) => "KEYBOARD_ERROR",
            AppError::Validation { .. } => "VALIDATION_ERROR",
            AppError::PlatformNotSupported(_) => "PLATFORM_NOT_SUPPORTED",
            AppError::PermissionDenied(_) => "PERMISSION_DENIED",
        }
    }
}

/// Structured error response for frontend
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl From<&AppError> for ErrorResponse {
    fn from(err: &AppError) -> Self {
        ErrorResponse {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;

// ============================================================================
// Validation helpers
// ============================================================================

/// Validate MAC address format
pub fn validate_mac_address(mac: &str) -> AppResult<()> {
    let cleaned: String = mac
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':' && *c != '-')
        .collect();

    if cleaned.len() != 12 {
        return Err(AppError::validation(
            "mac_address",
            "MAC 地址必须是12位十六进制数字",
        ));
    }

    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::validation(
            "mac_address",
            "MAC 地址只能包含十六进制字符 (0-9, A-F)",
        ));
    }

    Ok(())
}

/// Validate path is not empty
pub fn validate_path_not_empty(path: &str, field_name: &str) -> AppResult<()> {
    if path.trim().is_empty() {
        return Err(AppError::validation(field_name, "路径不能为空"));
    }
    Ok(())
}
