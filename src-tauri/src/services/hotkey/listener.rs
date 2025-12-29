//! Interception-based keyboard listener
//!
//! Uses the interception driver to intercept keyboard input and detect hotkeys.
//! Non-hotkey inputs are forwarded back to the system.

#![cfg(target_os = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use interception::{Interception, KeyState, Stroke};

use crate::error::{AppError, AppResult};

/// Hotkey listener using interception driver
pub struct HotkeyListener {
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// Callback type for hotkey events
pub type HotkeyCallback = Box<dyn Fn(HotkeyEvent) + Send + 'static>;

/// Hotkey event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Start,
    Stop,
}

/// Listener configuration
#[derive(Clone)]
pub struct ListenerConfig {
    pub start_scancode: u16,
    pub stop_scancode: u16,
}

/// Maximum time to wait for listener thread to join (in milliseconds)
const LISTENER_JOIN_TIMEOUT_MS: u64 = 500;

impl HotkeyListener {
    /// Create and start a new hotkey listener
    pub fn new<F>(config: ListenerConfig, callback: F) -> AppResult<Self>
    where
        F: Fn(HotkeyEvent) + Send + 'static,
    {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            if let Err(e) = run_listener_loop(&stop_clone, config, callback) {
                log::error!("Hotkey listener error: {}", e);
            }
        });

        Ok(Self {
            stop_flag,
            handle: Some(handle),
        })
    }

    /// Stop the listener with timeout to prevent freezing
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            // Wait for thread to finish with timeout
            let start = std::time::Instant::now();
            let timeout = Duration::from_millis(LISTENER_JOIN_TIMEOUT_MS);

            while !handle.is_finished() {
                if start.elapsed() >= timeout {
                    log::warn!(
                        "Listener 线程在 {}ms 内未能退出，放弃等待",
                        LISTENER_JOIN_TIMEOUT_MS
                    );
                    // Detach the thread - it will clean up when it finishes
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }

            // Thread finished, safe to join
            if let Err(e) = handle.join() {
                log::error!("Listener 线程 join 失败: {:?}", e);
            }
        }
    }
}

impl Drop for HotkeyListener {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Run the keyboard interception loop
fn run_listener_loop<F>(
    stop_flag: &Arc<AtomicBool>,
    config: ListenerConfig,
    callback: F,
) -> AppResult<()>
where
    F: Fn(HotkeyEvent) + Send + 'static,
{
    let ctx = Interception::new().ok_or_else(|| {
        AppError::Hotkey("无法创建 Interception 上下文，请确保已安装 Interception 驱动".into())
    })?;

    // Set filter for all keyboards
    ctx.set_filter(
        interception::is_keyboard,
        interception::Filter::KeyFilter(interception::KeyFilter::all()),
    );

    log::info!(
        "Hotkey listener started: start={:#04x}, stop={:#04x}",
        config.start_scancode,
        config.stop_scancode
    );

    let mut strokes = [Stroke::Keyboard {
        code: interception::ScanCode::Esc,
        state: KeyState::empty(),
        information: 0,
    }; 1];

    while !stop_flag.load(Ordering::SeqCst) {
        // Wait for input with timeout (allows periodic stop check)
        let device = ctx.wait_with_timeout(Duration::from_millis(100));

        // Device 0 means timeout or no input
        if device == 0 {
            continue;
        }

        // Receive the stroke
        let received = ctx.receive(device, &mut strokes);
        if received <= 0 {
            continue;
        }

        // Process keyboard strokes
        if let Stroke::Keyboard { code, state, .. } = strokes[0] {
            let scancode: u16 = code as u16;
            let is_keydown = !state.contains(KeyState::UP);

            // Check for hotkeys (only on key down)
            if is_keydown {
                if scancode == config.start_scancode {
                    log::info!("Start hotkey detected: scancode={:#04x}", scancode);
                    callback(HotkeyEvent::Start);
                    // Don't forward the hotkey to the system
                    continue;
                } else if scancode == config.stop_scancode {
                    log::info!("Stop hotkey detected: scancode={:#04x}", scancode);
                    callback(HotkeyEvent::Stop);
                    // Don't forward the hotkey to the system
                    continue;
                }
            } else {
                // Also block key-up events for hotkeys
                if scancode == config.start_scancode || scancode == config.stop_scancode {
                    continue;
                }
            }
        }

        // Forward non-hotkey strokes back to the system
        ctx.send(device, &strokes[..received as usize]);
    }

    log::info!("Hotkey listener stopped");
    Ok(())
}

/// Convert a key label to scancode
pub fn label_to_scancode(label: &str) -> AppResult<u16> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("按键不能为空".into()));
    }

    let upper = trimmed.to_uppercase();

    // Single character keys A-Z, 0-9
    if upper.len() == 1 {
        if let Some(ch) = upper.chars().next() {
            if ch.is_ascii_uppercase() {
                // A=0x1E, B=0x30, C=0x2E, etc. (scan codes are not sequential)
                return Ok(char_to_scancode(ch));
            }
            if ch.is_ascii_digit() {
                // 0=0x0B, 1=0x02, 2=0x03, etc.
                return Ok(digit_to_scancode(ch));
            }
        }
    }

    // Special keys mapping to scan codes
    let scancode = match upper.as_str() {
        "ESC" | "ESCAPE" => 0x01,
        "F1" => 0x3B,
        "F2" => 0x3C,
        "F3" => 0x3D,
        "F4" => 0x3E,
        "F5" => 0x3F,
        "F6" => 0x40,
        "F7" => 0x41,
        "F8" => 0x42,
        "F9" => 0x43,
        "F10" => 0x44,
        "F11" => 0x57,
        "F12" => 0x58,
        "BACKSPACE" => 0x0E,
        "TAB" => 0x0F,
        "ENTER" | "RETURN" => 0x1C,
        "CTRL" | "CONTROL" | "LCTRL" => 0x1D,
        "SHIFT" | "LSHIFT" => 0x2A,
        "RSHIFT" => 0x36,
        "ALT" | "LALT" => 0x38,
        "SPACE" => 0x39,
        "CAPSLOCK" | "CAPS" => 0x3A,
        "NUMLOCK" => 0x45,
        "SCROLLLOCK" => 0x46,
        "HOME" => 0x47,
        "UP" | "ARROWUP" => 0x48,
        "PAGEUP" => 0x49,
        "LEFT" | "ARROWLEFT" => 0x4B,
        "RIGHT" | "ARROWRIGHT" => 0x4D,
        "END" => 0x4F,
        "DOWN" | "ARROWDOWN" => 0x50,
        "PAGEDOWN" => 0x51,
        "INSERT" => 0x52,
        "DELETE" | "DEL" => 0x53,
        // Numpad
        "NUM0" | "NUMPAD0" => 0x52,
        "NUM1" | "NUMPAD1" => 0x4F,
        "NUM2" | "NUMPAD2" => 0x50,
        "NUM3" | "NUMPAD3" => 0x51,
        "NUM4" | "NUMPAD4" => 0x4B,
        "NUM5" | "NUMPAD5" => 0x4C,
        "NUM6" | "NUMPAD6" => 0x4D,
        "NUM7" | "NUMPAD7" => 0x47,
        "NUM8" | "NUMPAD8" => 0x48,
        "NUM9" | "NUMPAD9" => 0x49,
        "NUMMUL" | "NUMSTAR" | "NUMMULTIPLY" => 0x37,
        "NUMSUB" | "NUMMINUS" => 0x4A,
        "NUMADD" | "NUMPLUS" => 0x4E,
        "NUMDOT" | "NUMDECIMAL" => 0x53,
        "NUMDIV" | "NUMSLASH" | "NUMDIVIDE" => 0x35,
        // OEM keys
        ";" | "SEMICOLON" | "OEM1" => 0x27,
        "=" | "EQUALS" | "OEMPLUS" => 0x0D,
        "," | "COMMA" | "OEMCOMMA" => 0x33,
        "-" | "MINUS" | "OEMMINUS" => 0x0C,
        "." | "PERIOD" | "OEMPERIOD" => 0x34,
        "/" | "SLASH" | "OEM2" => 0x35,
        "`" | "GRAVE" | "BACKQUOTE" | "OEM3" => 0x29,
        "[" | "BRACKETLEFT" | "OEM4" => 0x1A,
        "\\" | "BACKSLASH" | "OEM5" => 0x2B,
        "]" | "BRACKETRIGHT" | "OEM6" => 0x1B,
        "'" | "QUOTE" | "OEM7" => 0x28,
        _ => return Err(AppError::Hotkey(format!("不支持的按键: {}", trimmed))),
    };

    Ok(scancode)
}

/// Convert A-Z character to scan code
fn char_to_scancode(ch: char) -> u16 {
    match ch {
        'A' => 0x1E,
        'B' => 0x30,
        'C' => 0x2E,
        'D' => 0x20,
        'E' => 0x12,
        'F' => 0x21,
        'G' => 0x22,
        'H' => 0x23,
        'I' => 0x17,
        'J' => 0x24,
        'K' => 0x25,
        'L' => 0x26,
        'M' => 0x32,
        'N' => 0x31,
        'O' => 0x18,
        'P' => 0x19,
        'Q' => 0x10,
        'R' => 0x13,
        'S' => 0x1F,
        'T' => 0x14,
        'U' => 0x16,
        'V' => 0x2F,
        'W' => 0x11,
        'X' => 0x2D,
        'Y' => 0x15,
        'Z' => 0x2C,
        _ => 0,
    }
}

/// Convert 0-9 digit to scan code
fn digit_to_scancode(ch: char) -> u16 {
    match ch {
        '0' => 0x0B,
        '1' => 0x02,
        '2' => 0x03,
        '3' => 0x04,
        '4' => 0x05,
        '5' => 0x06,
        '6' => 0x07,
        '7' => 0x08,
        '8' => 0x09,
        '9' => 0x0A,
        _ => 0,
    }
}
