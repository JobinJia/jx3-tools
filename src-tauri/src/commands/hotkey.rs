//! Hotkey automation commands

use tauri::{command, AppHandle};

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::services::hotkey::{HotkeyConfig, HotkeyStatus};

#[cfg(target_os = "windows")]
use crate::services::hotkey::window::WindowInfo;

/// Get the current hotkey configuration
#[command]
pub fn get_hotkey_config(state: tauri::State<AppState>) -> HotkeyConfig {
    log::debug!("Command: get_hotkey_config");
    state.hotkey().get_config()
}

/// Get the current hotkey status
#[command]
pub fn get_hotkey_status(state: tauri::State<AppState>) -> HotkeyStatus {
    log::debug!("Command: get_hotkey_status");
    state.hotkey().get_status()
}

/// Save hotkey configuration and register shortcuts
#[command]
pub fn save_hotkey_config(
    app: AppHandle,
    state: tauri::State<AppState>,
    config: HotkeyConfig,
) -> AppResult<HotkeyConfig> {
    log::debug!("Command: save_hotkey_config({:?})", config);
    state.hotkey().save_config(&app, config)
}

/// Stop the running hotkey automation task
#[command]
pub fn stop_hotkey_task(app: AppHandle, state: tauri::State<AppState>) {
    log::debug!("Command: stop_hotkey_task");
    state.hotkey().stop_runner(&app);
}

/// 获取可见窗口列表（仅 Windows）
#[cfg(target_os = "windows")]
#[command]
pub fn list_windows(filter: Option<String>) -> AppResult<Vec<WindowInfo>> {
    log::debug!("Command: list_windows(filter={:?})", filter);
    crate::services::hotkey::window::enumerate_windows(filter.as_deref())
}

/// 获取可见窗口列表（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
#[command]
pub fn list_windows(_filter: Option<String>) -> AppResult<Vec<()>> {
    Ok(vec![])
}

/// 检查窗口是否仍然有效
#[cfg(target_os = "windows")]
#[command]
pub fn check_window_valid(hwnd: u64) -> bool {
    log::debug!("Command: check_window_valid(hwnd={})", hwnd);
    crate::services::hotkey::window::is_window_valid(hwnd)
}

/// 检查窗口是否仍然有效（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
#[command]
pub fn check_window_valid(_hwnd: u64) -> bool {
    false
}

/// 定位随包分发的官方 Interception 安装器。
/// 安装版用 resources/ 目录，绿色版用 include_bytes! 写到临时目录。
#[cfg(target_os = "windows")]
fn resolve_installer_exe(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    use tauri::path::BaseDirectory;
    use tauri::Manager;

    if let Ok(path) = app.path().resolve(
        "resources/interception/install-interception.exe",
        BaseDirectory::Resource,
    ) {
        if path.is_file() {
            return Ok(path);
        }
    }

    const INSTALLER: &[u8] =
        include_bytes!("../../resources/interception/install-interception.exe");
    let dir = std::env::temp_dir().join("jx3-tools-driver");
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Hotkey(format!("创建临时目录失败: {e}")))?;
    let path = dir.join("install-interception.exe");
    std::fs::write(&path, INSTALLER)
        .map_err(|e| AppError::Hotkey(format!("写出安装器失败: {e}")))?;
    Ok(path)
}

/// 安装按键驱动（只装键盘：拷 keyboard.sys + 注册服务 + 加键盘 UpperFilters），
/// 需重启生效。涉及文件拷贝与注册表写入，spawn_blocking 避免阻塞主线程
#[cfg(target_os = "windows")]
#[command]
pub async fn install_hotkey_driver(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<HotkeyStatus> {
    log::info!("Command: install_hotkey_driver");
    let installer = resolve_installer_exe(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::hotkey::driver::install(&installer)
    })
    .await
    .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))??;
    let service = state.hotkey();
    service.update_status(&app, |_| {}); // 广播最新驱动状态
    Ok(service.get_status())
}

/// 安装按键驱动（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
#[command]
pub async fn install_hotkey_driver() -> AppResult<HotkeyStatus> {
    Err(AppError::Hotkey("仅支持 Windows 平台".into()))
}

/// 卸载按键驱动（删键盘过滤器/服务/文件并清理旧版鼠标残留），需重启生效
#[cfg(target_os = "windows")]
#[command]
pub async fn uninstall_hotkey_driver(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<HotkeyStatus> {
    log::info!("Command: uninstall_hotkey_driver");
    tauri::async_runtime::spawn_blocking(crate::services::hotkey::driver::uninstall)
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))??;
    let service = state.hotkey();
    service.update_status(&app, |_| {});
    Ok(service.get_status())
}

/// 卸载按键驱动（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
#[command]
pub async fn uninstall_hotkey_driver() -> AppResult<HotkeyStatus> {
    Err(AppError::Hotkey("仅支持 Windows 平台".into()))
}

/// 移除残留的 interception 鼠标过滤器（旧版安装包全装遗留）
#[cfg(target_os = "windows")]
#[command]
pub async fn remove_mouse_filter(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<HotkeyStatus> {
    log::info!("Command: remove_mouse_filter");
    tauri::async_runtime::spawn_blocking(crate::services::hotkey::driver::remove_mouse_filter)
        .await
        .map_err(|e| AppError::Command(format!("后台任务执行失败: {e}")))??;
    let service = state.hotkey();
    service.update_status(&app, |_| {});
    Ok(service.get_status())
}

/// 移除残留的鼠标过滤器（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
#[command]
pub async fn remove_mouse_filter() -> AppResult<HotkeyStatus> {
    Err(AppError::Hotkey("仅支持 Windows 平台".into()))
}
