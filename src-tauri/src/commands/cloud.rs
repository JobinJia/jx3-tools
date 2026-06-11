//! Cloud sync commands (WebDAV)

use tauri::{command, AppHandle, Emitter};

use crate::error::{validate_path_not_empty, AppError, AppResult};
use crate::services::cloud::config::{load_config, save_config, CloudConfig};
use crate::services::cloud::sync::{
    CloudBatchUploadReport, CloudDownloadReport, CloudRoleEntry, CloudSyncService,
};
use crate::services::cloud::webdav::WebDavStorage;

/// 上传/下载进度事件名（前端 listen 此事件驱动进度条）
pub const CLOUD_PROGRESS_EVENT: &str = "cloud://progress";

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

/// 保存前先验证连通性——配错地址（如坚果云漏掉 /dav/）会在这里被拦下，
/// 而不是等到上传时报一串 409
#[command]
pub async fn save_cloud_config(config: CloudConfig) -> AppResult<()> {
    log::debug!("Command: save_cloud_config({})", config.server_url);
    tauri::async_runtime::spawn_blocking(move || {
        use crate::services::cloud::webdav::CloudStorage;
        WebDavStorage::new(&config.server_url, &config.username, &config.app_password)?.check()?;
        save_config(&config)
    })
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

/// 批量上传 userdata 下所有角色（键位 + 插件配置），无需选择
#[command]
pub async fn cloud_upload_all(
    app: AppHandle,
    userdata_path: String,
) -> AppResult<CloudBatchUploadReport> {
    log::debug!("Command: cloud_upload_all({userdata_path})");
    validate_path_not_empty(&userdata_path, "userdata_path")?;
    tauri::async_runtime::spawn_blocking(move || {
        let storage = storage_from_saved()?;
        CloudSyncService::upload_all_roles(&storage, std::path::Path::new(&userdata_path), &|p| {
            let _ = app.emit(CLOUD_PROGRESS_EVENT, p);
        })
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
pub async fn cloud_download_role(
    app: AppHandle,
    key: String,
    target_path: String,
) -> AppResult<CloudDownloadReport> {
    log::debug!("Command: cloud_download_role({key} -> {target_path})");
    validate_path_not_empty(&target_path, "target_path")?;
    tauri::async_runtime::spawn_blocking(move || {
        let storage = storage_from_saved()?;
        CloudSyncService::download_role(
            &storage,
            &key,
            std::path::Path::new(&target_path),
            &|p| {
                let _ = app.emit(CLOUD_PROGRESS_EVENT, p);
            },
        )
    })
    .await
    .map_err(|e| AppError::Cloud(format!("后台任务执行失败: {e}")))?
}
