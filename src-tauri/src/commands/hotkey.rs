//! Hotkey automation commands

use tauri::{command, AppHandle};

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::services::hotkey::window::WindowInfo;
use crate::services::hotkey::{HotkeyConfig, HotkeyStatus};

/// Get the current hotkey configuration
#[command]
pub fn get_hotkey_config(state: tauri::State<AppState>) -> HotkeyConfig {
    log::debug!("Command: get_hotkey_config");
    state.hotkey().get_config()
}

/// Get the current hotkey status
#[command]
pub fn get_hotkey_status(state: tauri::State<AppState>) -> HotkeyStatus {
    log::debug!("Command: get_hotkey_status");
    state.hotkey().get_status()
}

/// Save hotkey configuration and register shortcuts
#[command]
pub fn save_hotkey_config(
    app: AppHandle,
    state: tauri::State<AppState>,
    config: HotkeyConfig,
) -> AppResult<HotkeyConfig> {
    log::debug!("Command: save_hotkey_config({:?})", config);
    state.hotkey().save_config(&app, config)
}

/// Stop the running hotkey automation task
#[command]
pub fn stop_hotkey_task(app: AppHandle, state: tauri::State<AppState>) {
    log::debug!("Command: stop_hotkey_task");
    state.hotkey().stop_runner(&app);
}

/// 获取可见窗口列表（仅 Windows）
#[command]
pub fn list_windows(filter: Option<String>) -> AppResult<Vec<WindowInfo>> {
    log::debug!("Command: list_windows(filter={:?})", filter);
    crate::services::hotkey::window::enumerate_windows(filter.as_deref())
}

/// 检查窗口是否仍然有效
#[command]
pub fn check_window_valid(hwnd: u64) -> bool {
    log::debug!("Command: check_window_valid(hwnd={})", hwnd);
    crate::services::hotkey::window::is_window_valid(hwnd)
}
