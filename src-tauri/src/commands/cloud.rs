//! Cloud sync commands (WebDAV)

use tauri::command;

use crate::error::{validate_path_not_empty, AppError, AppResult};
use crate::services::cloud::config::{load_config, save_config, CloudConfig};
use crate::services::cloud::sync::{
    CloudDownloadReport, CloudRoleEntry, CloudSyncService, CloudUploadReport,
};
use crate::services::cloud::webdav::WebDavStorage;

/// 用已保存的配置构建 WebDAV 存储；未配置则报错引导用户先绑定
fn storage_from_saved() -> AppResult<WebDavStorage> {
    let config =
        load_config()?.ok_or_else(|| AppError::Cloud("尚未配置云同步账号，请先绑定网盘".into()))?;
    WebDavStorage::new(&config.server_url, &config.username, &config.app_password)
}

#[command]
pub async fn get_cloud_config() -> AppResult<Option<CloudConfig>> {
    log::debug!("Command: get_cloud_config");
    tauri::async_runtime::spawn_blocking(load_config)
        .await
        .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}

#[command]
pub async fn save_cloud_config(config: CloudConfig) -> AppResult<()> {
    log::debug!("Command: save_cloud_config({})", config.server_url);
    tauri::async_runtime::spawn_blocking(move || save_config(&config))
        .await
        .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}

/// 用传入配置（而非已保存配置）测试连通性，供保存前验证
#[command]
pub async fn test_cloud_connection(config: CloudConfig) -> AppResult<()> {
    log::debug!("Command: test_cloud_connection({})", config.server_url);
    tauri::async_runtime::spawn_blocking(move || {
        use crate::services::cloud::webdav::CloudStorage;
        WebDavStorage::new(&config.server_url, &config.username, &config.app_password)?.check()
    })
    .await
    .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}

#[command]
pub async fn cloud_upload_role(role_path: String) -> AppResult<CloudUploadReport> {
    log::debug!("Command: cloud_upload_role({role_path})");
    validate_path_not_empty(&role_path, "role_path")?;
    tauri::async_runtime::spawn_blocking(move || {
        let storage = storage_from_saved()?;
        CloudSyncService::upload_role(&storage, std::path::Path::new(&role_path))
    })
    .await
    .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}

#[command]
pub async fn cloud_list_roles() -> AppResult<Vec<CloudRoleEntry>> {
    log::debug!("Command: cloud_list_roles");
    tauri::async_runtime::spawn_blocking(move || {
        let storage = storage_from_saved()?;
        CloudSyncService::list_roles(&storage)
    })
    .await
    .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}

#[command]
pub async fn cloud_download_role(key: String, target_path: String) -> AppResult<CloudDownloadReport> {
    log::debug!("Command: cloud_download_role({key} -> {target_path})");
    validate_path_not_empty(&target_path, "target_path")?;
    tauri::async_runtime::spawn_blocking(move || {
        let storage = storage_from_saved()?;
        CloudSyncService::download_role(&storage, &key, std::path::Path::new(&target_path))
    })
    .await
    .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}
