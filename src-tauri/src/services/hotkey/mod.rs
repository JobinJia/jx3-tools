//! Hotkey service for automated key pressing
//!
//! This module provides functionality for:
//! - Registering global hotkeys for start/stop
//! - Running automated key sequences
//! - Platform-specific key sending (Windows/macOS)

mod config;
mod keys;
mod shortcuts;
mod types;

pub use config::CONFIG_FILE_NAME;
pub use types::{HotkeyConfig, HotkeyStatus};

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

#[cfg(target_os = "windows")]
use enigo::{Enigo, Key, Settings};
#[cfg(not(target_os = "windows"))]
use enigo::Key;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use config::{ensure_app_config_dir, load_config, save_config, validate_config, validate_runtime_config};
use keys::{parse_trigger_key, sleep_with_interrupt};
use types::{HotkeyInner, Runner};

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

        #[cfg(target_os = "windows")]
        {
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
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.update_status(app, |status| {
                status.registered = false;
                status.last_error = Some("热键功能仅支持 Windows".to_string());
            });
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

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            shortcuts::register_shortcuts(self, app)?;
            self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            });
            self.emit_status(app);
            return Ok(config);
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let message = "热键功能仅支持 Windows 或 macOS".to_string();
            self.update_status(app, |status| {
                status.registered = false;
                status.last_error = Some(message.clone());
            });
            return Err(AppError::Hotkey(message));
        }
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
    #[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
    pub fn start_runner(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let _ = app;
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            return Err(AppError::Hotkey("热键功能仅支持 Windows 或 macOS".into()));
        }

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            let (config, key) = {
                let mut guard = self
                    .inner
                    .lock()
                    .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;
                if guard.status.running {
                    return Ok(());
                }

                validate_runtime_config(&guard.config)?;
                let key = parse_trigger_key(&guard.config.trigger_key)?;
                guard.status.running = true;
                guard.status.last_error = None;
                (guard.config.clone(), key)
            };

            let stop_flag = Arc::new(AtomicBool::new(false));
            let stop_clone = Arc::clone(&stop_flag);
            let service = Arc::clone(self);
            let app_handle = app.clone();

            let handle = thread::spawn(move || {
                run_key_loop(&stop_clone, &service, &app_handle, key, config.interval_ms);
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
    }

    /// Mark runner as finished
    #[cfg_attr(not(any(target_os = "windows", target_os = "macos")), allow(dead_code))]
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

/// Run the key sending loop
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn run_key_loop(
    stop_flag: &Arc<AtomicBool>,
    _service: &Arc<HotkeyService>,
    app_handle: &AppHandle,
    key: Key,
    interval_ms: u64,
) {
    #[cfg(target_os = "windows")]
    {
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(err) => {
                log::error!("创建 Enigo 实例失败: {}", err);
                return;
            }
        };

        while !stop_flag.load(Ordering::SeqCst) {
            if let Err(err) = keys::send_key_windows(&mut enigo, key) {
                log::error!("热键触发失败: {}", err);
            }
            sleep_with_interrupt(stop_flag, interval_ms);
        }
    }

    #[cfg(target_os = "macos")]
    {
        while !stop_flag.load(Ordering::SeqCst) {
            if let Err(err) = keys::send_key_macos(app_handle, key) {
                log::error!("热键触发失败: {}", err);
            }
            sleep_with_interrupt(stop_flag, interval_ms);
        }
    }
}
