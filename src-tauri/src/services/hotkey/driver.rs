//! Interception 键盘驱动安装 / 卸载 / 状态检测（**只装键盘，从不碰鼠标**）
//!
//! 历史教训：官方 `install-interception.exe /install` 会**同时**装键盘和鼠标两个
//! class 过滤驱动，没有只装键盘的开关；鼠标过滤器曾把用户鼠标搞瘫（成功加载后
//! 在转发 IRP 时对某些鼠标失灵）。本工具只需要键盘注入，因此**不再调用官方安装
//! 器**，改为自己手动只做键盘那一半：
//!   1. 把随包的已签名 `keyboard.sys` 拷到 `System32\drivers\`
//!   2. 用 SCM 注册内核驱动服务 `keyboard`（DEMAND_START + ERROR_NORMAL）
//!   3. 往**键盘** class（GUID `{4D36E96B-…}`）的 `UpperFilters` 加入 `keyboard`
//! 全程不写任何 `mouse` 服务 / `mouse.sys` / 鼠标 class，鼠标事故链根本不存在。
//!
//! 安全要点：`ErrorControl=SERVICE_ERROR_NORMAL` 保证万一驱动加载失败，PnP 会
//! 跳过该过滤器继续启动键盘设备——最坏只是「注入不生效」，键盘本身照常可用。
//!
//! 注入侧不用 `interception.dll`（它的 `create_context` 要求 20 个键鼠设备全开，
//! 只装键盘必然失败），由 `keys.rs` 直连 `\\.\interception0N` 键盘设备完成。

use serde::Serialize;

/// 按键驱动安装状态
// 非 Windows 下只会构造 NotInstalled，其余变体仅为序列化契约存在
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DriverState {
    /// 内核驱动已加载，可注入按键
    Ready,
    /// 注册表已有键盘过滤器但驱动未加载（等待重启）
    PendingReboot,
    #[default]
    NotInstalled,
}

/// 解析 REG_MULTI_SZ 缓冲（UTF-16，双 NUL 结尾）为字符串列表
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn parse_multi_sz(buf: &[u16]) -> Vec<String> {
    buf.split(|&c| c == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf16_lossy(s))
        .collect()
}

/// 编码字符串列表为 REG_MULTI_SZ 缓冲（每项 NUL 结尾，整体再补一个 NUL）
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn encode_multi_sz(entries: &[String]) -> Vec<u16> {
    let mut buf: Vec<u16> = Vec::new();
    for entry in entries {
        buf.extend(entry.encode_utf16());
        buf.push(0);
    }
    buf.push(0);
    buf
}

#[cfg(target_os = "windows")]
pub use windows_impl::{install, mouse_filter_present, registry_state, remove_mouse_filter, uninstall};

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::path::{Path, PathBuf};

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SERVICE_EXISTS, ERROR_SUCCESS, WIN32_ERROR};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
        HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_MULTI_SZ, REG_SAM_FLAGS,
        REG_VALUE_TYPE,
    };
    use windows::Win32::System::Services::{
        CloseServiceHandle, CreateServiceW, DeleteService, OpenSCManagerW, OpenServiceW,
        StartServiceW, SC_HANDLE, SC_MANAGER_CREATE_SERVICE, SERVICE_ALL_ACCESS,
        SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER,
    };

    use super::{encode_multi_sz, parse_multi_sz, DriverState};
    use crate::error::{AppError, AppResult};

    /// 键盘 class 注册表键（GUID 为系统固定值）
    const KEYBOARD_CLASS_KEY: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4D36E96B-E325-11CE-BFC1-08002BE10318}";
    /// 鼠标 class 注册表键（仅用于检测/清理旧版遗留，安装时绝不写入）
    const MOUSE_CLASS_KEY: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4D36E96F-E325-11CE-BFC1-08002BE10318}";
    const UPPER_FILTERS: &str = "UpperFilters";
    /// 过滤驱动服务名 = .sys 基名 = UpperFilters 项名，三者必须一致
    const KEYBOARD_FILTER: &str = "keyboard";
    const MOUSE_FILTER: &str = "mouse";

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn reg_err(action: &str, code: WIN32_ERROR) -> AppError {
        AppError::Hotkey(format!("{action}失败（注册表错误码 {}）", code.0))
    }

    // ───────────────────────── 注册表 UpperFilters ─────────────────────────

    struct RegKey(HKEY);

    impl RegKey {
        fn open(subkey: &str, access: REG_SAM_FLAGS) -> Result<Option<Self>, AppError> {
            let wide = to_wide(subkey);
            let mut hkey = HKEY::default();
            let res = unsafe {
                RegOpenKeyExW(HKEY_LOCAL_MACHINE, PCWSTR(wide.as_ptr()), Some(0), access, &mut hkey)
            };
            if res == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if res != ERROR_SUCCESS {
                return Err(reg_err("打开注册表键", res));
            }
            Ok(Some(Self(hkey)))
        }
    }

    impl Drop for RegKey {
        fn drop(&mut self) {
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }

    /// 读取 class 键的 UpperFilters（键或值不存在返回 None）
    fn read_upper_filters(class_key: &str) -> AppResult<Option<Vec<String>>> {
        let key = match RegKey::open(class_key, KEY_QUERY_VALUE)? {
            Some(key) => key,
            None => return Ok(None),
        };
        let name = to_wide(UPPER_FILTERS);
        let mut value_type = REG_VALUE_TYPE::default();
        let mut size: u32 = 0;
        let res = unsafe {
            RegQueryValueExW(
                key.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut size),
            )
        };
        if res == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if res != ERROR_SUCCESS {
            return Err(reg_err("读取 UpperFilters", res));
        }
        if value_type != REG_MULTI_SZ {
            return Err(AppError::Hotkey(format!(
                "UpperFilters 类型异常（{}），拒绝改动",
                value_type.0
            )));
        }

        let mut buf: Vec<u16> = vec![0; (size as usize).div_ceil(2)];
        let mut size_in = size;
        let res = unsafe {
            RegQueryValueExW(
                key.0,
                PCWSTR(name.as_ptr()),
                None,
                None,
                Some(buf.as_mut_ptr().cast()),
                Some(&mut size_in),
            )
        };
        if res != ERROR_SUCCESS {
            return Err(reg_err("读取 UpperFilters", res));
        }
        Ok(Some(parse_multi_sz(&buf)))
    }

    /// 覆写 class 键的 UpperFilters（entries 为空则删除该值）
    fn write_upper_filters(class_key: &str, entries: &[String]) -> AppResult<()> {
        let key = RegKey::open(class_key, KEY_QUERY_VALUE | KEY_SET_VALUE)?
            .ok_or_else(|| AppError::Hotkey(format!("注册表键不存在: {class_key}")))?;
        let name = to_wide(UPPER_FILTERS);

        if entries.is_empty() {
            let res = unsafe { RegDeleteValueW(key.0, PCWSTR(name.as_ptr())) };
            if res != ERROR_SUCCESS && res != ERROR_FILE_NOT_FOUND {
                return Err(reg_err("删除 UpperFilters", res));
            }
            return Ok(());
        }

        let buf = encode_multi_sz(entries);
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len() * 2) };
        let res = unsafe {
            RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, REG_MULTI_SZ, Some(bytes))
        };
        if res != ERROR_SUCCESS {
            return Err(reg_err("写入 UpperFilters", res));
        }
        Ok(())
    }

    fn filters_contain(filters: &[String], name: &str) -> bool {
        filters.iter().any(|f| f.eq_ignore_ascii_case(name))
    }

    /// 在 class 的 UpperFilters 末尾加入指定过滤器（已存在则不动）
    fn add_filter(class_key: &str, filter: &str) -> AppResult<()> {
        let mut filters = read_upper_filters(class_key)?.unwrap_or_default();
        if filters_contain(&filters, filter) {
            return Ok(());
        }
        filters.push(filter.to_string());
        write_upper_filters(class_key, &filters)?;
        log::info!("已向 {class_key} UpperFilters 加入 {filter}");
        Ok(())
    }

    /// 从 class 的 UpperFilters 移除指定过滤器（其余项原样保留）
    fn remove_filter(class_key: &str, filter: &str) -> AppResult<()> {
        let filters = match read_upper_filters(class_key)? {
            Some(filters) => filters,
            None => return Ok(()),
        };
        if !filters_contain(&filters, filter) {
            return Ok(());
        }
        let kept: Vec<String> = filters
            .into_iter()
            .filter(|f| !f.eq_ignore_ascii_case(filter))
            .collect();
        write_upper_filters(class_key, &kept)?;
        log::info!("已从 {class_key} UpperFilters 移除 {filter}，保留: {kept:?}");
        Ok(())
    }

    /// 鼠标 class 的 UpperFilters 是否残留 interception 鼠标过滤器
    pub fn mouse_filter_present() -> bool {
        match read_upper_filters(MOUSE_CLASS_KEY) {
            Ok(Some(filters)) => filters_contain(&filters, MOUSE_FILTER),
            Ok(None) => false,
            Err(err) => {
                log::warn!("检查鼠标过滤器失败: {err}");
                false
            }
        }
    }

    /// 移除残留的 interception 鼠标过滤器（旧版安装包全装遗留）
    pub fn remove_mouse_filter() -> AppResult<()> {
        remove_filter(MOUSE_CLASS_KEY, MOUSE_FILTER)
    }

    /// 仅凭注册表判断键盘过滤器的安装状态（驱动是否真加载由 keys.rs 判断）
    pub fn registry_state() -> DriverState {
        match read_upper_filters(KEYBOARD_CLASS_KEY) {
            Ok(Some(filters)) if filters_contain(&filters, KEYBOARD_FILTER) => {
                DriverState::PendingReboot
            }
            Ok(_) => DriverState::NotInstalled,
            Err(err) => {
                log::warn!("检查键盘过滤器失败: {err}");
                DriverState::NotInstalled
            }
        }
    }

    // ───────────────────────── 驱动文件 ─────────────────────────

    /// `%SystemRoot%\System32\drivers\keyboard.sys` 的目标路径
    fn driver_dest_path() -> PathBuf {
        let system_root = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        system_root
            .join("System32")
            .join("drivers")
            .join("keyboard.sys")
    }

    fn copy_driver(src: &Path) -> AppResult<()> {
        if !src.exists() {
            return Err(AppError::Hotkey(format!(
                "找不到驱动文件: {}",
                src.display()
            )));
        }
        let dest = driver_dest_path();
        std::fs::copy(src, &dest).map_err(|e| {
            AppError::Hotkey(format!("拷贝驱动到 {} 失败: {e}", dest.display()))
        })?;
        log::info!("驱动已拷贝到 {}", dest.display());
        Ok(())
    }

    fn delete_driver_file() {
        let dest = driver_dest_path();
        match std::fs::remove_file(&dest) {
            Ok(()) => log::info!("已删除驱动文件 {}", dest.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => log::warn!("删除驱动文件 {} 失败: {e}", dest.display()),
        }
    }

    // ───────────────────────── SCM 服务 ─────────────────────────

    struct ScHandle(SC_HANDLE);

    impl Drop for ScHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseServiceHandle(self.0);
            }
        }
    }

    /// 注册内核驱动服务 `keyboard`（已存在视为成功）
    fn create_keyboard_service() -> AppResult<()> {
        let scm = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CREATE_SERVICE) }
            .map_err(|e| AppError::Hotkey(format!("打开服务管理器失败: {e}")))?;
        let scm = ScHandle(scm);

        let name = to_wide(KEYBOARD_FILTER);
        // 内核驱动 ImagePath 缺省即 \SystemRoot\System32\drivers\<服务名>.sys，
        // 这里显式给出以免歧义
        let bin_path = to_wide(r"\SystemRoot\System32\drivers\keyboard.sys");

        let res = unsafe {
            CreateServiceW(
                scm.0,
                PCWSTR(name.as_ptr()),
                PCWSTR(name.as_ptr()),
                SERVICE_ALL_ACCESS,
                SERVICE_KERNEL_DRIVER,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                PCWSTR(bin_path.as_ptr()),
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            )
        };

        match res {
            Ok(svc) => {
                let _ = ScHandle(svc);
                log::info!("已注册键盘驱动服务");
                Ok(())
            }
            Err(e) if e.code() == ERROR_SERVICE_EXISTS.to_hresult() => {
                log::info!("键盘驱动服务已存在，跳过创建");
                Ok(())
            }
            Err(e) => Err(AppError::Hotkey(format!("注册键盘驱动服务失败: {e}"))),
        }
    }

    /// 删除指定内核驱动服务（不存在视为成功）
    fn delete_service(service: &str) -> AppResult<()> {
        let scm = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CREATE_SERVICE) }
            .map_err(|e| AppError::Hotkey(format!("打开服务管理器失败: {e}")))?;
        let scm = ScHandle(scm);

        let name = to_wide(service);
        // SERVICE_ALL_ACCESS 已含 DELETE 权限
        let svc = match unsafe { OpenServiceW(scm.0, PCWSTR(name.as_ptr()), SERVICE_ALL_ACCESS) } {
            Ok(svc) => ScHandle(svc),
            // 服务不存在
            Err(_) => return Ok(()),
        };
        unsafe { DeleteService(svc.0) }
            .map_err(|e| AppError::Hotkey(format!("删除服务 {service} 失败: {e}")))?;
        log::info!("已删除驱动服务 {service}");
        Ok(())
    }

    // ───────────────────────── 安装 / 卸载 ─────────────────────────

    /// 安装按键驱动：运行官方 install-interception.exe /install，然后剥离鼠标过滤器。
    pub fn install(installer_exe: &Path) -> AppResult<()> {
        if !installer_exe.exists() {
            return Err(AppError::Hotkey(format!(
                "找不到驱动安装器: {}",
                installer_exe.display()
            )));
        }

        let output = std::process::Command::new(installer_exe)
            .arg("/install")
            .output()
            .map_err(|e| AppError::Hotkey(format!("启动驱动安装器失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::Hotkey(format!(
                "驱动安装器返回错误（exit {}）: {} {}",
                output.status.code().unwrap_or(-1),
                stdout.trim(),
                stderr.trim()
            )));
        }
        log::info!("官方安装器执行成功");

        strip_mouse_filter();

        // 尝试立刻启动驱动服务（让内核加载驱动、创建 interception 设备）。
        // 这只是 SCM 级别的 StartService，不碰任何设备栈、不重启键盘。
        // 如果失败也没关系——重启电脑后一定会加载。
        start_driver_service();

        log::info!("键盘驱动安装完成（鼠标过滤器已剥离）");
        Ok(())
    }

    /// 尝试启动 keyboard 驱动服务（让内核立刻加载驱动）。
    /// 已在运行或失败都静默处理——不影响安装结果。
    fn start_driver_service() {
        let Ok(scm) =
            (unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CREATE_SERVICE) })
        else {
            return;
        };
        let scm = ScHandle(scm);
        let name = to_wide(KEYBOARD_FILTER);
        let Ok(svc) =
            (unsafe { OpenServiceW(scm.0, PCWSTR(name.as_ptr()), SERVICE_ALL_ACCESS) })
        else {
            return;
        };
        let svc = ScHandle(svc);
        match unsafe { StartServiceW(svc.0, None) } {
            Ok(()) => log::info!("驱动服务已启动（内核加载驱动）"),
            Err(e) => log::debug!("启动驱动服务: {e}（可能已在运行）"),
        }
    }

    fn strip_mouse_filter() {
        if let Err(e) = remove_filter(MOUSE_CLASS_KEY, MOUSE_FILTER) {
            log::warn!("剥离鼠标 UpperFilters 失败: {e}");
        }
        if let Err(e) = delete_service(MOUSE_FILTER) {
            log::warn!("删除鼠标服务失败: {e}");
        }
        let mouse_sys = driver_dest_path().with_file_name("mouse.sys");
        if mouse_sys.exists() {
            if let Err(e) = std::fs::remove_file(&mouse_sys) {
                log::warn!("删除 mouse.sys 失败: {e}");
            }
        }
        log::info!("鼠标过滤器已剥离");
    }

    /// 卸载按键驱动：移除键盘 UpperFilters → 删服务 → 删驱动文件；
    /// 顺带清理旧版全装可能残留的鼠标过滤器/服务/文件。需重启后彻底生效。
    pub fn uninstall() -> AppResult<()> {
        // 键盘侧：先摘过滤器引用，再删服务与文件
        remove_filter(KEYBOARD_CLASS_KEY, KEYBOARD_FILTER)?;
        delete_service(KEYBOARD_FILTER)?;
        delete_driver_file();

        // 旧版遗留鼠标侧：尽力清理，不影响键盘卸载结果
        if let Err(err) = remove_filter(MOUSE_CLASS_KEY, MOUSE_FILTER) {
            log::warn!("清理残留鼠标过滤器失败: {err}");
        }
        if let Err(err) = delete_service(MOUSE_FILTER) {
            log::warn!("清理残留鼠标服务失败: {err}");
        }
        let mouse_sys = driver_dest_path().with_file_name("mouse.sys");
        if mouse_sys.exists() {
            if let Err(e) = std::fs::remove_file(&mouse_sys) {
                log::warn!("删除残留 mouse.sys 失败: {e}");
            }
        }

        log::info!("键盘驱动卸载完成，重启电脑后彻底生效");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn multi_sz(entries: &[&str]) -> Vec<u16> {
        encode_multi_sz(&entries.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parse_multi_sz_splits_entries() {
        let buf = multi_sz(&["keyboard", "kbdclass"]);
        assert_eq!(parse_multi_sz(&buf), vec!["keyboard", "kbdclass"]);
    }

    #[test]
    fn parse_multi_sz_handles_empty() {
        assert!(parse_multi_sz(&[0]).is_empty());
        assert!(parse_multi_sz(&[]).is_empty());
    }

    #[test]
    fn encode_multi_sz_double_nul_terminated() {
        let buf = multi_sz(&["keyboard"]);
        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf[buf.len() - 2], 0);
        assert_eq!(parse_multi_sz(&buf), vec!["keyboard"]);
    }

    #[test]
    fn roundtrip_preserves_order() {
        let entries = vec!["a".to_string(), "bb".to_string(), "ccc".to_string()];
        assert_eq!(parse_multi_sz(&encode_multi_sz(&entries)), entries);
    }
}
