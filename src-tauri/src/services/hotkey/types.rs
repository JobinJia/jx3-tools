use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use serde::{Deserialize, Serialize};

/// 按键发送模式
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum KeyMode {
    #[default]
    Global, // 全局模式，使用 enigo 发送
    Window, // 窗口模式，使用 Windows API 发送到指定窗口
}

/// 目标窗口信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TargetWindow {
    pub hwnd: u64,           // 窗口句柄
    pub title: String,       // 窗口标题
    pub class_name: String,  // 窗口类名
    pub process_name: String, // 进程名
}

/// Configuration for hotkey automation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub trigger_key: String,
    pub interval_ms: u64,
    pub start_hotkey: String,
    pub stop_hotkey: String,
    #[serde(default)]
    pub key_mode: KeyMode,
    #[serde(default)]
    pub target_window: Option<TargetWindow>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            trigger_key: String::new(),
            interval_ms: 1000,
            start_hotkey: "F11".to_string(),
            stop_hotkey: "F12".to_string(),
            key_mode: KeyMode::default(),
            target_window: None,
        }
    }
}

/// Status of the hotkey service
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub running: bool,
    pub registered: bool,
    pub last_error: Option<String>,
}

/// Internal state of the hotkey service
#[derive(Debug)]
pub struct HotkeyInner {
    pub config: HotkeyConfig,
    pub status: HotkeyStatus,
    pub runner: Option<Runner>,
}

impl Default for HotkeyInner {
    fn default() -> Self {
        Self {
            config: HotkeyConfig::default(),
            status: HotkeyStatus::default(),
            runner: None,
        }
    }
}

/// Thread runner for key automation
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug)]
pub struct Runner {
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl Runner {
    pub fn new(stop_flag: Arc<AtomicBool>, handle: thread::JoinHandle<()>) -> Self {
        Self {
            stop_flag,
            handle: Some(handle),
        }
    }

    pub fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.request_stop();
        self.join();
    }
}
