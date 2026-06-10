//! Interception 驱动安装 / 检测 / 鼠标过滤器清理
//!
//! `install-interception.exe /install` 会同时安装键盘和鼠标两个 class 过滤驱动，
//! 且没有只装键盘的参数。鼠标过滤器曾导致用户鼠标瘫痪（UpperFilters 引用的过滤
//! 器加载失败时，整个鼠标设备栈都起不来），而本工具只需要键盘注入。
//!
//! 因此安装流程固定为：运行官方安装器 → **重启之前**立即从 Mouse class 的
//! `UpperFilters` 移除 `mouse` 项（过滤器只在设备栈重建时加载，重启前移除即
//! 从未生效）。清理失败则自动 `/uninstall` 回滚，绝不让带鼠标过滤器的注册表
//! 状态活到重启。
//!
//! 注意：移除鼠标过滤器后 interception.dll 的 `create_context` 必然失败（它要
//! 求键盘+鼠标全部 20 个设备可打开），所以按键注入不走 dll，由 `keys.rs` 直连
//! `\\.\interception0N` 键盘设备。

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
    use std::os::windows::process::CommandExt;
    use std::path::Path;
    use std::process::Command;

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE,
        KEY_QUERY_VALUE, KEY_SET_VALUE, REG_MULTI_SZ, REG_VALUE_TYPE,
    };

    use super::{encode_multi_sz, parse_multi_sz, DriverState};
    use crate::error::{AppError, AppResult};

    /// Interception 安装器写入的鼠标 class 注册表键（GUID 为系统固定值）
    const MOUSE_CLASS_KEY: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4D36E96F-E325-11CE-BFC1-08002BE10318}";
    /// 键盘 class 注册表键
    const KEYBOARD_CLASS_KEY: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4D36E96B-E325-11CE-BFC1-08002BE10318}";
    const UPPER_FILTERS: &str = "UpperFilters";
    /// Interception 安装器注册的服务名就叫 mouse / keyboard（已从安装器二进制确认）
    const MOUSE_FILTER: &str = "mouse";
    const KEYBOARD_FILTER: &str = "keyboard";

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn reg_err(action: &str, code: WIN32_ERROR) -> AppError {
        AppError::Hotkey(format!("{action}失败（注册表错误码 {}）", code.0))
    }

    struct RegKey(HKEY);

    impl RegKey {
        fn open(subkey: &str, access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Result<Option<Self>, AppError> {
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

    /// 覆写 class 键的 UpperFilters
    fn write_upper_filters(class_key: &str, entries: &[String]) -> AppResult<()> {
        let key = RegKey::open(class_key, KEY_QUERY_VALUE | KEY_SET_VALUE)?
            .ok_or_else(|| AppError::Hotkey(format!("注册表键不存在: {class_key}")))?;
        let name = to_wide(UPPER_FILTERS);
        let buf = encode_multi_sz(entries);
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len() * 2)
        };
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

    /// 仅凭注册表判断键盘过滤器的安装状态（驱动是否真正加载由 keys.rs 判断）
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

    /// 从鼠标 class 的 UpperFilters 移除 interception 鼠标过滤器。
    /// 只删除指定名字的项，其余过滤器（如 mouclass）原样保留。
    pub fn remove_mouse_filter() -> AppResult<()> {
        let filters = match read_upper_filters(MOUSE_CLASS_KEY)? {
            Some(filters) => filters,
            None => return Ok(()),
        };
        if !filters_contain(&filters, MOUSE_FILTER) {
            return Ok(());
        }
        let kept: Vec<String> = filters
            .into_iter()
            .filter(|f| !f.eq_ignore_ascii_case(MOUSE_FILTER))
            .collect();
        if kept.is_empty() {
            // 正常机器上至少还有 mouclass；空列表说明环境异常，拒绝写入以免搞坏鼠标
            return Err(AppError::Hotkey(
                "移除鼠标过滤器后 UpperFilters 将为空，已中止（请手动检查注册表）".into(),
            ));
        }
        write_upper_filters(MOUSE_CLASS_KEY, &kept)?;
        log::info!("已从鼠标 UpperFilters 移除 interception 过滤器，保留: {kept:?}");
        Ok(())
    }

    fn run_installer(installer: &Path, arg: &str) -> AppResult<()> {
        if !installer.exists() {
            return Err(AppError::Hotkey(format!(
                "找不到驱动安装器: {}",
                installer.display()
            )));
        }
        let output = Command::new(installer)
            .arg(arg)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| AppError::Hotkey(format!("运行驱动安装器失败: {e}")))?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::Hotkey(format!(
                "驱动安装器执行失败（{arg}，退出码 {:?}）: {}",
                output.status.code(),
                stdout.trim()
            )));
        }
        Ok(())
    }

    /// 安装按键驱动：官方安装器全装后立即移除鼠标过滤器（重启前移除 = 从未生效）。
    /// 清理失败立刻回滚卸载，绝不带着鼠标过滤器进入下一次重启。
    pub fn install(installer: &Path) -> AppResult<()> {
        run_installer(installer, "/install")?;
        log::info!("Interception 驱动安装完成，开始移除鼠标过滤器");

        if let Err(err) = remove_mouse_filter() {
            log::error!("移除鼠标过滤器失败，回滚卸载驱动: {err}");
            match run_installer(installer, "/uninstall") {
                Ok(()) => Err(AppError::Hotkey(format!(
                    "安装后清理鼠标过滤器失败，已自动卸载驱动（{err}）。请勿重启前重试安装"
                ))),
                Err(rollback_err) => Err(AppError::Hotkey(format!(
                    "清理鼠标过滤器失败（{err}），且回滚卸载也失败（{rollback_err}）。\
                     请勿重启电脑，先以管理员运行 install-interception.exe /uninstall"
                ))),
            }
        } else {
            Ok(())
        }
    }

    /// 卸载按键驱动（官方 /uninstall 会清理键盘+鼠标两侧的注册表与驱动文件）
    pub fn uninstall(installer: &Path) -> AppResult<()> {
        run_installer(installer, "/uninstall")
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
        let buf = multi_sz(&["mouse", "mouclass"]);
        assert_eq!(parse_multi_sz(&buf), vec!["mouse", "mouclass"]);
    }

    #[test]
    fn parse_multi_sz_handles_empty() {
        assert!(parse_multi_sz(&[0]).is_empty());
        assert!(parse_multi_sz(&[]).is_empty());
    }

    #[test]
    fn encode_multi_sz_double_nul_terminated() {
        let buf = multi_sz(&["mouclass"]);
        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf[buf.len() - 2], 0);
        assert_eq!(parse_multi_sz(&buf), vec!["mouclass"]);
    }

    #[test]
    fn roundtrip_preserves_order() {
        let entries = vec!["a".to_string(), "bb".to_string(), "ccc".to_string()];
        assert_eq!(parse_multi_sz(&encode_multi_sz(&entries)), entries);
    }
}
