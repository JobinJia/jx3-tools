//! 按键模拟模块 - 使用 Windows SendInput API
//!
//! 使用 SendInput 替代 rdev，提供更可靠的按键模拟

#![cfg(target_os = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crate::error::{AppError, AppResult};

/// 将按键名称解析为 Windows Virtual Key Code
pub fn parse_to_virtual_key(label: &str) -> AppResult<u16> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("触发按键不能为空".into()));
    }
    let upper = trimmed.to_uppercase();

    // 单字符按键 A-Z, 0-9
    if upper.len() == 1 {
        let ch = upper.chars().next().unwrap();
        if ch.is_ascii_uppercase() {
            return Ok(ch as u16); // VK_A = 0x41, etc.
        }
        if ch.is_ascii_digit() {
            return Ok(ch as u16); // VK_0 = 0x30, etc.
        }
    }

    // 特殊按键映射
    let vk = match upper.as_str() {
        "SPACE" => 0x20,            // VK_SPACE
        "ENTER" | "RETURN" => 0x0D, // VK_RETURN
        "TAB" => 0x09,              // VK_TAB
        "ESC" | "ESCAPE" => 0x1B,   // VK_ESCAPE
        "BACKSPACE" => 0x08,        // VK_BACK
        "DELETE" | "DEL" => 0x2E,   // VK_DELETE

        // 方向键
        "UP" | "ARROWUP" => 0x26,
        "DOWN" | "ARROWDOWN" => 0x28,
        "LEFT" | "ARROWLEFT" => 0x25,
        "RIGHT" | "ARROWRIGHT" => 0x27,

        // 功能键 F1-F20
        "F1" => 0x70,
        "F2" => 0x71,
        "F3" => 0x72,
        "F4" => 0x73,
        "F5" => 0x74,
        "F6" => 0x75,
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "F11" => 0x7A,
        "F12" => 0x7B,
        "F13" => 0x7C,
        "F14" => 0x7D,
        "F15" => 0x7E,
        "F16" => 0x7F,
        "F17" => 0x80,
        "F18" => 0x81,
        "F19" => 0x82,
        "F20" => 0x83,

        // 小键盘
        "NUM0" | "NUMPAD0" => 0x60,
        "NUM1" | "NUMPAD1" => 0x61,
        "NUM2" | "NUMPAD2" => 0x62,
        "NUM3" | "NUMPAD3" => 0x63,
        "NUM4" | "NUMPAD4" => 0x64,
        "NUM5" | "NUMPAD5" => 0x65,
        "NUM6" | "NUMPAD6" => 0x66,
        "NUM7" | "NUMPAD7" => 0x67,
        "NUM8" | "NUMPAD8" => 0x68,
        "NUM9" | "NUMPAD9" => 0x69,
        "NUMADD" | "NUMPLUS" => 0x6B,
        "NUMSUB" | "NUMMINUS" => 0x6D,
        "NUMMUL" | "NUMSTAR" | "NUMMULTIPLY" => 0x6A,
        "NUMDIV" | "NUMSLASH" | "NUMDIVIDE" => 0x6F,
        "NUMDOT" | "NUMDECIMAL" => 0x6E,

        // 导航键
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" => 0x21,
        "PAGEDOWN" => 0x22,
        "INSERT" => 0x2D,

        // 锁定键
        "CAPSLOCK" | "CAPS" => 0x14,
        "NUMLOCK" => 0x90,
        "SCROLLLOCK" => 0x91,

        // 修饰键
        "ALT" => 0x12,
        "CTRL" | "CONTROL" => 0x11,
        "SHIFT" => 0x10,
        "LSHIFT" | "LEFTSHIFT" => 0xA0,
        "RSHIFT" | "RIGHTSHIFT" => 0xA1,
        "LCTRL" | "LCONTROL" | "LEFTCTRL" => 0xA2,
        "RCTRL" | "RCONTROL" | "RIGHTCTRL" => 0xA3,
        "WIN" | "META" | "SUPER" | "WINDOWS" => 0x5B,

        _ => return Err(AppError::Hotkey(format!("暂不支持的触发按键: {trimmed}"))),
    };

    Ok(vk)
}

/// 使用 SendInput API 模拟按键点击 (按下 + 释放)
pub fn simulate_key_click(vk: u16) -> AppResult<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        VIRTUAL_KEY,
    };

    let scan_code = unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(
            vk as u32,
            windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC,
        ) as u16
    };

    // 构造按键按下事件
    let key_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan_code,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    // 构造按键释放事件
    let key_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan_code,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        // 发送按键按下
        let sent = SendInput(&[key_down], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(AppError::Hotkey("发送按键按下失败".into()));
        }

        // 短暂延迟
        thread::sleep(Duration::from_millis(10));

        // 发送按键释放
        let sent = SendInput(&[key_up], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(AppError::Hotkey("发送按键释放失败".into()));
        }
    }

    Ok(())
}

/// Sleep with interrupt capability
pub fn sleep_with_interrupt(flag: &Arc<AtomicBool>, total_ms: u64) {
    let mut remaining = if total_ms == 0 { 1 } else { total_ms };
    while remaining > 0 && !flag.load(Ordering::SeqCst) {
        let step = remaining.min(50);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}
