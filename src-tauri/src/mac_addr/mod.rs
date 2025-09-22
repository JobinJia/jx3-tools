use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

use tauri::command;

use dirs;
use mac_address;

#[cfg(target_os = "windows")]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::io::ErrorKind;

#[command]
pub fn get_mac_address() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        return get_mac_address_windows();
    }

    #[cfg(not(target_os = "windows"))]
    {
        return get_mac_address_fallback();
    }
}

#[cfg(target_os = "windows")]
fn get_mac_address_windows() -> Result<String, String> {
    let adapter = get_primary_adapter()?;
    Ok(format_mac_address(&adapter.mac_address))
}

#[cfg(not(target_os = "windows"))]
fn get_mac_address_fallback() -> Result<String, String> {
    match mac_address::get_mac_address() {
        Ok(Some(addr)) => Ok(format_mac_address(&addr.to_string())),
        Ok(None) => Err("无法找到MAC地址".to_string()),
        Err(e) => Err(format!("获取MAC地址失败: {}", e)),
    }
}

// 添加新的restore_mac_address函数
pub fn restore_mac_address() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return restore_mac_address_windows();
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("此功能仅支持Windows系统".to_string());
    }
}

#[cfg(target_os = "windows")]
fn restore_mac_address_windows() -> Result<(), String> {
    if let Some(state) = load_mac_state()? {
        match apply_restore(&state.adapter_guid, &state.adapter_name) {
            Ok(_) => {
                if let Err(err) = clear_mac_state() {
                    log::warn!("清理MAC状态文件失败: {}", err);
                }
                return Ok(());
            }
            Err(err) => log::warn!("使用记录的适配器还原失败: {}，尝试自动检测", err),
        }
    }

    let adapter = get_primary_adapter()?;
    apply_restore(&adapter.interface_guid, &adapter.name)?;
    if let Err(err) = clear_mac_state() {
        log::warn!("清理MAC状态文件失败: {}", err);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_restore(adapter_guid: &str, adapter_name: &str) -> Result<(), String> {
    set_network_address_value(adapter_guid, None)?;
    restart_network_adapter(adapter_name)?;
    Ok(())
}

// 存储自动还原设置的文件路径
fn get_auto_restore_file_path() -> PathBuf {
    let mut path = ensure_config_dir();
    path.push("mac_auto_restore.txt");
    path
}

fn ensure_config_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("jx3-tools");
    if let Err(err) = fs::create_dir_all(&path) {
        log::warn!("创建配置目录失败: {}", err);
    }
    path
}

#[command]
pub fn get_auto_restore_setting() -> Result<bool, String> {
    let path = get_auto_restore_file_path();

    if !path.exists() {
        return Ok(false); // 默认不自动还原
    }

    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    Ok(contents.trim() == "true")
}

#[command]
pub fn set_auto_restore_setting(auto_restore: bool) -> Result<(), String> {
    let path = get_auto_restore_file_path();

    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    file.write_all(if auto_restore { b"true" } else { b"false" })
        .map_err(|e| e.to_string())?;

    // 根据设置开机自启动
    if auto_restore {
        setup_auto_restore_on_boot()?;
    } else {
        remove_auto_restore_on_boot()?;
    }

    Ok(())
}

// 设置开机自动还原MAC地址
fn setup_auto_restore_on_boot() -> Result<(), String> {
    // 在Windows上使用任务计划程序
    let app_path = std::env::current_exe().map_err(|e| e.to_string())?;

    let task_name = "JX3ToolsMacRestore";

    // 创建任务计划
    let _ = Command::new("schtasks")
        .args(&[
            "/create",
            "/tn",
            task_name,
            "/tr",
            &format!("\"{}\" --restore-mac", app_path.to_string_lossy()),
            "/sc",
            "onlogon",
            "/f", // 强制创建，如果已存在则覆盖
        ])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 移除开机自动还原MAC地址
fn remove_auto_restore_on_boot() -> Result<(), String> {
    // 在Windows上使用任务计划程序
    let task_name = "JX3ToolsMacRestore";

    // 删除任务计划
    let _ = Command::new("schtasks")
        .args(&["/delete", "/tn", task_name, "/f"])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn change_mac_address(mac_address: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return change_mac_address_windows(mac_address);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = mac_address;
        return Err("此功能仅支持Windows系统".to_string());
    }
}

#[cfg(target_os = "windows")]
fn change_mac_address_windows(mac_address: &str) -> Result<(), String> {
    let adapter = get_primary_adapter()?;
    let sanitized = sanitize_mac_input(mac_address)?;

    set_network_address_value(&adapter.interface_guid, Some(&sanitized))?;
    restart_network_adapter(&adapter.name)?;
    save_mac_state(&MacState {
        adapter_guid: adapter.interface_guid,
        adapter_name: adapter.name,
    })?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn sanitize_mac_input(mac_address: &str) -> Result<String, String> {
    let cleaned: String = mac_address
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':' && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if cleaned.len() != 12 || !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("MAC地址格式不正确".to_string());
    }

    // 第一个字节不能为组播地址
    let first_byte =
        u8::from_str_radix(&cleaned[0..2], 16).map_err(|_| "MAC地址格式不正确".to_string())?;
    if first_byte & 0x01 == 0x01 {
        return Err("MAC地址不能是组播地址".to_string());
    }

    Ok(cleaned)
}

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
#[derive(Debug, Deserialize)]
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

#[cfg(target_os = "windows")]
fn get_primary_adapter() -> Result<AdapterInfo, String> {
    let script = "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and $_.HardwareInterface -eq $true -and -not $_.Virtual } | Sort-Object -Property InterfaceMetric | Select-Object -First 1 -Property Name, InterfaceGuid, MacAddress | ConvertTo-Json -Compress";

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("执行PowerShell失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("无法获取网络适配器信息: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("找不到可用的网络适配器".to_string());
    }

    serde_json::from_str(trimmed).map_err(|e| format!("解析网络适配器信息失败: {}", e))
}

#[cfg(target_os = "windows")]
fn set_network_address_value(interface_guid: &str, value: Option<&str>) -> Result<(), String> {
    let template = "\
$guid = '{GUID}';\
$guidTrimmed = $guid.Trim('{}').ToUpper();\
$normalizedGuid = '{' + $guidTrimmed + '}';\
$classKey = 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4D36E972-E325-11CE-BFC1-08002BE10318}';\
$target = Get-ChildItem $classKey | Where-Object {\
    try {\
        $props = Get-ItemProperty $_.PSPath;\
        if ($null -ne $props -and $props.PSObject.Properties['NetCfgInstanceId']) {\
            $props.NetCfgInstanceId.ToUpper() -eq $normalizedGuid\
        } else {\
            $false\
        }\
    } catch { $false }\
} | Select-Object -First 1;\
if (-not $target) { throw '未找到对应的网络适配器注册表项'; };\
{ACTION}";

    let guid = interface_guid.replace('"', "").replace('\'', "''");
    let template = template.replace("{GUID}", &guid);

    let script = match value {
        Some(mac_value) => {
            let action = "Set-ItemProperty -Path $target.PSPath -Name 'NetworkAddress' -Value '{VALUE}' -Force;";
            let sanitized_value = mac_value.replace(':', "").replace('-', "").to_uppercase();
            template.replace("{ACTION}", &action.replace("{VALUE}", &sanitized_value))
        }
        None => {
            let action = "Remove-ItemProperty -Path $target.PSPath -Name 'NetworkAddress' -ErrorAction SilentlyContinue;";
            template.replace("{ACTION}", action)
        }
    };

    run_powershell(&script)
}

#[cfg(target_os = "windows")]
fn restart_network_adapter(adapter_name: &str) -> Result<(), String> {
    let script = format!(
        "$name = '{}'; Disable-NetAdapter -Name $name -Confirm:$false -ErrorAction Stop; Start-Sleep -Milliseconds 1000; Enable-NetAdapter -Name $name -Confirm:$false -ErrorAction Stop;",
        adapter_name
            .replace('"', "")
            .replace('\'', "''")
    );

    run_powershell(&script)
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<(), String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("执行PowerShell失败: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

#[cfg(target_os = "windows")]
fn save_mac_state(state: &MacState) -> Result<(), String> {
    let path = get_mac_state_file_path();
    let data = serde_json::to_string(state).map_err(|e| format!("保存MAC状态失败: {}", e))?;
    fs::write(path, data).map_err(|e| format!("写入MAC状态失败: {}", e))
}

#[cfg(target_os = "windows")]
fn load_mac_state() -> Result<Option<MacState>, String> {
    let path = get_mac_state_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path).map_err(|e| format!("读取MAC状态失败: {}", e))?;
    let state = serde_json::from_str(&contents).map_err(|e| format!("解析MAC状态失败: {}", e))?;
    Ok(Some(state))
}

#[cfg(target_os = "windows")]
fn clear_mac_state() -> Result<(), String> {
    let path = get_mac_state_file_path();
    match fs::remove_file(&path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(target_os = "windows")]
fn get_mac_state_file_path() -> PathBuf {
    let mut path = ensure_config_dir();
    path.push("mac_state.json");
    path
}

#[command]
pub fn restore_mac_cmd() -> Result<(), String> {
    restore_mac_address()
}
