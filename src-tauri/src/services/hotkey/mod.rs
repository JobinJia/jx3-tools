//! Hotkey service for automated key pressing
//!
//! This module provides functionality for:
//! - Registering global hotkeys for start/stop
//! - Running automated key sequences
//! - Windows SendInput for key simulation

mod config;
#[cfg(target_os = "windows")]
mod keys;
mod shortcuts;
mod types;
#[cfg(target_os = "windows")]
pub mod window;

pub use config::CONFIG_FILE_NAME;
pub use types::{HotkeyConfig, HotkeyStatus};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use config::{ensure_app_config_dir, load_config, save_config, validate_config};
use types::HotkeyInner;

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use config::validate_runtime_config;
#[cfg(target_os = "windows")]
use keys::{parse_to_virtual_key, simulate_key_click, sleep_with_interrupt};
#[cfg(target_os = "windows")]
use types::Runner;

/// Event name for hotkey status updates
pub const HOTKEY_STATUS_EVENT: &str = "hotkey://status";

/// Service for managing hotkey automation
pub struct HotkeyService {
    config_path: PathBuf,
    inner: Mutex<HotkeyInner>,
}

impl HotkeyService {
    /// Create a new HotkeyService
    pub fn new() -> AppResult<Self> {
        let config_dir = ensure_app_config_dir()?;
        let config_path = config_dir.join(CONFIG_FILE_NAME);
        Ok(Self {
            config_path,
            inner: Mutex::new(HotkeyInner::default()),
        })
    }

    /// Initialize the service with saved config
    pub fn initialize(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        let config = load_config(&self.config_path).unwrap_or_default();
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
            guard.config = config;
            guard.status.last_error = None;
        }

        // 注册全局热键
        match shortcuts::register_shortcuts(self, app) {
            Ok(_) => self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            }),
            Err(err) => {
                log::warn!("注册热键失败: {}", err);
                self.update_status(app, |status| {
                    status.registered = false;
                    status.last_error = Some(err.to_string());
                });
            }
        }

        Ok(())
    }

    /// Get the current config
    pub fn get_config(&self) -> HotkeyConfig {
        self.inner
            .lock()
            .map(|inner| inner.config.clone())
            .unwrap_or_default()
    }

    /// Get the current status
    pub fn get_status(&self) -> HotkeyStatus {
        self.inner
            .lock()
            .map(|inner| inner.status.clone())
            .unwrap_or_default()
    }

    /// Save a new config and re-register shortcuts
    pub fn save_config(
        self: &Arc<Self>,
        app: &AppHandle,
        config: HotkeyConfig,
    ) -> AppResult<HotkeyConfig> {
        validate_config(&config)?;

        // Stop any running task first (take runner out before joining)
        let runner = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
            guard.runner.take()
        };

        // Join outside the lock to avoid deadlock
        if let Some(mut runner) = runner {
            runner.request_stop();
            runner.join();
        }

        // Now update config with lock
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
            guard.config = config.clone();
            guard.status.running = false;
            guard.status.last_error = None;
        }

        save_config(&self.config_path, &config)?;

        // 重新注册热键监听
        shortcuts::register_shortcuts(self, app)?;
        self.update_status(app, |status| {
            status.registered = true;
            status.last_error = None;
        });
        self.emit_status(app);
        Ok(config)
    }

    /// Stop the running automation task
    pub fn stop_runner(self: &Arc<Self>, app: &AppHandle) {
        // Take runner out while holding lock, then release lock before join
        // to avoid deadlock (finish_running also needs the lock)
        let runner = {
            let mut guard = match self.inner.lock() {
                Ok(lock) => lock,
                Err(err) => {
                    log::error!("停止热键任务时加锁失败: {}", err);
                    return;
                }
            };
            guard.runner.take()
        };

        // Join outside the lock to avoid deadlock
        if let Some(mut runner) = runner {
            runner.request_stop();
            runner.join();
        }

        // Update status after thread has stopped
        if let Ok(mut guard) = self.inner.lock() {
            guard.status.running = false;
        }
        self.emit_status(app);
    }

    /// Start the automation runner
    #[cfg(target_os = "windows")]
    pub fn start_runner(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        let (config, vk) = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
            if guard.status.running {
                return Ok(());
            }

            validate_runtime_config(&guard.config)?;
            let vk = parse_to_virtual_key(&guard.config.trigger_key)?;
            guard.status.running = true;
            guard.status.last_error = None;
            (guard.config.clone(), vk)
        };

        // 窗口模式额外验证
        let (key_mode, target_hwnd) = {
            let mode = config.key_mode.clone();
            let hwnd = if mode == types::KeyMode::Window {
                match &config.target_window {
                    Some(tw) => {
                        if !window::is_window_valid(tw.hwnd) {
                            return Err(AppError::Hotkey("目标窗口已关闭，请重新选择".into()));
                        }
                        Some(tw.hwnd)
                    }
                    None => return Err(AppError::Hotkey("窗口模式需要选择目标窗口".into())),
                }
            } else {
                None
            };
            (mode, hwnd)
        };

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_flag);
        let service = Arc::clone(self);
        let app_handle = app.clone();

        let handle = thread::spawn(move || {
            run_key_loop(
                &stop_clone,
                vk,
                config.interval_ms,
                key_mode,
                target_hwnd,
            );
            service.finish_running(&app_handle);
        });

        let mut guard = self
            .inner
            .lock()
            .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
        guard.runner = Some(Runner::new(stop_flag, handle));
        drop(guard);
        self.emit_status(app);
        Ok(())
    }

    /// Start the automation runner (non-Windows - not supported)
    #[cfg(not(target_os = "windows"))]
    pub fn start_runner(self: &Arc<Self>, _app: &AppHandle) -> AppResult<()> {
        Err(AppError::Hotkey("按键模拟仅支持 Windows 平台".into()))
    }

    /// Mark runner as finished
    #[cfg(target_os = "windows")]
    fn finish_running(&self, app: &AppHandle) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.status.running = false;
            guard.runner = None;
        }
        self.emit_status(app);
    }

    /// Emit current status to frontend
    fn emit_status(&self, app: &AppHandle) {
        if let Ok(status) = self.inner.lock().map(|inner| inner.status.clone()) {
            if let Err(err) = app.emit(HOTKEY_STATUS_EVENT, status) {
                log::warn!("广播热键状态失败: {}", err);
            }
        }
    }

    /// Update status and emit to frontend
    pub fn update_status<F>(&self, app: &AppHandle, mut updater: F)
    where
        F: FnMut(&mut HotkeyStatus),
    {
        if let Ok(mut guard) = self.inner.lock() {
            updater(&mut guard.status);
        }
        self.emit_status(app);
    }
}

/// Run the key sending loop (Windows version with dual mode support)
#[cfg(target_os = "windows")]
fn run_key_loop(
    stop_flag: &Arc<AtomicBool>,
    vk: u16,
    interval_ms: u64,
    key_mode: types::KeyMode,
    target_hwnd: Option<u64>,
) {
    match key_mode {
        types::KeyMode::Global => {
            // 全局模式：使用 SendInput API
            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = simulate_key_click(vk) {
                    log::error!("热键触发失败: {}", err);
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
        types::KeyMode::Window => {
            // 窗口模式：使用 Windows PostMessage API
            let hwnd = match target_hwnd {
                Some(h) => h,
                None => {
                    log::error!("窗口模式未指定目标窗口");
                    return;
                }
            };

            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = window::send_key_to_window(hwnd, vk) {
                    log::error!("发送窗口按键失败: {}", err);
                    // 窗口可能已关闭，停止任务
                    break;
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
    }
}
