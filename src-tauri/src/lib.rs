mod hotkey;
mod keyboard;
pub mod mac_addr;

use hotkey::{
    get_hotkey_config, get_hotkey_status, save_hotkey_config, stop_hotkey_task, HotkeyManager,
};
use keyboard::{cp_source_to_target, list_directory_contents};
use mac_addr::{
    change_mac_address, get_auto_restore_setting, get_mac_address, restore_mac_cmd,
    set_auto_restore_setting,
};
use tauri::Manager;

pub use mac_addr::restore_mac_address;

// use tauri::command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .setup(|app| {
            let manager = HotkeyManager::new();
            manager
                .initialize(&app.handle())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            app.manage(manager);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 直接使用导入的函数
            get_mac_address,
            change_mac_address,
            restore_mac_cmd,
            get_auto_restore_setting,
            set_auto_restore_setting,
            list_directory_contents,
            cp_source_to_target,
            get_hotkey_config,
            save_hotkey_config,
            get_hotkey_status,
            stop_hotkey_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
