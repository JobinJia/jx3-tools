//! Windows-specific window enumeration and key sending

use serde::Serialize;

use crate::error::{AppError, AppResult};

/// 窗口信息（用于前端显示）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    pub display_name: String,
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindow, IsWindowVisible, PostMessageW, WM_KEYDOWN, WM_KEYUP,
    };

    use super::WindowInfo;
    use crate::error::{AppError, AppResult};

    /// 枚举所有可见窗口
    pub fn enumerate_windows(filter: Option<&str>) -> AppResult<Vec<WindowInfo>> {
        let mut windows: Vec<WindowInfo> = Vec::new();

        unsafe {
            EnumWindows(Some(enum_window_callback), LPARAM(&mut windows as *mut _ as isize))
                .map_err(|e| AppError::Hotkey(format!("枚举窗口失败: {e}")))?;
        }

        // 应用过滤器
        if let Some(keyword) = filter {
            if !keyword.is_empty() {
                let keyword_lower = keyword.to_lowercase();
                windows.retain(|w| {
                    w.title.to_lowercase().contains(&keyword_lower)
                        || w.process_name.to_lowercase().contains(&keyword_lower)
                        || w.class_name.to_lowercase().contains(&keyword_lower)
                });
            }
        }

        Ok(windows)
    }

    /// 窗口枚举回调
    unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // 跳过不可见窗口
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1); // 继续枚举
        }

        // 获取窗口标题
        let title_len = GetWindowTextLengthW(hwnd);
        if title_len == 0 {
            return BOOL(1); // 跳过无标题窗口
        }

        let mut title_buf: Vec<u16> = vec![0; (title_len + 1) as usize];
        GetWindowTextW(hwnd, &mut title_buf);
        let title = OsString::from_wide(&title_buf)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_string();

        // 跳过空标题
        if title.trim().is_empty() {
            return BOOL(1);
        }

        // 获取窗口类名
        let mut class_buf: Vec<u16> = vec![0; 256];
        GetClassNameW(hwnd, &mut class_buf);
        let class_name = OsString::from_wide(&class_buf)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_string();

        // 获取进程名
        let process_name = get_process_name(hwnd).unwrap_or_default();

        // 构建显示名称
        let display_name = if process_name.is_empty() {
            title.clone()
        } else {
            format!("[{}] {}", process_name, title)
        };

        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
        windows.push(WindowInfo {
            hwnd: hwnd.0 as usize as u64,
            title,
            class_name,
            process_name,
            display_name,
        });

        BOOL(1) // 继续枚举
    }

    /// 获取窗口所属进程名
    unsafe fn get_process_name(hwnd: HWND) -> Option<String> {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buf: Vec<u16> = vec![0; 260];
        let mut size = buf.len() as u32;

        let result = if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut size).is_ok() {
            // 确保 size 不超过 buf 长度
            let actual_size = (size as usize).min(buf.len());
            let path = OsString::from_wide(&buf[..actual_size])
                .to_string_lossy()
                .to_string();
            // 提取文件名
            path.rsplit('\\').next().map(|s| s.to_string())
        } else {
            None
        };

        // 关闭进程句柄，避免资源泄漏
        let _ = CloseHandle(handle);

        result
    }

    /// 将 u64 转换为 HWND
    fn u64_to_hwnd(hwnd: u64) -> HWND {
        HWND(hwnd as *mut std::ffi::c_void)
    }

    /// 检查窗口是否有效
    pub fn is_window_valid(hwnd: u64) -> bool {
        unsafe { IsWindow(u64_to_hwnd(hwnd)).as_bool() }
    }

    /// 向指定窗口发送按键
    pub fn send_key_to_window(hwnd: u64, virtual_key: u16) -> AppResult<()> {
        let hwnd = u64_to_hwnd(hwnd);

        unsafe {
            if !IsWindow(hwnd).as_bool() {
                return Err(AppError::Hotkey("目标窗口已关闭".into()));
            }

            // 发送 WM_KEYDOWN
            PostMessageW(hwnd, WM_KEYDOWN, WPARAM(virtual_key as usize), LPARAM(0))
                .map_err(|e| AppError::Hotkey(format!("发送 WM_KEYDOWN 失败: {e}")))?;

            // 短暂延迟
            std::thread::sleep(std::time::Duration::from_millis(10));

            // 发送 WM_KEYUP (设置 bit 31 和 bit 30 表示 key release)
            PostMessageW(
                hwnd,
                WM_KEYUP,
                WPARAM(virtual_key as usize),
                LPARAM(0xC0000001u32 as isize),
            )
            .map_err(|e| AppError::Hotkey(format!("发送 WM_KEYUP 失败: {e}")))?;
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;

// 非 Windows 平台的占位实现
#[cfg(not(target_os = "windows"))]
pub fn enumerate_windows(_filter: Option<&str>) -> AppResult<Vec<WindowInfo>> {
    Err(AppError::Hotkey("窗口模式仅支持 Windows".into()))
}

#[cfg(not(target_os = "windows"))]
pub fn is_window_valid(_hwnd: u64) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn send_key_to_window(_hwnd: u64, _virtual_key: u16) -> AppResult<()> {
    Err(AppError::Hotkey("窗口模式仅支持 Windows".into()))
}
