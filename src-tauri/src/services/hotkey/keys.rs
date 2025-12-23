use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[cfg(target_os = "macos")]
use std::sync::mpsc;
#[cfg(target_os = "macos")]
use tauri::AppHandle;

use crate::error::{AppError, AppResult};

/// 将按键名称解析为 Windows Virtual Key Code
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

/// Parse a key label into an enigo Key
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn parse_trigger_key(label: &str) -> AppResult<Key> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("触发按键不能为空".into()));
    }
    let upper = trimmed.to_uppercase();

    // Single character keys
    if upper.len() == 1 {
        if let Some(ch) = upper.chars().next() {
            return Ok(Key::Unicode(ch));
        }
    }

    let key = match upper.as_str() {
        // 控制键
        "SPACE" => Key::Space,
        "ENTER" | "RETURN" => Key::Return,
        "TAB" => Key::Tab,
        "ESC" | "ESCAPE" => Key::Escape,

        // 方向键
        "UP" | "ARROWUP" => Key::UpArrow,
        "DOWN" | "ARROWDOWN" => Key::DownArrow,
        "LEFT" | "ARROWLEFT" => Key::LeftArrow,
        "RIGHT" | "ARROWRIGHT" => Key::RightArrow,

        // 功能键 F1-F20 (跨平台)
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
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,

        // macOS Function 键
        #[cfg(target_os = "macos")]
        "FUNCTION" | "FN" => Key::Function,

        // 导航键
        "HOME" => Key::Home,
        "END" => Key::End,
        "PAGEUP" => Key::PageUp,
        "PAGEDOWN" => Key::PageDown,
        "DELETE" | "DEL" => Key::Delete,
        "BACKSPACE" => Key::Backspace,

        // 小键盘数字键
        "NUM0" | "NUMPAD0" => Key::Numpad0,
        "NUM1" | "NUMPAD1" => Key::Numpad1,
        "NUM2" | "NUMPAD2" => Key::Numpad2,
        "NUM3" | "NUMPAD3" => Key::Numpad3,
        "NUM4" | "NUMPAD4" => Key::Numpad4,
        "NUM5" | "NUMPAD5" => Key::Numpad5,
        "NUM6" | "NUMPAD6" => Key::Numpad6,
        "NUM7" | "NUMPAD7" => Key::Numpad7,
        "NUM8" | "NUMPAD8" => Key::Numpad8,
        "NUM9" | "NUMPAD9" => Key::Numpad9,

        // 小键盘运算符
        "NUMADD" | "NUMPLUS" => Key::Add,
        "NUMSUB" | "NUMMINUS" => Key::Subtract,
        "NUMMUL" | "NUMSTAR" | "NUMMULTIPLY" => Key::Multiply,
        "NUMDIV" | "NUMSLASH" | "NUMDIVIDE" => Key::Divide,
        "NUMDOT" | "NUMDECIMAL" => Key::Decimal,

        // 锁定键
        "CAPSLOCK" | "CAPS" => Key::CapsLock,

        // 修饰键
        "ALT" => Key::Alt,
        "CTRL" | "CONTROL" => Key::Control,
        "SHIFT" => Key::Shift,
        "LSHIFT" | "LEFTSHIFT" => Key::LShift,
        "RSHIFT" | "RIGHTSHIFT" => Key::RShift,
        "LCTRL" | "LCONTROL" | "LEFTCTRL" => Key::LControl,
        "RCTRL" | "RCONTROL" | "RIGHTCTRL" => Key::RControl,
        "WIN" | "META" | "SUPER" | "WINDOWS" => Key::Meta,
        #[cfg(target_os = "macos")]
        "CMD" | "COMMAND" => Key::Meta,
        #[cfg(target_os = "macos")]
        "OPTION" | "OPT" => Key::Option,

        // 媒体键
        "PLAYPAUSE" | "MEDIAPLAYPAUSE" => Key::MediaPlayPause,
        "NEXTTRACK" | "MEDIANEXT" | "MEDIANEXTTRACK" => Key::MediaNextTrack,
        "PREVTRACK" | "MEDIAPREV" | "MEDIAPREVTRACK" => Key::MediaPrevTrack,
        "VOLUMEUP" | "VOLUP" => Key::VolumeUp,
        "VOLUMEDOWN" | "VOLDOWN" => Key::VolumeDown,
        "MUTE" | "VOLUMEMUTE" => Key::VolumeMute,

        _ => return Err(AppError::Hotkey(format!("暂不支持的触发按键: {trimmed}"))),
    };
    Ok(key)
}

/// Fallback key parser for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn parse_trigger_key(label: &str) -> AppResult<Key> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("触发按键不能为空".into()));
    }
    let upper = trimmed.to_uppercase();

    if upper.len() == 1 {
        if let Some(ch) = upper.chars().next() {
            return Ok(Key::Unicode(ch));
        }
    }

    match upper.as_str() {
        "SPACE" => Ok(Key::Space),
        "ENTER" => Ok(Key::Return),
        "TAB" => Ok(Key::Tab),
        "ESC" | "ESCAPE" => Ok(Key::Escape),
        _ => Err(AppError::Hotkey("热键功能仅支持 Windows 或 macOS".into())),
    }
}

/// Send a key press on Windows
#[cfg(target_os = "windows")]
pub fn send_key_windows(enigo: &mut Enigo, key: Key) -> AppResult<()> {
    enigo
        .key(key, Direction::Click)
        .map_err(|e| AppError::Hotkey(format!("发送按键失败: {e}")))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn send_key_windows(_enigo: &mut Enigo, _key: Key) -> AppResult<()> {
    Err(AppError::Hotkey("此功能仅支持 Windows".into()))
}

/// Send a key press on macOS (must run on main thread)
#[cfg(target_os = "macos")]
pub fn send_key_macos(app: &AppHandle, key: Key) -> AppResult<()> {
    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    let key_to_send = key;

    app.run_on_main_thread(move || {
        let result = Enigo::new(&Settings::default())
            .map_err(|e| format!("创建 Enigo 实例失败: {e}"))
            .and_then(|mut enigo| {
                enigo
                    .key(key_to_send, Direction::Click)
                    .map_err(|e| format!("发送按键失败: {e}"))
            });
        let _ = tx.send(result);
    })
    .map_err(|e| AppError::Hotkey(format!("调度到主线程失败: {e}")))?;

    rx.recv()
        .map_err(|_| AppError::Hotkey("等待主线程执行结果超时".into()))?
        .map_err(AppError::Hotkey)
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn send_key_macos(_app: &tauri::AppHandle, _key: Key) -> AppResult<()> {
    Err(AppError::Hotkey("此功能仅支持 macOS".into()))
}

/// Sleep with interrupt capability
#[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
pub fn sleep_with_interrupt(flag: &Arc<AtomicBool>, total_ms: u64) {
    let mut remaining = if total_ms == 0 { 1 } else { total_ms };
    while remaining > 0 && !flag.load(Ordering::SeqCst) {
        let step = remaining.min(50);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}
