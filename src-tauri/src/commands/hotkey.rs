//! Hotkey automation commands

use tauri::{command, AppHandle};

use crate::app_state::AppState;
use crate::error::AppResult;
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
