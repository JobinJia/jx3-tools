use tauri::command;
use std::fs;
use std::process::Command;
use std::path::PathBuf;
use std::io::{ Read, Write};
use dirs;
use mac_address;
use if_addrs;

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

    // 禁用网络适配器
    let _ = Command::new("netsh")
        .args(&["interface", "set", "interface", &interface_name, "admin=disable"])
        .output()
        .map_err(|e| e.to_string())?;

    // 启用网络适配器
    let output = Command::new("netsh")
        .args(&["interface", "set", "interface", &interface_name, "admin=enable"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("还原MAC地址失败: {}", error));
    }

    Ok(())
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

    // 修改MAC地址
    let output = Command::new("netsh")
        .args(&["interface", "set", "interface", &interface_name, "newmac", &mac_address.replace(":", "-")])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("修改MAC地址失败: {}", error));
    }

    Ok(())
}

#[command]
pub fn restore_mac_cmd() -> Result<(), String> {
    restore_mac_address()
}
