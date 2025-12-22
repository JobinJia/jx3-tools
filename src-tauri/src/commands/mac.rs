//! MAC address management commands

use tauri::command;

use crate::app_state::AppState;
use crate::error::{validate_mac_address, AppResult};

/// Get the current MAC address
#[command]
pub fn get_mac_address(state: tauri::State<AppState>) -> AppResult<String> {
    log::debug!("Command: get_mac_address");
    state.mac().get_mac_address()
}

/// Change the MAC address to a new value
#[command]
pub fn change_mac_address(state: tauri::State<AppState>, mac_address: String) -> AppResult<()> {
    log::debug!("Command: change_mac_address({})", mac_address);

    // Validate MAC address format at command layer
    validate_mac_address(&mac_address)?;

    state.mac().change_mac_address(&mac_address)
}

/// Restore the original MAC address
#[command]
pub fn restore_mac_cmd(state: tauri::State<AppState>) -> AppResult<()> {
    log::debug!("Command: restore_mac_cmd");
    state.mac().restore_mac_address()
}

/// Get the auto-restore on reboot setting
#[command]
pub fn get_auto_restore_setting(state: tauri::State<AppState>) -> AppResult<bool> {
    log::debug!("Command: get_auto_restore_setting");
    state.mac().get_auto_restore_setting()
}

/// Set the auto-restore on reboot setting
#[command]
pub fn set_auto_restore_setting(
    state: tauri::State<AppState>,
    auto_restore: bool,
) -> AppResult<()> {
    log::debug!("Command: set_auto_restore_setting({})", auto_restore);
    state.mac().set_auto_restore_setting(auto_restore)
}
