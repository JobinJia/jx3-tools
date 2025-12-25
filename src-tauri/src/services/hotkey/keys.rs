use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use rdev::{EventType, Key, simulate};

use crate::error::{AppError, AppResult};

/// 将按键名称解析为 Windows Virtual Key Code (用于窗口模式)
#[cfg(target_os = "windows")]
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

        // 锁定键
        "CAPSLOCK" | "CAPS" => 0x14,

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

/// 将按键名称解析为 rdev Key
pub fn parse_trigger_key(label: &str) -> AppResult<Key> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("触发按键不能为空".into()));
    }
    let upper = trimmed.to_uppercase();

    // 单字符按键
    if upper.len() == 1 {
        let ch = upper.chars().next().unwrap();
        // 字母 A-Z
        if ch.is_ascii_uppercase() {
            return match ch {
                'A' => Ok(Key::KeyA),
                'B' => Ok(Key::KeyB),
                'C' => Ok(Key::KeyC),
                'D' => Ok(Key::KeyD),
                'E' => Ok(Key::KeyE),
                'F' => Ok(Key::KeyF),
                'G' => Ok(Key::KeyG),
                'H' => Ok(Key::KeyH),
                'I' => Ok(Key::KeyI),
                'J' => Ok(Key::KeyJ),
                'K' => Ok(Key::KeyK),
                'L' => Ok(Key::KeyL),
                'M' => Ok(Key::KeyM),
                'N' => Ok(Key::KeyN),
                'O' => Ok(Key::KeyO),
                'P' => Ok(Key::KeyP),
                'Q' => Ok(Key::KeyQ),
                'R' => Ok(Key::KeyR),
                'S' => Ok(Key::KeyS),
                'T' => Ok(Key::KeyT),
                'U' => Ok(Key::KeyU),
                'V' => Ok(Key::KeyV),
                'W' => Ok(Key::KeyW),
                'X' => Ok(Key::KeyX),
                'Y' => Ok(Key::KeyY),
                'Z' => Ok(Key::KeyZ),
                _ => Err(AppError::Hotkey(format!("暂不支持的触发按键: {trimmed}"))),
            };
        }
        // 数字 0-9
        if ch.is_ascii_digit() {
            return match ch {
                '0' => Ok(Key::Num0),
                '1' => Ok(Key::Num1),
                '2' => Ok(Key::Num2),
                '3' => Ok(Key::Num3),
                '4' => Ok(Key::Num4),
                '5' => Ok(Key::Num5),
                '6' => Ok(Key::Num6),
                '7' => Ok(Key::Num7),
                '8' => Ok(Key::Num8),
                '9' => Ok(Key::Num9),
                _ => Err(AppError::Hotkey(format!("暂不支持的触发按键: {trimmed}"))),
            };
        }
    }

    let key = match upper.as_str() {
        // 控制键
        "SPACE" => Key::Space,
        "ENTER" | "RETURN" => Key::Return,
        "TAB" => Key::Tab,
        "ESC" | "ESCAPE" => Key::Escape,
        "BACKSPACE" => Key::Backspace,
        "DELETE" | "DEL" => Key::Delete,

        // 方向键
        "UP" | "ARROWUP" => Key::UpArrow,
        "DOWN" | "ARROWDOWN" => Key::DownArrow,
        "LEFT" | "ARROWLEFT" => Key::LeftArrow,
        "RIGHT" | "ARROWRIGHT" => Key::RightArrow,

        // 功能键 F1-F12
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,

        // 导航键
        "HOME" => Key::Home,
        "END" => Key::End,
        "PAGEUP" => Key::PageUp,
        "PAGEDOWN" => Key::PageDown,
        "INSERT" => Key::Insert,

        // 小键盘数字键
        "NUM0" | "NUMPAD0" => Key::Kp0,
        "NUM1" | "NUMPAD1" => Key::Kp1,
        "NUM2" | "NUMPAD2" => Key::Kp2,
        "NUM3" | "NUMPAD3" => Key::Kp3,
        "NUM4" | "NUMPAD4" => Key::Kp4,
        "NUM5" | "NUMPAD5" => Key::Kp5,
        "NUM6" | "NUMPAD6" => Key::Kp6,
        "NUM7" | "NUMPAD7" => Key::Kp7,
        "NUM8" | "NUMPAD8" => Key::Kp8,
        "NUM9" | "NUMPAD9" => Key::Kp9,

        // 小键盘运算符
        "NUMADD" | "NUMPLUS" => Key::KpPlus,
        "NUMSUB" | "NUMMINUS" => Key::KpMinus,
        "NUMMUL" | "NUMSTAR" | "NUMMULTIPLY" => Key::KpMultiply,
        "NUMDIV" | "NUMSLASH" | "NUMDIVIDE" => Key::KpDivide,
        // 小键盘小数点 - rdev 没有专门的 KpDecimal，使用 Delete 作为替代
        "NUMDOT" | "NUMDECIMAL" => Key::Delete,

        // 锁定键
        "CAPSLOCK" | "CAPS" => Key::CapsLock,
        "NUMLOCK" => Key::NumLock,
        "SCROLLLOCK" => Key::ScrollLock,

        // 修饰键
        "ALT" => Key::Alt,
        "CTRL" | "CONTROL" => Key::ControlLeft,
        "SHIFT" => Key::ShiftLeft,
        "LSHIFT" | "LEFTSHIFT" => Key::ShiftLeft,
        "RSHIFT" | "RIGHTSHIFT" => Key::ShiftRight,
        "LCTRL" | "LCONTROL" | "LEFTCTRL" => Key::ControlLeft,
        "RCTRL" | "RCONTROL" | "RIGHTCTRL" => Key::ControlRight,
        "WIN" | "META" | "SUPER" | "WINDOWS" => Key::MetaLeft,
        #[cfg(target_os = "macos")]
        "CMD" | "COMMAND" => Key::MetaLeft,
        #[cfg(target_os = "macos")]
        "OPTION" | "OPT" => Key::Alt,

        _ => return Err(AppError::Hotkey(format!("暂不支持的触发按键: {trimmed}"))),
    };
    Ok(key)
}

/// 模拟按键点击 (按下 + 释放)
pub fn simulate_key_click(key: Key) -> AppResult<()> {
    // 按下
    simulate(&EventType::KeyPress(key))
        .map_err(|e| AppError::Hotkey(format!("发送按键按下失败: {:?}", e)))?;

    // 短暂延迟
    thread::sleep(Duration::from_millis(10));

    // 释放
    simulate(&EventType::KeyRelease(key))
        .map_err(|e| AppError::Hotkey(format!("发送按键释放失败: {:?}", e)))?;

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
