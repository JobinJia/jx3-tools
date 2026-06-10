//! MAC address management service
//!
//! Windows strategy: write the `NetworkAddress` override on the adapter's
//! class registry key, restart the adapter, then read the MAC back to verify
//! the driver actually accepted it (many drivers, especially wireless ones,
//! silently ignore the override). Restore removes the override from every
//! physical adapter so they fall back to the permanent (burned-in) address.
//! The registry and Task Scheduler are the single source of truth — no local
//! state files, so the state survives app restarts and stays accurate.

mod scripts;

#[cfg(any(target_os = "windows", test))]
use serde::Deserialize;
use serde::Serialize;

use crate::error::{AppError, AppResult};

#[cfg(target_os = "windows")]
const TASK_NAME: &str = "JX3ToolsMacRestore";

/// Adapter MAC info reported to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacInfo {
    pub adapter_name: String,
    pub current_mac: String,
    pub permanent_mac: String,
    pub is_modified: bool,
}

/// JSON payload emitted by the PowerShell scripts
#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PsAdapterInfo {
    name: String,
    current_mac: String,
    permanent_mac: String,
    has_override: bool,
}

/// Service for MAC address management
pub struct MacService;

impl MacService {
    /// Create a new MacService
    pub fn new() -> AppResult<Self> {
        Ok(Self)
    }

    /// Get the primary adapter's MAC info
    pub fn get_mac_info(&self) -> AppResult<MacInfo> {
        #[cfg(target_os = "windows")]
        {
            let stdout = run_powershell(&scripts::get_mac_info_script())?;
            Ok(mac_info_from_ps(parse_adapter_info(&stdout)?))
        }

        #[cfg(not(target_os = "windows"))]
        {
            match mac_address::get_mac_address() {
                Ok(Some(addr)) => {
                    let mac = format_mac_address(&addr.to_string());
                    Ok(MacInfo {
                        adapter_name: "本机网卡".into(),
                        current_mac: mac.clone(),
                        permanent_mac: mac,
                        is_modified: false,
                    })
                }
                Ok(None) => Err(AppError::Message("无法找到 MAC 地址".into())),
                Err(e) => Err(AppError::Message(format!("获取 MAC 地址失败: {e}"))),
            }
        }
    }

    /// Change the MAC address to a random locally-administered value (Windows only).
    /// The change is verified by reading the MAC back; if the driver ignored it
    /// the registry override is rolled back and an error is returned.
    pub fn randomize_mac_address(&self) -> AppResult<MacInfo> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err(AppError::platform_not_supported("MAC 地址修改"));
        }

        #[cfg(target_os = "windows")]
        {
            let new_mac = generate_random_mac()?;
            log::info!("修改 MAC 地址为 {new_mac}");
            let stdout = run_powershell(&scripts::change_mac_script(&new_mac))?;
            Ok(mac_info_from_ps(parse_adapter_info(&stdout)?))
        }
    }

    /// Restore the original MAC address by removing all overrides (Windows only).
    /// No-op (and non-disruptive) when nothing is overridden.
    pub fn restore_mac_address(&self) -> AppResult<MacInfo> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err(AppError::platform_not_supported("MAC 地址还原"));
        }

        #[cfg(target_os = "windows")]
        {
            let stdout = run_powershell(&scripts::restore_mac_script())?;
            Ok(mac_info_from_ps(parse_adapter_info(&stdout)?))
        }
    }

    /// Whether the auto-restore-on-logon scheduled task exists
    pub fn get_auto_restore_setting(&self) -> AppResult<bool> {
        #[cfg(target_os = "windows")]
        {
            auto_restore_task_exists()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(false)
        }
    }

    /// Create or delete the auto-restore-on-logon scheduled task (Windows only)
    pub fn set_auto_restore_setting(&self, auto_restore: bool) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            if auto_restore {
                return Err(AppError::platform_not_supported("MAC 地址自动还原"));
            }
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            if auto_restore {
                create_auto_restore_task()
            } else {
                delete_auto_restore_task()
            }
        }
    }
}

// ============================================================================
// Shared helpers
// ============================================================================

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

/// Generate a random unicast, locally-administered MAC (12 uppercase hex chars).
/// Drivers commonly reject spoofed MACs without the locally-administered bit.
#[cfg(any(target_os = "windows", test))]
fn generate_random_mac() -> AppResult<String> {
    let mut bytes = [0u8; 6];
    getrandom::fill(&mut bytes).map_err(|e| AppError::Command(format!("生成随机 MAC 失败: {e}")))?;
    bytes[0] = (bytes[0] | 0x02) & 0xFE;
    Ok(bytes.iter().map(|b| format!("{b:02X}")).collect())
}

#[cfg(any(target_os = "windows", test))]
fn parse_adapter_info(stdout: &str) -> AppResult<PsAdapterInfo> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(AppError::Command("未获取到网卡信息".into()));
    }
    serde_json::from_str(trimmed).map_err(|e| AppError::Command(format!("解析网卡信息失败: {e}")))
}

#[cfg(any(target_os = "windows", test))]
fn mac_info_from_ps(info: PsAdapterInfo) -> MacInfo {
    let current_mac = format_mac_address(&info.current_mac);
    let permanent_mac = format_mac_address(&info.permanent_mac);
    let differs =
        !current_mac.is_empty() && !permanent_mac.is_empty() && current_mac != permanent_mac;
    MacInfo {
        adapter_name: info.name,
        current_mac,
        permanent_mac,
        is_modified: info.has_override || differs,
    }
}

#[cfg(any(target_os = "windows", test))]
fn is_permission_error(stderr: &str) -> bool {
    [
        "需要管理员权限",
        "Access is denied",
        "拒绝访问",
        "AccessDenied",
        "requires elevation",
        "管理员",
    ]
    .iter()
    .any(|pattern| stderr.contains(pattern))
}

/// First non-empty line of PowerShell stderr — the human-readable message,
/// before the CategoryInfo/position noise
#[cfg(any(target_os = "windows", test))]
fn first_meaningful_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("未知错误")
        .to_string()
}

// ============================================================================
// Windows process execution
// ============================================================================

/// Execute a PowerShell script and return its stdout
#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> AppResult<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // 隐藏控制台窗口，避免每次操作闪现 PowerShell 黑窗
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::Command(format!("执行 PowerShell 失败: {e}")))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_permission_error(&stderr) {
        return Err(AppError::permission_denied("修改网络设置"));
    }
    Err(AppError::Command(first_meaningful_line(&stderr)))
}

#[cfg(target_os = "windows")]
fn run_schtasks(args: &[&str]) -> AppResult<std::process::Output> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    Command::new("schtasks")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::Command(format!("执行 schtasks 失败: {e}")))
}

#[cfg(target_os = "windows")]
fn auto_restore_task_exists() -> AppResult<bool> {
    Ok(run_schtasks(&["/query", "/tn", TASK_NAME])?.status.success())
}

#[cfg(target_os = "windows")]
fn create_auto_restore_task() -> AppResult<()> {
    let app_path = std::env::current_exe()?;
    let action = format!("\"{}\" --restore-mac", app_path.to_string_lossy());
    // /rl HIGHEST：还原需要管理员权限，否则任务会在登录时静默失败
    let output = run_schtasks(&[
        "/create", "/tn", TASK_NAME, "/tr", &action, "/sc", "onlogon", "/rl", "HIGHEST", "/f",
    ])?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_permission_error(&stderr) {
        return Err(AppError::permission_denied("创建开机自动还原任务"));
    }
    Err(AppError::Command(format!(
        "创建计划任务失败: {}",
        first_meaningful_line(&stderr)
    )))
}

#[cfg(target_os = "windows")]
fn delete_auto_restore_task() -> AppResult<()> {
    if !auto_restore_task_exists()? {
        return Ok(());
    }
    let output = run_schtasks(&["/delete", "/tn", TASK_NAME, "/f"])?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_permission_error(&stderr) {
        return Err(AppError::permission_denied("删除开机自动还原任务"));
    }
    Err(AppError::Command(format!(
        "删除计划任务失败: {}",
        first_meaningful_line(&stderr)
    )))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_mac_address_normalizes_separators_and_case() {
        assert_eq!(format_mac_address("aa-bb-cc-dd-ee-ff"), "AA:BB:CC:DD:EE:FF");
        assert_eq!(format_mac_address("AABBCCDDEEFF"), "AA:BB:CC:DD:EE:FF");
        assert_eq!(format_mac_address("aa:bb:cc:dd:ee:ff"), "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn format_mac_address_keeps_invalid_input_trimmed() {
        assert_eq!(format_mac_address(""), "");
        assert_eq!(format_mac_address(" n/a "), "n/a");
        assert_eq!(format_mac_address("AABB"), "AABB");
    }

    #[test]
    fn generate_random_mac_is_unicast_and_locally_administered() {
        for _ in 0..50 {
            let mac = generate_random_mac().unwrap();
            assert_eq!(mac.len(), 12);
            assert!(mac.chars().all(|c| c.is_ascii_hexdigit()));
            let first_byte = u8::from_str_radix(&mac[0..2], 16).unwrap();
            assert_eq!(first_byte & 0x01, 0, "must be unicast");
            assert_eq!(first_byte & 0x02, 0x02, "must be locally administered");
        }
    }

    #[test]
    fn parse_adapter_info_reads_script_json() {
        let info = parse_adapter_info(
            r#"{"name":"以太网","currentMac":"AA-BB-CC-DD-EE-FF","permanentMac":"AABBCCDDEEFF","hasOverride":false}"#,
        )
        .unwrap();
        assert_eq!(info.name, "以太网");
        assert_eq!(info.current_mac, "AA-BB-CC-DD-EE-FF");
        assert_eq!(info.permanent_mac, "AABBCCDDEEFF");
        assert!(!info.has_override);
    }

    #[test]
    fn parse_adapter_info_rejects_empty_output() {
        assert!(parse_adapter_info("  \n").is_err());
    }

    #[test]
    fn mac_info_unmodified_when_current_equals_permanent() {
        let info = mac_info_from_ps(PsAdapterInfo {
            name: "以太网".into(),
            current_mac: "AA-BB-CC-DD-EE-FF".into(),
            permanent_mac: "AABBCCDDEEFF".into(),
            has_override: false,
        });
        assert_eq!(info.current_mac, "AA:BB:CC:DD:EE:FF");
        assert_eq!(info.permanent_mac, "AA:BB:CC:DD:EE:FF");
        assert!(!info.is_modified);
    }

    #[test]
    fn mac_info_modified_when_override_present_or_macs_differ() {
        let by_override = mac_info_from_ps(PsAdapterInfo {
            name: "以太网".into(),
            current_mac: "AA-BB-CC-DD-EE-FF".into(),
            permanent_mac: "AABBCCDDEEFF".into(),
            has_override: true,
        });
        assert!(by_override.is_modified);

        let by_difference = mac_info_from_ps(PsAdapterInfo {
            name: "以太网".into(),
            current_mac: "02-11-22-33-44-55".into(),
            permanent_mac: "AABBCCDDEEFF".into(),
            has_override: false,
        });
        assert!(by_difference.is_modified);
    }

    #[test]
    fn mac_info_not_modified_when_permanent_unknown_without_override() {
        let info = mac_info_from_ps(PsAdapterInfo {
            name: "以太网".into(),
            current_mac: "AA-BB-CC-DD-EE-FF".into(),
            permanent_mac: "".into(),
            has_override: false,
        });
        assert!(!info.is_modified);
    }

    #[test]
    fn permission_errors_are_detected_from_stderr() {
        assert!(is_permission_error("需要管理员权限，请以管理员身份运行本程序"));
        assert!(is_permission_error("Disable-NetAdapter : Access is denied."));
        assert!(!is_permission_error("未找到物理网卡"));
    }

    #[test]
    fn first_meaningful_line_skips_blank_lines() {
        assert_eq!(first_meaningful_line("\n  \n错误信息\n+ CategoryInfo"), "错误信息");
        assert_eq!(first_meaningful_line(""), "未知错误");
    }
}
