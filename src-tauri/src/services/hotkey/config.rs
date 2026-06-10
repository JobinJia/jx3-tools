use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use super::keymap;
use super::types::{HotkeyConfig, KeyMode};

pub const CONFIG_FILE_NAME: &str = "hotkey_config.json";

/// Ensure the app config directory exists and return its path
pub fn ensure_app_config_dir() -> AppResult<PathBuf> {
    let mut base = dirs::config_dir()
        .ok_or_else(|| AppError::Config("无法获取配置目录".into()))?;
    base.push("jx3-tools");
    fs::create_dir_all(&base)?;
    Ok(base)
}

/// Load config from disk
pub fn load_config(config_path: &PathBuf) -> AppResult<HotkeyConfig> {
    if !config_path.exists() {
        return Ok(HotkeyConfig::default());
    }
    let content = fs::read_to_string(config_path)?;
    let config = serde_json::from_str::<HotkeyConfig>(&content)?;
    Ok(config)
}

/// Save config to disk
pub fn save_config(config_path: &PathBuf, config: &HotkeyConfig) -> AppResult<()> {
    let data = serde_json::to_string_pretty(config)?;
    fs::write(config_path, data)?;
    Ok(())
}

/// Validate config before saving
pub fn validate_config(config: &HotkeyConfig) -> AppResult<()> {
    if config.trigger_key.trim().is_empty() {
        return Err(AppError::Hotkey("触发按键不能为空".into()));
    }
    if config.interval_ms < 20 {
        return Err(AppError::Hotkey("触发频率不能低于 20 毫秒".into()));
    }
    if config.interval_ms > 60000 {
        return Err(AppError::Hotkey("触发频率不能高于 60000 毫秒".into()));
    }
    if config.start_hotkey.trim().is_empty() {
        return Err(AppError::Hotkey("开始热键不能为空".into()));
    }
    if config.stop_hotkey.trim().is_empty() {
        return Err(AppError::Hotkey("结束热键不能为空".into()));
    }
    keymap::resolve_key(&config.trigger_key)
        .map_err(|e| AppError::Hotkey(format!("触发按键格式无效: {e}")))?;
    let start = keymap::parse_shortcut(&config.start_hotkey)
        .map_err(|e| AppError::Hotkey(format!("开始热键格式无效: {e}")))?;
    let stop = keymap::parse_shortcut(&config.stop_hotkey)
        .map_err(|e| AppError::Hotkey(format!("结束热键格式无效: {e}")))?;

    if start == stop {
        return Err(AppError::Hotkey("开始与结束热键不能相同".into()));
    }

    // 触发按键被模拟按下时会命中同名热键，必须与开始/结束热键错开
    if let Ok(trigger) = keymap::parse_shortcut(&config.trigger_key) {
        if trigger == start || trigger == stop {
            return Err(AppError::Hotkey("触发按键不能与开始/结束热键相同".into()));
        }
    }

    // 窗口模式验证
    if config.key_mode == KeyMode::Window {
        #[cfg(not(target_os = "windows"))]
        return Err(AppError::Hotkey("窗口模式仅支持 Windows".into()));

        #[cfg(target_os = "windows")]
        if config.target_window.is_none() {
            return Err(AppError::Hotkey("窗口模式需要选择目标窗口".into()));
        }
    }

    Ok(())
}

/// Validate config at runtime (before starting runner)
#[cfg(target_os = "windows")]
pub fn validate_runtime_config(config: &HotkeyConfig) -> AppResult<()> {
    if config.trigger_key.trim().is_empty() {
        return Err(AppError::Hotkey("触发按键未设置".into()));
    }
    if config.interval_ms < 20 {
        return Err(AppError::Hotkey("触发频率不能低于 20 毫秒".into()));
    }
    if config.interval_ms > 60000 {
        return Err(AppError::Hotkey("触发频率不能高于 60000 毫秒".into()));
    }
    Ok(())
}
