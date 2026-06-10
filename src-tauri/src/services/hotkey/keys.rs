//! 按键模拟模块 - 使用 SendInput 扫描码方式发送按键
//!
//! 扫描码级模拟（KEYEVENTF_SCANCODE）对游戏（DirectInput）兼容性最好；
//! 方向键等 0xE0 前缀键需要额外的 KEYEVENTF_EXTENDEDKEY 标志。

#![cfg(target_os = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};

use super::keymap::KeyDef;
use crate::error::{AppError, AppResult};

fn keyboard_input(scancode: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scancode,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// 模拟按键点击 (按下 + 释放)
pub fn simulate_key_press(key: KeyDef) -> AppResult<()> {
    let mut down_flags = KEYEVENTF_SCANCODE;
    let mut up_flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
    if key.extended {
        down_flags |= KEYEVENTF_EXTENDEDKEY;
        up_flags |= KEYEVENTF_EXTENDEDKEY;
    }

    let key_down = keyboard_input(key.scancode, down_flags);
    let key_up = keyboard_input(key.scancode, up_flags);

    unsafe {
        if SendInput(&[key_down], std::mem::size_of::<INPUT>() as i32) == 0 {
            return Err(AppError::Hotkey("SendInput 发送按键按下失败".into()));
        }

        thread::sleep(Duration::from_millis(10));

        if SendInput(&[key_up], std::mem::size_of::<INPUT>() as i32) == 0 {
            return Err(AppError::Hotkey("SendInput 发送按键释放失败".into()));
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
