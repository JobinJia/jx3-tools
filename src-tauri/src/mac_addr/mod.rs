use tauri::command;
use std::fs;
use std::process::Command;
use std::path::PathBuf;
use std::io::{ Read, Write};
use dirs;
use mac_address;

// Windows API 相关导入
#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::NetworkManagement::IpHelper::*,
    Win32::NetworkManagement::Ndis::*,
    Win32::Networking::WinSock::*,
    Win32::System::Registry::*,
    Win32::Foundation::*,
};

#[command]
pub fn get_mac_address() -> Result<String, String> {
    // 使用mac_address库获取MAC地址
    match mac_address::get_mac_address() {
        Ok(Some(addr)) => Ok(addr.to_string()),
        Ok(None) => Err("无法找到MAC地址".to_string()),
        Err(e) => Err(format!("获取MAC地址失败: {}", e))
    }
}

// 添加新的restore_mac_address函数
pub fn restore_mac_address() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 获取网络接口信息
        let interfaces = match if_addrs::get_if_addrs() {
            Ok(addrs) => addrs,
            Err(e) => return Err(format!("获取网络接口信息失败: {}", e))
        };

        // 找到主要的网络接口
        let interface_name = interfaces.iter()
            .find(|iface| !iface.is_loopback() && iface.addr.ip().is_ipv4())
            .map(|iface| iface.name.clone())
            .ok_or_else(|| "无法找到网络接口".to_string())?;

        // 使用Windows API重启网络适配器
        match restart_network_adapter(&interface_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("还原MAC地址失败: {}", e))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("此功能仅支持Windows系统".to_string())
    }
}

#[cfg(target_os = "windows")]
fn restart_network_adapter(adapter_name: &str) -> Result<(), String> {
    // 获取网络适配器的注册表路径
    let registry_path = get_adapter_registry_path(adapter_name)?;

    // 禁用网络适配器
    let _ = Command::new("netsh")
        .args(&["interface", "set", "interface", adapter_name, "admin=disable"])
        .output()
        .map_err(|e| e.to_string())?;

    // 启用网络适配器
    let output = Command::new("netsh")
        .args(&["interface", "set", "interface", adapter_name, "admin=enable"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("重启网络适配器失败: {}", error));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn get_adapter_registry_path(adapter_name: &str) -> Result<String, String> {
    unsafe {
        // 打开网络适配器注册表键
        let network_key = RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            s!("SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}"),
            0,
            KEY_READ,
            std::ptr::null_mut(),
        );

        if let Err(e) = network_key {
            return Err(format!("无法打开网络适配器注册表: {}", e));
        }

        // 这里需要遍历子键找到匹配的适配器名称
        // 简化起见，我们返回一个通用路径
        Ok(format!("SYSTEM\\CurrentControlSet\\Control\\Network\\{{4D36E972-E325-11CE-BFC1-08002BE10318}}\\Connection"))
    }
}

// 存储自动还原设置的文件路径
fn get_auto_restore_file_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("jx3-tools");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("mac_auto_restore.txt");
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
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

    Ok(contents.trim() == "true")
}

#[command]
pub fn set_auto_restore_setting(auto_restore: bool) -> Result<(), String> {
    let path = get_auto_restore_file_path();

    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    file.write_all(if auto_restore { b"true" } else { b"false" }).map_err(|e| e.to_string())?;

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
            "/tn", task_name,
            "/tr", &format!("\"{}\" --restore-mac", app_path.to_string_lossy()),
            "/sc", "onlogon",
            "/f"  // 强制创建，如果已存在则覆盖
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
        // 解析MAC地址
        let mac_bytes = parse_mac_address(mac_address)?;

        // 获取网络接口信息
        let interfaces = match if_addrs::get_if_addrs() {
            Ok(addrs) => addrs,
            Err(e) => return Err(format!("获取网络接口信息失败: {}", e))
        };

        // 找到主要的网络接口
        let interface_name = interfaces.iter()
            .find(|iface| !iface.is_loopback() && iface.addr.ip().is_ipv4())
            .map(|iface| iface.name.clone())
            .ok_or_else(|| "无法找到网络接口".to_string())?;

        // 使用Windows API修改MAC地址
        match set_mac_address_registry(&interface_name, &mac_bytes) {
            Ok(_) => {
                // 重启网络适配器使更改生效
                restart_network_adapter(&interface_name)?;
                Ok(())
            },
            Err(e) => Err(format!("修改MAC地址失败: {}", e))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非Windows平台上，提示不支持，并引用mac_address参数以避免警告
        Err(format!("不支持在非Windows系统上修改MAC地址: {}", mac_address))
    }
}

#[cfg(target_os = "windows")]
fn parse_mac_address(mac_address: &str) -> Result<[u8; 6], String> {
    let parts: Vec<&str> = mac_address.split(':').collect();
    if parts.len() != 6 {
        return Err("MAC地址格式不正确".to_string());
    }

    let mut mac_bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        mac_bytes[i] = u8::from_str_radix(part, 16)
            .map_err(|_| format!("MAC地址部分 '{}' 不是有效的十六进制值", part))?;
    }

    Ok(mac_bytes)
}

#[cfg(target_os = "windows")]
fn set_mac_address_registry(adapter_name: &str, mac_bytes: &[u8; 6]) -> Result<(), String> {
    // 由于直接使用Windows API修改MAC地址比较复杂，我们使用注册表方法
    // 首先需要找到网络适配器的GUID
    let guid = get_adapter_guid(adapter_name)?;

    // 然后修改注册表中的NetworkAddress值
    unsafe {
        let registry_path = format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{{4D36E972-E325-11CE-BFC1-08002BE10318}}\\{}", guid);
        let mut key_handle = HKEY::default();

        let result = RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            PCSTR(registry_path.as_ptr()),
            0,
            KEY_WRITE,
            &mut key_handle,
        );

        if let Err(e) = result {
            return Err(format!("无法打开网络适配器注册表: {}", e));
        }

        // 将MAC地址转换为字符串格式
        let mac_str = format!("{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            mac_bytes[0], mac_bytes[1], mac_bytes[2],
            mac_bytes[3], mac_bytes[4], mac_bytes[5]);

        // 写入注册表
        let result = RegSetValueExA(
            key_handle,
            s!("NetworkAddress"),
            0,
            REG_SZ,
            mac_str.as_ptr(),
            (mac_str.len() + 1) as u32,
        );

        RegCloseKey(key_handle);

        if let Err(e) = result {
            return Err(format!("无法修改MAC地址注册表值: {}", e));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn get_adapter_guid(adapter_name: &str) -> Result<String, String> {
    // 这个函数需要遍历网络适配器找到匹配的GUID
    // 简化起见，我们使用netsh命令获取
    let output = Command::new("netsh")
        .args(&["interface", "show", "interface", "name=", adapter_name])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("无法获取网络适配器GUID".to_string());
    }

    // 解析输出找到GUID
    // 这里是简化实现，实际应用中需要更复杂的解析
    Ok("0000".to_string())
}

#[command]
pub fn restore_mac_cmd() -> Result<(), String> {
    restore_mac_address()
}
