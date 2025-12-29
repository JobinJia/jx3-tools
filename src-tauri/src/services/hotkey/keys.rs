//! 按键模拟模块 - 使用 Interception 驱动发送按键
//!
//! 使用 Interception 驱动级方案，提供更好的游戏兼容性
//! 若驱动未安装，自动回退到 SendInput 扫描码模式

#![cfg(target_os = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
};
use std::thread;
use std::time::Duration;

use interception::{Interception, KeyState, ScanCode, Stroke};

use crate::error::{AppError, AppResult};

/// 全局 Interception 发送上下文（延迟初始化）
static SENDER_CTX: OnceLock<Option<SenderContext>> = OnceLock::new();

/// Interception 发送上下文
struct SenderContext {
    ctx: Interception,
    keyboard_device: interception::Device,
}

// 手动实现 Send 和 Sync
unsafe impl Send for SenderContext {}
unsafe impl Sync for SenderContext {}

/// 获取或初始化发送上下文
fn get_sender_ctx() -> Option<&'static SenderContext> {
    SENDER_CTX
        .get_or_init(|| {
            match init_sender() {
                Ok(ctx) => {
                    log::info!("Interception 发送器初始化成功");
                    Some(ctx)
                }
                Err(e) => {
                    log::warn!("Interception 发送器不可用: {}, 将使用 SendInput 回退", e);
                    None
                }
            }
        })
        .as_ref()
}

/// 初始化 Interception 发送器
fn init_sender() -> AppResult<SenderContext> {
    let ctx = Interception::new().ok_or_else(|| {
        AppError::Hotkey("无法创建 Interception 上下文".into())
    })?;

    // 键盘设备 ID 1
    let keyboard_device: interception::Device = 1;

    Ok(SenderContext {
        ctx,
        keyboard_device,
    })
}

/// 使用 Interception 驱动模拟按键点击
fn send_key_interception(ctx: &SenderContext, scan_code: u16) -> AppResult<()> {
    let code = ScanCode::try_from(scan_code)
        .map_err(|_| AppError::Hotkey(format!("无效的扫描码: {:#04x}", scan_code)))?;

    // 构造按键按下事件
    let key_down = Stroke::Keyboard {
        code,
        state: KeyState::empty(),
        information: 0,
    };

    // 构造按键释放事件
    let key_up = Stroke::Keyboard {
        code,
        state: KeyState::UP,
        information: 0,
    };

    // 发送按键按下
    let sent = ctx.ctx.send(ctx.keyboard_device, &[key_down]);
    if sent == 0 {
        return Err(AppError::Hotkey("Interception 发送按键按下失败".into()));
    }

    // 短暂延迟
    thread::sleep(Duration::from_millis(10));

    // 发送按键释放
    let sent = ctx.ctx.send(ctx.keyboard_device, &[key_up]);
    if sent == 0 {
        return Err(AppError::Hotkey("Interception 发送按键释放失败".into()));
    }

    Ok(())
}

/// 使用 SendInput API 模拟按键点击 (回退方案)
fn send_key_sendinput(scan_code: u16) -> AppResult<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, VIRTUAL_KEY,
    };

    let key_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let key_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan_code,
                dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        let sent = SendInput(&[key_down], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(AppError::Hotkey("SendInput 发送按键按下失败".into()));
        }

        thread::sleep(Duration::from_millis(10));

        let sent = SendInput(&[key_up], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(AppError::Hotkey("SendInput 发送按键释放失败".into()));
        }
    }

    Ok(())
}

/// 模拟按键点击 (按下 + 释放)
/// 优先使用 Interception 驱动，失败时回退到 SendInput
pub fn simulate_key_press(scan_code: u16) -> AppResult<()> {
    if let Some(ctx) = get_sender_ctx() {
        return send_key_interception(ctx, scan_code);
    }
    send_key_sendinput(scan_code)
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

/// 检查 Interception 驱动是否可用
pub fn is_interception_available() -> bool {
    get_sender_ctx().is_some()
}
