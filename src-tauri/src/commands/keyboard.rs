//! Keyboard configuration commands

use std::process::Command;
use tauri::command;

use crate::error::{validate_path_not_empty, AppError, AppResult};
use crate::services::keyboard::{CopyParams, FileEntry, KeyboardService};
use crate::services::plugin_data::{PluginDataService, PluginSyncReport};

/// List directory contents for keyboard configuration
///
/// async + spawn_blocking：同步命令会在主线程执行，递归磁盘 IO 会卡 UI
#[command]
pub async fn list_directory_contents(path: String) -> AppResult<Vec<FileEntry>> {
    log::debug!("Command: list_directory_contents({})", path);
    validate_path_not_empty(&path, "path")?;
    tauri::async_runtime::spawn_blocking(move || KeyboardService::list_directory_contents(&path))
        .await
        .map_err(|e| AppError::Keyboard(format!("后台任务执行失败: {e}")))?
}

/// Copy keyboard configuration from source to target
#[command]
pub async fn cp_source_to_target(params: CopyParams) -> AppResult<bool> {
    log::debug!(
        "Command: cp_source_to_target({} -> {})",
        params.source_path,
        params.target_path
    );
    validate_path_not_empty(&params.source_path, "source_path")?;
    validate_path_not_empty(&params.target_path, "target_path")?;
    tauri::async_runtime::spawn_blocking(move || KeyboardService::copy_source_to_target(&params))
        .await
        .map_err(|e| AppError::Keyboard(format!("后台任务执行失败: {e}")))?
}

/// Sync plugin config (interface/*#data) from source role to target role
///
/// 与键位复制同一套参数（userdata 下的源/目标角色目录路径）
#[command]
pub async fn sync_plugin_config(params: CopyParams) -> AppResult<PluginSyncReport> {
    log::debug!(
        "Command: sync_plugin_config({} -> {})",
        params.source_path,
        params.target_path
    );
    validate_path_not_empty(&params.source_path, "source_path")?;
    validate_path_not_empty(&params.target_path, "target_path")?;
    tauri::async_runtime::spawn_blocking(move || PluginDataService::sync_plugin_config(&params))
        .await
        .map_err(|e| AppError::Plugin(format!("后台任务执行失败: {e}")))?
}

/// Open folder in system file explorer
#[command]
pub fn open_folder(path: &str) -> AppResult<()> {
    log::debug!("Command: open_folder({})", path);
    validate_path_not_empty(path, "path")?;

    #[cfg(target_os = "windows")]
    {
        // explorer.exe 不认正斜杠：前端拼的路径是 basePath\...userdata/账号/角色 混合分隔符，
        // 不归一化会回退打开默认目录（文档）而不是目标目录
        let normalized = path.replace('/', "\\");
        if let Err(e) = Command::new("explorer").arg(&normalized).spawn() {
            log::error!("无法打开文件夹 {}: {}", normalized, e);
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
