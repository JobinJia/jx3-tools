//! MAC address management service
//!
//! This module provides functionality for:
//! - Getting current MAC address
//! - Changing MAC address (Windows only)
//! - Restoring original MAC address
//! - Auto-restore on reboot via Windows Task Scheduler

mod scripts;

use std::fs;
use std::io::Read;
#[cfg(target_os = "windows")]
use std::io::Write;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::process::Command;

use dirs;
#[cfg(not(target_os = "windows"))]
use mac_address;
#[cfg(target_os = "windows")]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use serde_json;

use crate::error::{AppError, AppResult};

const AUTO_RESTORE_FILE: &str = "mac_auto_restore.txt";
#[cfg(target_os = "windows")]
const MAC_STATE_FILE: &str = "mac_state.json";
#[cfg(target_os = "windows")]
const TASK_NAME: &str = "JX3ToolsMacRestore";

/// Service for MAC address management
pub struct MacService {
    base_dir: PathBuf,
}

impl MacService {
    /// Create a new MacService
    pub fn new() -> AppResult<Self> {
        let mut dir =
            dirs::config_dir().ok_or_else(|| AppError::Config("无法定位配置目录".into()))?;
        dir.push("jx3-tools");
        fs::create_dir_all(&dir)?;
        Ok(Self { base_dir: dir })
    }

    /// Get the current MAC address
    pub fn get_mac_address(&self) -> AppResult<String> {
        #[cfg(target_os = "windows")]
        {
            let adapter = get_primary_adapter()?;
            Ok(format_mac_address(&adapter.mac_address))
        }

        #[cfg(not(target_os = "windows"))]
        {
            match mac_address::get_mac_address() {
                Ok(Some(addr)) => Ok(format_mac_address(&addr.to_string())),
                Ok(None) => Err(AppError::Message("无法找到 MAC 地址".into())),
                Err(e) => Err(AppError::Message(format!("获取 MAC 地址失败: {e}"))),
            }
        }
    }

    /// Change the MAC address (Windows only)
    pub fn change_mac_address(&self, mac_address: &str) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = mac_address;
            return Err(AppError::Message("此功能仅支持 Windows 系统".into()));
        }

        #[cfg(target_os = "windows")]
        {
            let adapter = get_primary_adapter()?;
            let sanitized = sanitize_mac_input(mac_address)?;
            set_network_address_value(&adapter.interface_guid, Some(&sanitized))?;
            restart_network_adapter(&adapter.name)?;
            save_mac_state(
                self.state_file_path(),
                MacState {
                    adapter_guid: adapter.interface_guid,
                    adapter_name: adapter.name,
                },
            )?;
            Ok(())
        }
    }

    /// Restore the original MAC address (Windows only)
    pub fn restore_mac_address(&self) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err(AppError::Message("此功能仅支持 Windows 系统".into()));
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(state) = load_mac_state(self.state_file_path())? {
                if apply_restore(&state.adapter_guid, &state.adapter_name).is_err() {
                    log::warn!("使用缓存信息还原失败，尝试自动检测网络适配器");
                } else {
                    clear_mac_state(self.state_file_path())?;
                    return Ok(());
                }
            }

            let adapter = get_primary_adapter()?;
            apply_restore(&adapter.interface_guid, &adapter.name)?;
            clear_mac_state(self.state_file_path())?;
            Ok(())
        }
    }

    /// Get the auto-restore on reboot setting
    pub fn get_auto_restore_setting(&self) -> AppResult<bool> {
        let path = self.auto_restore_file_path();
        if !path.exists() {
            return Ok(false);
        }
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.trim() == "true")
    }

    /// Set the auto-restore on reboot setting (Windows only)
    pub fn set_auto_restore_setting(&self, auto_restore: bool) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            if auto_restore {
                return Err(AppError::Message("此功能仅支持 Windows 系统".into()));
            }
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let path = self.auto_restore_file_path();
            let mut file = fs::File::create(path)?;
            file.write_all(if auto_restore { b"true" } else { b"false" })?;
            if auto_restore {
                setup_auto_restore_on_boot()?;
            } else {
                remove_auto_restore_on_boot()?;
            }
            Ok(())
        }
    }

    fn auto_restore_file_path(&self) -> PathBuf {
        self.base_dir.join(AUTO_RESTORE_FILE)
    }

    #[cfg(target_os = "windows")]
    fn state_file_path(&self) -> PathBuf {
        self.base_dir.join(MAC_STATE_FILE)
    }
}

// ============================================================================
// Windows-specific types and functions
// ============================================================================

#[cfg(target_os = "windows")]
#[derive(Debug, Serialize, Deserialize)]
struct AdapterInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "InterfaceGuid")]
    interface_guid: String,
    #[serde(rename = "MacAddress")]
    mac_address: String,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Serialize, Deserialize)]
struct MacState {
    adapter_guid: String,
    adapter_name: String,
}

/// Get primary network adapter information using PowerShell
#[cfg(target_os = "windows")]
fn get_primary_adapter() -> AppResult<AdapterInfo> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            scripts::GET_ADAPTER,
        ])
        .output()
        .map_err(|e| AppError::Command(format!("执行 PowerShell 失败: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Command(format!(
            "无法获取网络适配器信息: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Err(AppError::Command("找不到可用的网络适配器".into()));
    }

    serde_json::from_str(stdout.trim())
        .map_err(|e| AppError::Command(format!("解析网络适配器信息失败: {e}")))
}

#[cfg(target_os = "windows")]
fn apply_restore(adapter_guid: &str, adapter_name: &str) -> AppResult<()> {
    set_network_address_value(adapter_guid, None)?;
    restart_network_adapter(adapter_name)?;
    Ok(())
}

/// Format MAC address to standard format (XX:XX:XX:XX:XX:XX)
fn format_mac_address(mac: &str) -> String {
    let cleaned: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if cleaned.len() != 12 {
        return mac.trim().to_string();
    }

    let mut formatted = String::with_capacity(17);
    for i in 0..6 {
        if i > 0 {
            formatted.push(':');
        }
        formatted.push_str(&cleaned[i * 2..i * 2 + 2]);
    }

    formatted
}

#[cfg(target_os = "windows")]
fn setup_auto_restore_on_boot() -> AppResult<()> {
    let app_path = std::env::current_exe()?;
    Command::new("schtasks")
        .args(&[
            "/create",
            "/tn",
            TASK_NAME,
            "/tr",
            &format!("\"{}\" --restore-mac", app_path.to_string_lossy()),
            "/sc",
            "onlogon",
            "/f",
        ])
        .output()
        .map_err(|e| AppError::Command(format!("创建任务计划失败: {e}")))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn remove_auto_restore_on_boot() -> AppResult<()> {
    Command::new("schtasks")
        .args(&["/delete", "/tn", TASK_NAME, "/f"])
        .output()
        .map_err(|e| AppError::Command(format!("删除任务计划失败: {e}")))?;
    Ok(())
}

/// Validate and sanitize MAC address input
#[cfg(target_os = "windows")]
fn sanitize_mac_input(mac_address: &str) -> AppResult<String> {
    let cleaned: String = mac_address
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':' && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if cleaned.len() != 12 || !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Message("MAC 地址格式不正确".into()));
    }

    let first_byte = u8::from_str_radix(&cleaned[0..2], 16)
        .map_err(|_| AppError::Message("MAC 地址格式不正确".into()))?;
    if first_byte & 0x01 == 0x01 {
        return Err(AppError::Message("MAC 地址不能是组播地址".into()));
    }

    Ok(cleaned)
}

/// Restart network adapter using PowerShell
#[cfg(target_os = "windows")]
fn restart_network_adapter(adapter_name: &str) -> AppResult<()> {
    let sanitized_name = adapter_name.replace('"', "").replace('\'', "''");
    let script = scripts::RESTART_ADAPTER_TEMPLATE.replace("{NAME}", &sanitized_name);
    run_powershell(&script)
}

/// Execute a PowerShell script
#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> AppResult<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| AppError::Command(format!("执行 PowerShell 失败: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = stderr.trim();

        // Check for common permission-related errors
        if error_msg.contains("Access is denied")
            || error_msg.contains("拒绝访问")
            || error_msg.contains("AccessDenied")
            || error_msg.contains("requires elevation")
            || error_msg.contains("Run as administrator")
            || error_msg.contains("管理员")
        {
            return Err(AppError::permission_denied("修改网络设置"));
        }

        Err(AppError::Command(error_msg.to_string()))
    }
}

/// Set or remove the NetworkAddress registry value
#[cfg(target_os = "windows")]
fn set_network_address_value(interface_guid: &str, value: Option<&str>) -> AppResult<()> {
    let guid = interface_guid.replace('"', "").replace('\'', "''");
    let template = scripts::SET_NETWORK_ADDRESS_TEMPLATE.replace("{GUID}", &guid);

    let script = match value {
        Some(mac_value) => {
            let sanitized_value = mac_value.replace(':', "").replace('-', "").to_uppercase();
            template.replace("{ACTION}", &scripts::set_mac_action(&sanitized_value))
        }
        None => template.replace("{ACTION}", scripts::remove_mac_action()),
    };

    run_powershell(&script)
}

// ============================================================================
// State persistence
// ============================================================================

#[cfg(target_os = "windows")]
fn save_mac_state(path: PathBuf, state: MacState) -> AppResult<()> {
    let data = serde_json::to_string(&state)
        .map_err(|e| AppError::Command(format!("保存 MAC 状态失败: {e}")))?;
    fs::write(path, data)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn load_mac_state(path: PathBuf) -> AppResult<Option<MacState>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let state = serde_json::from_str(&content)
        .map_err(|e| AppError::Command(format!("解析 MAC 状态失败: {e}")))?;
    Ok(Some(state))
}

#[cfg(target_os = "windows")]
fn clear_mac_state(path: PathBuf) -> AppResult<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
