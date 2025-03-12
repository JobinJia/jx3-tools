pub mod mac_addr;
mod keyboard;

use mac_addr::{get_mac_address, change_mac_address, restore_mac_cmd, get_auto_restore_setting, set_auto_restore_setting};
use keyboard::{list_directory_contents, cp_source_to_target};

pub use mac_addr::restore_mac_address;

// use tauri::command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
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
            cp_source_to_target
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
