//! MAC address management commands
//!
//! async + spawn_blocking：PowerShell/schtasks 调用耗时数百毫秒到十几秒
//! （改 MAC 含网卡重启与回读验证），不能阻塞主线程

use tauri::command;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::services::mac::MacInfo;

/// Get the primary adapter's MAC info
#[command]
pub async fn get_mac_info(state: tauri::State<'_, AppState>) -> AppResult<MacInfo> {
    log::debug!("Command: get_mac_info");
    let mac = state.mac();
    tauri::async_runtime::spawn_blocking(move || mac.get_mac_info())
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))?
}

/// Change the MAC address to a random value, verified against the driver
#[command]
pub async fn randomize_mac_address(state: tauri::State<'_, AppState>) -> AppResult<MacInfo> {
    log::debug!("Command: randomize_mac_address");
    let mac = state.mac();
    tauri::async_runtime::spawn_blocking(move || mac.randomize_mac_address())
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))?
}

/// Restore the original MAC address
#[command]
pub async fn restore_mac_cmd(state: tauri::State<'_, AppState>) -> AppResult<MacInfo> {
    log::debug!("Command: restore_mac_cmd");
    let mac = state.mac();
    tauri::async_runtime::spawn_blocking(move || mac.restore_mac_address())
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))?
}

/// Get the auto-restore on reboot setting (scheduled task existence)
#[command]
pub async fn get_auto_restore_setting(state: tauri::State<'_, AppState>) -> AppResult<bool> {
    log::debug!("Command: get_auto_restore_setting");
    let mac = state.mac();
    tauri::async_runtime::spawn_blocking(move || mac.get_auto_restore_setting())
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))?
}

/// Set the auto-restore on reboot setting
#[command]
pub async fn set_auto_restore_setting(
    state: tauri::State<'_, AppState>,
    auto_restore: bool,
) -> AppResult<()> {
    log::debug!("Command: set_auto_restore_setting({})", auto_restore);
    let mac = state.mac();
    tauri::async_runtime::spawn_blocking(move || mac.set_auto_restore_setting(auto_restore))
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))?
}
