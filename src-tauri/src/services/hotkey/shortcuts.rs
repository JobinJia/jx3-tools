//! Global hotkey listener using rdev
//!
//! This module provides global keyboard event listening functionality
//! to detect start/stop hotkey presses.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

use rdev::{listen, Event, EventType, Key};
use tauri::AppHandle;

use super::HotkeyService;
use crate::error::{AppError, AppResult};

/// 全局键盘监听器
pub struct GlobalListener {
    /// 监听线程句柄
    handle: Option<JoinHandle<()>>,
    /// 停止标志
    stop_flag: Arc<AtomicBool>,
}

impl GlobalListener {
    /// 创建新的监听器（未启动）
    pub fn new() -> Self {
        Self {
            handle: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动监听线程
    pub fn start(
        &mut self,
        start_key: Key,
        stop_key: Key,
        service: Arc<HotkeyService>,
        app: AppHandle,
    ) {
        // 先停止旧的监听器
        self.stop();

        let stop_flag = Arc::clone(&self.stop_flag);
        stop_flag.store(false, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            log::info!("全局热键监听线程启动");

            // 用于防止重复触发的状态跟踪
            let key_state = Arc::new(Mutex::new(KeyState::new()));

            let callback = move |event: Event| {
                // 检查停止标志
                if stop_flag.load(Ordering::SeqCst) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let mut state = match key_state.lock() {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        // 检查是否是开始热键
                        if key == start_key && !state.start_pressed {
                            state.start_pressed = true;
                            log::info!("检测到开始热键");
                            if let Err(err) = service.start_runner(&app) {
                                log::error!("启动热键任务失败: {}", err);
                                service.update_status(&app, |status| {
                                    status.last_error = Some(err.to_string());
                                });
                            }
                        }

                        // 检查是否是停止热键
                        if key == stop_key && !state.stop_pressed {
                            state.stop_pressed = true;
                            log::info!("检测到停止热键");
                            service.stop_runner(&app);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let mut state = match key_state.lock() {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        if key == start_key {
                            state.start_pressed = false;
                        }
                        if key == stop_key {
                            state.stop_pressed = false;
                        }
                    }
                    _ => {}
                }
            };

            if let Err(err) = listen(callback) {
                log::error!("全局热键监听失败: {:?}", err);
            }

            log::info!("全局热键监听线程退出");
        });

        self.handle = Some(handle);
    }

    /// 停止监听
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        // 注意：rdev::listen 是阻塞的，无法从外部强制停止
        // 我们只能设置标志位，让回调函数忽略后续事件
        // 线程会在进程退出时自动终止
        if let Some(handle) = self.handle.take() {
            // 不要 join，因为 listen 是阻塞的
            // 让线程在后台继续运行但忽略事件
            drop(handle);
        }
    }
}

impl Drop for GlobalListener {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 按键状态跟踪，防止重复触发
struct KeyState {
    start_pressed: bool,
    stop_pressed: bool,
}

impl KeyState {
    fn new() -> Self {
        Self {
            start_pressed: false,
            stop_pressed: false,
        }
    }
}

/// 解析热键字符串为 rdev Key
pub fn parse_hotkey(hotkey: &str) -> AppResult<Key> {
    let trimmed = hotkey.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("热键不能为空".into()));
    }

    // 支持的热键格式: F1-F12, 单个字母/数字, 标点符号
    let upper = trimmed.to_uppercase();

    match upper.as_str() {
        "F1" => Ok(Key::F1),
        "F2" => Ok(Key::F2),
        "F3" => Ok(Key::F3),
        "F4" => Ok(Key::F4),
        "F5" => Ok(Key::F5),
        "F6" => Ok(Key::F6),
        "F7" => Ok(Key::F7),
        "F8" => Ok(Key::F8),
        "F9" => Ok(Key::F9),
        "F10" => Ok(Key::F10),
        "F11" => Ok(Key::F11),
        "F12" => Ok(Key::F12),
        _ => {
            // 尝试解析为单个字符
            if trimmed.len() == 1 {
                let ch = trimmed.chars().next().unwrap();
                // 字母 A-Z (大小写都支持)
                if ch.is_ascii_alphabetic() {
                    return match ch.to_ascii_uppercase() {
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
                        _ => Err(AppError::Hotkey(format!("不支持的热键: {}", hotkey))),
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
                        _ => Err(AppError::Hotkey(format!("不支持的热键: {}", hotkey))),
                    };
                }
                // 标点符号
                return match ch {
                    ';' => Ok(Key::SemiColon),
                    ',' => Ok(Key::Comma),
                    '.' => Ok(Key::Dot),
                    '/' => Ok(Key::Slash),
                    '\\' => Ok(Key::BackSlash),
                    '\'' => Ok(Key::Quote),
                    '[' => Ok(Key::LeftBracket),
                    ']' => Ok(Key::RightBracket),
                    '-' => Ok(Key::Minus),
                    '=' => Ok(Key::Equal),
                    '`' => Ok(Key::BackQuote),
                    _ => Err(AppError::Hotkey(format!("不支持的热键: {}", hotkey))),
                };
            }
            Err(AppError::Hotkey(format!("不支持的热键格式: {}", hotkey)))
        }
    }
}

/// 注册全局快捷键（启动监听器）
pub fn register_shortcuts(service: &Arc<HotkeyService>, app: &AppHandle) -> AppResult<()> {
    let config = service.get_config();

    // 跳过空热键
    if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
        return Ok(());
    }

    // 解析热键
    let start_key = parse_hotkey(&config.start_hotkey)?;
    let stop_key = parse_hotkey(&config.stop_hotkey)?;

    // 获取监听器并启动
    service.start_listener(start_key, stop_key, Arc::clone(service), app.clone());

    Ok(())
}
