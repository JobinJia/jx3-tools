//! Hotkey service
//!
//! Global start/stop hotkeys are registered through tauri-plugin-global-shortcut
//! (RegisterHotKey under the hood) — no kernel driver required. The previous
//! Interception-based listener dynamically linked interception.dll, which
//! crashed the whole app at load time on machines without the driver.
//! The runner thread presses the trigger key via SendInput scancodes (global
//! mode) or PostMessage (window mode).

mod config;
pub mod keymap;
#[cfg(target_os = "windows")]
mod keys;
mod types;
#[cfg(target_os = "windows")]
pub mod window;

pub use config::CONFIG_FILE_NAME;
pub use types::{HotkeyConfig, HotkeyStatus};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::error::{AppError, AppResult};
use config::{ensure_app_config_dir, load_config, save_config, validate_config};
use keymap::parse_shortcut;
use types::HotkeyInner;

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use config::validate_runtime_config;
#[cfg(target_os = "windows")]
use keys::{simulate_key_press, sleep_with_interrupt};
#[cfg(target_os = "windows")]
use types::Runner;

/// Event name for hotkey status updates
pub const HOTKEY_STATUS_EVENT: &str = "hotkey://status";

/// Service for managing hotkey automation
pub struct HotkeyService {
    config_path: PathBuf,
    inner: Mutex<HotkeyInner>,
    /// Shortcuts currently registered with the global-shortcut plugin
    registered_shortcuts: Mutex<Vec<tauri_plugin_global_shortcut::Shortcut>>,
}

impl HotkeyService {
    /// Create a new HotkeyService
    pub fn new() -> AppResult<Self> {
        let config_dir = ensure_app_config_dir()?;
        let config_path = config_dir.join(CONFIG_FILE_NAME);
        Ok(Self {
            config_path,
            inner: Mutex::new(HotkeyInner::default()),
            registered_shortcuts: Mutex::new(Vec::new()),
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

        match self.register_listener(app) {
            Ok(()) => self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            }),
            Err(err) => {
                log::warn!("注册热键失败: {err}");
                self.update_status(app, |status| {
                    status.registered = false;
                    status.last_error = Some(err.to_string());
                });
            }
        }

        Ok(())
    }

    /// Register start/stop hotkeys with the global-shortcut plugin,
    /// replacing any previously registered ones
    fn register_listener(self: &Arc<Self>, app: &AppHandle) -> AppResult<()> {
        let config = self.get_config();

        // 注销旧热键
        {
            let mut guard = self
                .registered_shortcuts
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键注册表锁定失败: {e}")))?;
            for shortcut in guard.drain(..) {
                if let Err(err) = app.global_shortcut().unregister(shortcut) {
                    log::warn!("注销旧热键失败: {err}");
                }
            }
        }

        // 跳过空热键
        if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
            return Ok(());
        }

        let start = parse_shortcut(&config.start_hotkey)?;
        let stop = parse_shortcut(&config.stop_hotkey)?;

        // 事件回调跑在主线程，任务启停派发到新线程，避免阻塞事件循环
        let service = Arc::clone(self);
        app.global_shortcut()
            .on_shortcut(start, move |app, _shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                let service = Arc::clone(&service);
                let app = app.clone();
                thread::spawn(move || {
                    if let Err(err) = service.start_runner(&app) {
                        log::error!("启动热键任务失败: {err}");
                        service.update_status(&app, |status| {
                            status.last_error = Some(err.to_string());
                        });
                    }
                });
            })
            .map_err(|e| AppError::Hotkey(format!("注册开始热键失败: {e}")))?;

        let service = Arc::clone(self);
        if let Err(e) = app
            .global_shortcut()
            .on_shortcut(stop, move |app, _shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                let service = Arc::clone(&service);
                let app = app.clone();
                thread::spawn(move || service.stop_runner(&app));
            })
        {
            // 回滚已注册的开始热键，避免半注册状态
            let _ = app.global_shortcut().unregister(start);
            return Err(AppError::Hotkey(format!("注册结束热键失败: {e}")));
        }

        {
            let mut guard = self
                .registered_shortcuts
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键注册表锁定失败: {e}")))?;
            guard.push(start);
            guard.push(stop);
        }

        log::info!(
            "全局热键已注册: 开始={}, 停止={}",
            config.start_hotkey,
            config.stop_hotkey
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

    /// Save a new config and re-register hotkeys
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

        match self.register_listener(app) {
            Ok(()) => self.update_status(app, |status| {
                status.registered = true;
                status.last_error = None;
            }),
            Err(err) => {
                self.update_status(app, |status| {
                    status.registered = false;
                    status.last_error = Some(err.to_string());
                });
                return Err(err);
            }
        }

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

        let (config, trigger_key) = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Hotkey(format!("热键状态锁定失败: {e}")))?;

            // Double-check after re-acquiring lock
            if guard.status.running && guard.runner.is_some() {
                return Ok(());
            }

            validate_runtime_config(&guard.config)?;
            let key = keymap::resolve_key(&guard.config.trigger_key)?;
            guard.status.running = true;
            guard.status.last_error = None;
            (guard.config.clone(), key)
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
                trigger_key,
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
    trigger_key: keymap::KeyDef,
    interval_ms: u64,
    key_mode: types::KeyMode,
    target_hwnd: Option<u64>,
) {
    match key_mode {
        types::KeyMode::Global => {
            // 全局模式：SendInput 扫描码模拟
            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = simulate_key_press(trigger_key) {
                    log::error!("热键触发失败: {}", err);
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
        types::KeyMode::Window => {
            // 窗口模式：PostMessage 发送虚拟键码
            let hwnd = match target_hwnd {
                Some(h) => h,
                None => {
                    log::error!("窗口模式未指定目标窗口");
                    return;
                }
            };

            while !stop_flag.load(Ordering::SeqCst) {
                if let Err(err) = window::send_key_to_window(hwnd, trigger_key.vk) {
                    log::error!("发送窗口按键失败: {}", err);
                    break;
                }
                sleep_with_interrupt(stop_flag, interval_ms);
            }
        }
    }
}
