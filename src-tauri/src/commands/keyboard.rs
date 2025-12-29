//! Keyboard configuration commands

use std::process::Command;
use tauri::command;

use crate::error::{validate_path_not_empty, AppResult};
use crate::services::keyboard::{CopyParams, FileEntry, KeyboardService};

/// List directory contents for keyboard configuration
#[command]
pub fn list_directory_contents(path: &str) -> AppResult<Vec<FileEntry>> {
    log::debug!("Command: list_directory_contents({})", path);
    validate_path_not_empty(path, "path")?;
    KeyboardService::list_directory_contents(path)
}

/// Copy keyboard configuration from source to target
#[command]
pub fn cp_source_to_target(params: CopyParams) -> AppResult<bool> {
    log::debug!(
        "Command: cp_source_to_target({} -> {})",
        params.source_path,
        params.target_path
    );
    validate_path_not_empty(&params.source_path, "source_path")?;
    validate_path_not_empty(&params.target_path, "target_path")?;
    KeyboardService::copy_source_to_target(&params)
}

/// Open folder in system file explorer
#[command]
pub fn open_folder(path: &str) -> AppResult<()> {
    log::debug!("Command: open_folder({})", path);
    validate_path_not_empty(path, "path")?;

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = Command::new("explorer").arg(path).spawn() {
            log::error!("无法打开文件夹 {}: {}", path, e);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Err(e) = Command::new("open").arg(path).spawn() {
            log::error!("无法打开文件夹 {}: {}", path, e);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Err(e) = Command::new("xdg-open").arg(path).spawn() {
            log::error!("无法打开文件夹 {}: {}", path, e);
        }
    }

    Ok(())
}
