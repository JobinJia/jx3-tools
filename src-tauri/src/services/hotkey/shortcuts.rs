//! Global hotkey registration using tauri-plugin-global-shortcut
//!
//! This module provides global keyboard shortcut functionality
//! to detect start/stop hotkey presses.

use std::sync::Arc;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::HotkeyService;
use crate::error::{AppError, AppResult};

/// 注册全局快捷键
pub fn register_shortcuts(service: &Arc<HotkeyService>, app: &AppHandle) -> AppResult<()> {
    let config = service.get_config();

    // 跳过空热键
    if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
        return Ok(());
    }

    // 先取消注册所有已有的快捷键
    if let Err(e) = app.global_shortcut().unregister_all() {
        log::warn!("取消注册快捷键失败: {}", e);
    }

    // 解析热键字符串为 Shortcut
    let start_shortcut: Shortcut = config
        .start_hotkey
        .parse()
        .map_err(|e| AppError::Hotkey(format!("解析开始热键失败: {}", e)))?;

    let stop_shortcut: Shortcut = config
        .stop_hotkey
        .parse()
        .map_err(|e| AppError::Hotkey(format!("解析停止热键失败: {}", e)))?;

    let service_start = Arc::clone(service);
    let service_stop = Arc::clone(service);
    let app_start = app.clone();
    let app_stop = app.clone();

    // 注册开始热键
    app.global_shortcut()
        .on_shortcut(start_shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                log::info!("检测到开始热键");
                if let Err(err) = service_start.start_runner(&app_start) {
                    log::error!("启动热键任务失败: {}", err);
                    service_start.update_status(&app_start, |status| {
                        status.last_error = Some(err.to_string());
                    });
                }
            }
        })
        .map_err(|e| AppError::Hotkey(format!("注册开始热键失败: {}", e)))?;

    // 注册停止热键
    app.global_shortcut()
        .on_shortcut(stop_shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                log::info!("检测到停止热键");
                service_stop.stop_runner(&app_stop);
            }
        })
        .map_err(|e| AppError::Hotkey(format!("注册停止热键失败: {}", e)))?;

    log::info!(
        "全局热键已注册: 开始={}, 停止={}",
        config.start_hotkey,
        config.stop_hotkey
    );

    Ok(())
}

/// 取消注册所有快捷键
#[allow(dead_code)]
pub fn unregister_shortcuts(app: &AppHandle) -> AppResult<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::Hotkey(format!("取消注册快捷键失败: {}", e)))
}
