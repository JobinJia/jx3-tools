use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use tauri_plugin_global_shortcut::Shortcut;

use crate::error::{AppError, AppResult};
use super::types::HotkeyConfig;

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
    if config.start_hotkey.trim().is_empty() {
        return Err(AppError::Hotkey("开始热键不能为空".into()));
    }
    if config.stop_hotkey.trim().is_empty() {
        return Err(AppError::Hotkey("结束热键不能为空".into()));
    }
    if config.start_hotkey.eq_ignore_ascii_case(&config.stop_hotkey) {
        return Err(AppError::Hotkey("开始与结束热键不能相同".into()));
    }

    Shortcut::from_str(&config.start_hotkey)
        .map_err(|e| AppError::Hotkey(format!("开始热键格式无效: {e}")))?;
    Shortcut::from_str(&config.stop_hotkey)
        .map_err(|e| AppError::Hotkey(format!("结束热键格式无效: {e}")))?;

    Ok(())
}

/// Validate config at runtime (before starting runner)
#[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
pub fn validate_runtime_config(config: &HotkeyConfig) -> AppResult<()> {
    if config.trigger_key.trim().is_empty() {
        return Err(AppError::Hotkey("触发按键未设置".into()));
    }
    if config.interval_ms < 20 {
        return Err(AppError::Hotkey("触发频率不能低于 20 毫秒".into()));
    }
    Ok(())
}
