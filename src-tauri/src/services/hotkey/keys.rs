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
        "SPACE" => Key::Space,
        "ENTER" => Key::Return,
        "TAB" => Key::Tab,
        "ESC" | "ESCAPE" => Key::Escape,
        "UP" | "ARROWUP" => Key::UpArrow,
        "DOWN" | "ARROWDOWN" => Key::DownArrow,
        "LEFT" | "ARROWLEFT" => Key::LeftArrow,
        "RIGHT" | "ARROWRIGHT" => Key::RightArrow,
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
        #[cfg(target_os = "windows")]
        "F21" => Key::F21,
        #[cfg(target_os = "windows")]
        "F22" => Key::F22,
        #[cfg(target_os = "windows")]
        "F23" => Key::F23,
        #[cfg(target_os = "windows")]
        "F24" => Key::F24,
        #[cfg(target_os = "macos")]
        "FUNCTION" => Key::Function,
        "HOME" => Key::Home,
        "END" => Key::End,
        "PAGEUP" => Key::PageUp,
        "PAGEDOWN" => Key::PageDown,
        #[cfg(target_os = "windows")]
        "INSERT" => Key::Insert,
        "DELETE" => Key::Delete,
        "BACKSPACE" => Key::Backspace,
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
