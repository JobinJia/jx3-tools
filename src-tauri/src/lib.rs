mod app_state;
mod commands;
mod error;
mod services;

use app_state::AppState;
use commands::*;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

pub use error::AppError;
pub use services::hotkey::HOTKEY_STATUS_EVENT;
pub use services::mac::MacService;

/// Restore MAC address (called from main.rs for CLI)
pub fn restore_mac_address() -> error::AppResult<()> {
    let service = MacService::new()?;
    service.restore_mac_address()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        if let Some(location) = info.location() {
            log::error!(
                "应用发生未捕获 panic: {} ({}:{})",
                info,
                location.file(),
                location.line()
            );
        } else {
            log::error!("应用发生未捕获 panic: {}", info);
        }
    }));

    tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Never)
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .targets([
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let state = match AppState::initialize(&app.handle()) {
                Ok(state) => state,
                Err(err) => {
                    log::error!("初始化应用状态失败: {}", err);
                    return Err(Box::new(err));
                }
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // MAC address commands
            get_mac_address,
            change_mac_address,
            restore_mac_cmd,
            get_auto_restore_setting,
            set_auto_restore_setting,
            // Keyboard commands
            list_directory_contents,
            cp_source_to_target,
            open_folder,
            // Hotkey commands
            get_hotkey_config,
            get_hotkey_status,
            save_hotkey_config,
            stop_hotkey_task,
            list_windows,
            check_window_valid,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| {
            log::error!("Tauri 应用运行失败: {}", err);
            eprintln!("Tauri 应用运行失败: {}", err);
            std::process::exit(1);
        });
}
