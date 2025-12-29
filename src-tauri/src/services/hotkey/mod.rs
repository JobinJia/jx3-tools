//! Hotkey service using Interception driver
//!
//! This module provides:
//! - Global hotkey detection via Interception driver
//! - Automated key sequences with configurable intervals
//! - Window-specific key sending support

mod config;
#[cfg(target_os = "windows")]
mod keys;
#[cfg(target_os = "windows")]
mod listener;
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
use keys::{simulate_key_press, sleep_with_interrupt};
#[cfg(target_os = "windows")]
use listener::{label_to_scancode, HotkeyEvent, HotkeyListener, ListenerConfig};
#[cfg(target_os = "windows")]
use types::Runner;

/// Event name for hotkey status updates
pub const HOTKEY_STATUS_EVENT: &str = "hotkey://status";

/// Service for managing hotkey automation
pub struct HotkeyService {
    config_path: PathBuf,
    inner: Mutex<HotkeyInner>,
    #[cfg(target_os = "windows")]
    listener: Mutex<Option<HotkeyListener>>,
}

impl HotkeyService {
    /// Create a new HotkeyService
    pub fn new() -> AppResult<Self> {
        let config_dir = ensure_app_config_dir()?;
        let config_path = config_dir.join(CONFIG_FILE_NAME);
        Ok(Self {
            config_path,
            inner: Mutex::new(HotkeyInner::default()),
            #[cfg(target_os = "windows")]
            listener: Mutex::new(None),
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

        // 注册 Interception 监听器
        #[cfg(target_os = "windows")]
        match self.register_listener(app) {
            Ok(_) => self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            }),
            Err(err) => {
                log::warn!("注册热键监听器失败: {}", err);
                self.update_status(app, |status| {
                    status.registered = false;
                    status.last_error = Some(err.to_string());
                });
            }
        }

        #[cfg(not(target_os = "windows"))]
        self.update_status(app, |status| {
            status.registered = false;
            status.last_error = Some("热键功能仅支持 Windows".into());
        });

        Ok(())
    }

    /// Register the Interception-based hotkey listener
    #[cfg(target_os = "windows")]
    fn register_listener(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        let config = self.get_config();

        // 跳过空热键
        if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
            return Ok(());
        }

        // 停止现有监听器
        {
            let mut guard = self.listener.lock()
                .map_err(|e| AppError::Hotkey(format!("监听器锁定失败: {e}")))?;
            if let Some(mut listener) = guard.take() {
                listener.stop();
            }
        }

        // 解析热键为扫描码
        let start_scancode = label_to_scancode(&config.start_hotkey)?;
        let stop_scancode = label_to_scancode(&config.stop_hotkey)?;

        let listener_config = ListenerConfig {
            start_scancode,
            stop_scancode,
        };

        let service = Arc::clone(self);
        let app_handle = app.clone();

        // Use a separate thread for handling hotkey events to avoid blocking
        // the listener thread and potential deadlocks
        let listener = HotkeyListener::new(listener_config, move |event| {
            let service_clone = Arc::clone(&service);
            let app_clone = app_handle.clone();

            // Spawn a new thread to handle the event asynchronously
            // This prevents blocking the listener thread which could cause deadlocks
            thread::spawn(move || {
                match event {
                    HotkeyEvent::Start => {
                        if let Err(err) = service_clone.start_runner(&app_clone) {
                            log::error!("启动热键任务失败: {}", err);
                            service_clone.update_status(&app_clone, |status| {
                                status.last_error = Some(err.to_string());
                            });
                        }
                    }
                    HotkeyEvent::Stop => {
                        service_clone.stop_runner(&app_clone);
                    }
                }
            });
        })?;

        let mut guard = self.listener.lock()
            .map_err(|e| AppError::Hotkey(format!("监听器锁定失败: {e}")))?;
        *guard = Some(listener);

        log::info!(
            "Interception 热键监听器已注册: 开始={} (0x{:02X}), 停止={} (0x{:02X})",
            config.start_hotkey, start_scancode,
            config.stop_hotkey, stop_scancode
        );

        Ok(())
    }

    /// Get the current config
    pub fn get_config(&self) -> HotkeyConfig {
        match self.inner.lock() {
            Ok(inner) => inner.config.clone(),
            Err(poisoned) => {
                log::warn!("热键配置锁已损坏，使用损坏数据: {}", poisoned);
                poisoned.into_inner().config.clone()
            }
        }
    }

    /// Get the current status
    pub fn get_status(&self) -> HotkeyStatus {
        match self.inner.lock() {
            Ok(inner) => inner.status.clone(),
            Err(poisoned) => {
                log::warn!("热键状态锁已损坏，使用损坏数据: {}", poisoned);
                poisoned.into_inner().status.clone()
            }
        }
    }

    /// Save a new config and re-register listener
    pub fn save_config(
        self: &Arc<Self>,
        app: &AppHandle,
        config: HotkeyConfig,
    ) -> AppResult<HotkeyConfig> {
        validate_config(&config)?;

        // Stop any running task first
        let runner = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
            guard.runner.take()
        };

        if let Some(mut runner) = runner {
            runner.request_stop();
            runner.join();
        }

        // Update config
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

        // 重新注册监听器
        #[cfg(target_os = "windows")]
        {
            self.register_listener(app)?;
            self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            });
        }

        self.emit_status(app);
        Ok(config)
    }

    /// Stop the running automation task
    pub fn stop_runner(self: &Arc<Self>, app: &AppHandle) {
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

        if let Some(mut runner) = runner {
            runner.request_stop();
            runner.join();
        }

        if let Ok(mut guard) = self.inner.lock() {
            guard.status.running = false;
        }
        self.emit_status(app);
    }

    /// Start the automation runner
    #[cfg(target_os = "windows")]
    pub fn start_runner(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        // First, stop any existing runner to prevent multiple runners
        let existing_runner = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;

            // Already running, skip
            if guard.status.running && guard.runner.is_some() {
                return Ok(());
            }

            // Take any existing runner for cleanup
            guard.runner.take()
        };

        // Stop existing runner outside the lock to prevent blocking
        if let Some(mut runner) = existing_runner {
            runner.request_stop();
            runner.join();
        }

        let (config, trigger_scancode) = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;

            // Double-check after re-acquiring lock
            if guard.status.running && guard.runner.is_some() {
                return Ok(());
            }

            validate_runtime_config(&guard.config)?;
            let scancode = label_to_scancode(&guard.config.trigger_key)?;
            guard.status.running = true;
            guard.status.last_error = None;
            (guard.config.clone(), scancode)
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
                trigger_scancode,
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

    /// Start the automation runner (non-Windows)
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
        let status = match self.inner.lock() {
            Ok(inner) => inner.status.clone(),
            Err(poisoned) => {
                log::warn!("热键状态锁已损坏，使用损坏数据广播: {}", poisoned);
                poisoned.into_inner().status.clone()
            }
        };
        if let Err(err) = app.emit(HOTKEY_STATUS_EVENT, status) {
            log::warn!("广播热键状态失败: {}", err);
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

/// Run the key sending loop
#[cfg(target_os = "windows")]
fn run_key_loop(
    stop_flag: &Arc<AtomicBool>,
    trigger_scancode: u16,
    interval_ms: u64,
    key_mode: types::KeyMode,
    target_hwnd: Option<u64>,
) {
    match key_mode {
        types::KeyMode::Global => {
            // 全局模式：使用 Interception 或 SendInput
            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = simulate_key_press(trigger_scancode) {
                    log::error!("热键触发失败: {}", err);
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
        types::KeyMode::Window => {
            // 窗口模式：使用 PostMessage
            let hwnd = match target_hwnd {
                Some(h) => h,
                None => {
                    log::error!("窗口模式未指定目标窗口");
                    return;
                }
            };

            // 将扫描码转换为虚拟键码用于 PostMessage
            let vk = scancode_to_vk(trigger_scancode);

            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = window::send_key_to_window(hwnd, vk) {
                    log::error!("发送窗口按键失败: {}", err);
                    break;
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
    }
}

/// Convert scancode to virtual key code (for window mode)
#[cfg(target_os = "windows")]
fn scancode_to_vk(scancode: u16) -> u16 {
    unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(
            scancode as u32,
            windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VSC_TO_VK,
        ) as u16
    }
}
