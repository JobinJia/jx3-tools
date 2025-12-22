use std::sync::Arc;

use tauri::AppHandle;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutEvent, ShortcutState};

use crate::error::{AppError, AppResult};
use super::HotkeyService;

/// Register global shortcuts for start/stop hotkeys
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn register_shortcuts(service: &Arc<HotkeyService>, app: &AppHandle) -> AppResult<()> {
    let config = service.get_config();
    let manager = app.global_shortcut();

    // Unregister all existing shortcuts
    manager
        .unregister_all()
        .map_err(|e| AppError::Hotkey(format!("注销旧热键失败: {e}")))?;

    // Skip if hotkeys are empty
    if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
        return Ok(());
    }

    // Register start hotkey
    let service_for_start = Arc::clone(service);
    manager
        .on_shortcut(
            config.start_hotkey.as_str(),
            move |app_handle, _, event: ShortcutEvent| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                if let Err(err) = service_for_start.start_runner(app_handle) {
                    log::error!("启动热键任务失败: {}", err);
                    service_for_start.update_status(app_handle, |status| {
                        status.last_error = Some(err.to_string());
                    });
                }
            },
        )
        .map_err(|e| AppError::Hotkey(format!("注册开始热键失败: {e}")))?;

    // Register stop hotkey
    let service_for_stop = Arc::clone(service);
    manager
        .on_shortcut(
            config.stop_hotkey.as_str(),
            move |app_handle, _, event: ShortcutEvent| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                service_for_stop.stop_runner(app_handle);
            },
        )
        .map_err(|e| AppError::Hotkey(format!("注册停止热键失败: {e}")))?;

    Ok(())
}

/// Fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn register_shortcuts(_service: &Arc<HotkeyService>, _app: &AppHandle) -> AppResult<()> {
    Err(AppError::Hotkey("热键功能仅支持 Windows 或 macOS".into()))
}
