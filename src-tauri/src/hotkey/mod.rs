use std::{
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use dirs;
use enigo::{Enigo, Key, KeyboardControllable};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

const CONFIG_FILE: &str = "hotkey_config.json";
const STATUS_EVENT: &str = "hotkey://status";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub trigger_key: String,
    pub interval_ms: u64,
    pub start_hotkey: String,
    pub stop_hotkey: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            trigger_key: String::new(),
            interval_ms: 1000,
            start_hotkey: String::from("F11"),
            stop_hotkey: String::from("F12"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub running: bool,
    pub last_error: Option<String>,
}

struct Runner {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Runner {
    fn new(stop: Arc<AtomicBool>, handle: thread::JoinHandle<()>) -> Self {
        Self {
            stop,
            handle: Some(handle),
        }
    }

    fn request_stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    fn join(&mut self) {
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

struct HotkeyInner {
    config: HotkeyConfig,
    status: HotkeyStatus,
    runner: Option<Runner>,
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

pub struct HotkeyManager {
    inner: Mutex<HotkeyInner>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HotkeyInner::default()),
        }
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&self, app: &AppHandle) -> Result<(), String> {
        let config = self.read_config_from_disk()?;
        {
            let mut inner = self
                .inner
                .lock()
                .map_err(|e| format!("HotkeyManager 加锁失败: {e}"))?;
            inner.config = config;
            inner.status.last_error = None;
        }
        self.register_shortcuts(app)?;
        self.emit_status(app);
        Ok(())
    }

    pub fn config(&self) -> HotkeyConfig {
        self.inner
            .lock()
            .map(|inner| inner.config.clone())
            .unwrap_or_default()
    }

    pub fn status(&self) -> HotkeyStatus {
        self.inner
            .lock()
            .map(|inner| inner.status.clone())
            .unwrap_or_default()
    }

    pub fn save_config(
        &self,
        app: &AppHandle,
        config: HotkeyConfig,
    ) -> Result<HotkeyConfig, String> {
        validate_config(&config)?;

        let was_running = self.is_running();
        if was_running {
            self.stop_runner(app);
        }

        {
            let mut inner = self
                .inner
                .lock()
                .map_err(|e| format!("HotkeyManager 加锁失败: {e}"))?;
            inner.config = config.clone();
            inner.status.last_error = None;
        }

        self.write_config_to_disk(&config)?;
        self.register_shortcuts(app)?;
        self.emit_status(app);

        if was_running {
            if let Err(err) = self.start_runner(app) {
                self.record_error(app, err.clone());
                return Err(err);
            }
        }

        Ok(config)
    }

    pub fn start_runner(&self, app: &AppHandle) -> Result<(), String> {
        let (config, key) = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| format!("HotkeyManager 加锁失败: {e}"))?;

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
        let stop_clone = stop_flag.clone();
        let app_handle = app.clone();

        let handle = thread::spawn(move || {
            let mut enigo = Enigo::new();
            while !stop_clone.load(Ordering::SeqCst) {
                if let Err(err) = send_key(&mut enigo, key) {
                    log::error!("按键触发失败: {}", err);
                }
                sleep_with_interrupt(&stop_clone, config.interval_ms);
            }
            if let Some(manager) = app_handle.try_state::<HotkeyManager>() {
                manager.finish_running(&app_handle);
            }
        });

        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| format!("HotkeyManager 加锁失败: {e}"))?;
            guard.runner = Some(Runner::new(stop_flag, handle));
        }
        self.emit_status(app);
        Ok(())
    }

    pub fn stop_runner(&self, app: &AppHandle) {
        let mut guard = match self.inner.lock() {
            Ok(lock) => lock,
            Err(err) => {
                log::error!("HotkeyManager 加锁失败: {}", err);
                return;
            }
        };

        if let Some(mut runner) = guard.runner.take() {
            runner.request_stop();
            runner.join();
        }
        guard.status.running = false;
        drop(guard);
        self.emit_status(app);
    }

    fn finish_running(&self, app: &AppHandle) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.status.running = false;
            guard.runner = None;
        }
        self.emit_status(app);
    }

    fn record_error(&self, app: &AppHandle, message: String) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.status.last_error = Some(message);
            guard.status.running = false;
            guard.runner = None;
        }
        self.emit_status(app);
    }

    fn emit_status(&self, app: &AppHandle) {
        if let Ok(status) = self.inner.lock().map(|inner| inner.status.clone()) {
            if let Err(err) = app.emit(STATUS_EVENT, status.clone()) {
                log::warn!("广播热键状态失败: {}", err);
            }
        }
    }

    fn is_running(&self) -> bool {
        self.inner
            .lock()
            .map(|inner| inner.status.running)
            .unwrap_or(false)
    }

    fn register_shortcuts(&self, app: &AppHandle) -> Result<(), String> {
        let config = self
            .inner
            .lock()
            .map_err(|e| format!("HotkeyManager 加锁失败: {e}"))?
            .config
            .clone();

        let manager = app.global_shortcut();
        manager
            .unregister_all()
            .map_err(|e| format!("注销快捷键失败: {e}"))?;

        if config.start_hotkey.trim().is_empty() || config.stop_hotkey.trim().is_empty() {
            return Ok(());
        }

        let start_hotkey = config.start_hotkey.clone();
        manager
            .on_shortcut(
                start_hotkey.as_str(),
                move |app_handle, _, event: ShortcutEvent| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if let Some(manager) = app_handle.try_state::<HotkeyManager>() {
                        if let Err(err) = manager.start_runner(app_handle) {
                            log::error!("启动按键任务失败: {}", err);
                            manager.record_error(app_handle, err);
                        }
                    }
                },
            )
            .map_err(|e| format!("注册开始快捷键失败: {e}"))?;

        let stop_hotkey = config.stop_hotkey.clone();
        manager
            .on_shortcut(
                stop_hotkey.as_str(),
                move |app_handle, _, event: ShortcutEvent| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if let Some(manager) = app_handle.try_state::<HotkeyManager>() {
                        manager.stop_runner(app_handle);
                    }
                },
            )
            .map_err(|e| format!("注册结束快捷键失败: {e}"))?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf, String> {
        let mut path = dirs::config_dir().ok_or("无法定位配置目录")?;
        path.push("jx3-tools");
        fs::create_dir_all(&path).map_err(|e| format!("创建配置目录失败: {e}"))?;
        path.push(CONFIG_FILE);
        Ok(path)
    }

    fn read_config_from_disk(&self) -> Result<HotkeyConfig, String> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(HotkeyConfig::default());
        }
        let contents = fs::read_to_string(path).map_err(|e| format!("读取按键配置失败: {e}"))?;
        let config: HotkeyConfig =
            serde_json::from_str(&contents).map_err(|e| format!("解析按键配置失败: {e}"))?;
        Ok(config)
    }

    fn write_config_to_disk(&self, config: &HotkeyConfig) -> Result<(), String> {
        let path = Self::config_path()?;
        let data =
            serde_json::to_string_pretty(config).map_err(|e| format!("序列化按键配置失败: {e}"))?;
        fs::write(path, data).map_err(|e| format!("写入按键配置失败: {e}"))
    }
}

fn validate_config(config: &HotkeyConfig) -> Result<(), String> {
    if config.trigger_key.trim().is_empty() {
        return Err("触发按键不能为空".into());
    }
    if config.interval_ms == 0 {
        return Err("频率必须大于 0".into());
    }
    if config.start_hotkey.trim().is_empty() {
        return Err("开始热键不能为空".into());
    }
    if config.stop_hotkey.trim().is_empty() {
        return Err("结束热键不能为空".into());
    }
    if config
        .start_hotkey
        .eq_ignore_ascii_case(&config.stop_hotkey)
    {
        return Err("开始与结束热键不能相同".into());
    }

    Shortcut::from_str(&config.start_hotkey).map_err(|e| format!("开始热键格式无效: {e}"))?;
    Shortcut::from_str(&config.stop_hotkey).map_err(|e| format!("结束热键格式无效: {e}"))?;
    parse_trigger_key(&config.trigger_key)?;
    Ok(())
}

fn validate_runtime_config(config: &HotkeyConfig) -> Result<(), String> {
    if config.trigger_key.trim().is_empty() {
        return Err("触发按键未设置".into());
    }
    if config.interval_ms == 0 {
        return Err("频率必须大于 0".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn parse_trigger_key(label: &str) -> Result<Key, String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err("触发按键不能为空".into());
    }
    let upper = trimmed.to_uppercase();

    if upper.len() == 1 {
        if let Some(ch) = upper.chars().next() {
            return Ok(Key::Layout(ch));
        }
    }

    match upper.as_str() {
        "SPACE" => Ok(Key::Space),
        "ENTER" => Ok(Key::Return),
        "TAB" => Ok(Key::Tab),
        "ESC" | "ESCAPE" => Ok(Key::Escape),
        "UP" | "ARROWUP" => Ok(Key::UpArrow),
        "DOWN" | "ARROWDOWN" => Ok(Key::DownArrow),
        "LEFT" | "ARROWLEFT" => Ok(Key::LeftArrow),
        "RIGHT" | "ARROWRIGHT" => Ok(Key::RightArrow),
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
        "F13" => Ok(Key::F13),
        "F14" => Ok(Key::F14),
        "F15" => Ok(Key::F15),
        "F16" => Ok(Key::F16),
        "F17" => Ok(Key::F17),
        "F18" => Ok(Key::F18),
        "F19" => Ok(Key::F19),
        "F20" => Ok(Key::F20),
        "F21" => Ok(Key::F21),
        "F22" => Ok(Key::F22),
        "F23" => Ok(Key::F23),
        "F24" => Ok(Key::F24),
        "HOME" => Ok(Key::Home),
        "END" => Ok(Key::End),
        "PAGEUP" => Ok(Key::PageUp),
        "PAGEDOWN" => Ok(Key::PageDown),
        "INSERT" => Ok(Key::Insert),
        "DELETE" => Ok(Key::Delete),
        "BACKSPACE" => Ok(Key::Backspace),
        _ => Err(format!("暂不支持的触发按键: {trimmed}")),
    }
}

#[cfg(not(target_os = "windows"))]
fn parse_trigger_key(label: &str) -> Result<Key, String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err("触发按键不能为空".into());
    }
    let upper = trimmed.to_uppercase();

    if upper.len() == 1 {
        if let Some(ch) = upper.chars().next() {
            return Ok(Key::Layout(ch));
        }
    }

    match upper.as_str() {
        "SPACE" => Ok(Key::Space),
        "ENTER" => Ok(Key::Return),
        "TAB" => Ok(Key::Tab),
        "ESC" | "ESCAPE" => Ok(Key::Escape),
        "UP" | "ARROWUP" => Ok(Key::UpArrow),
        "DOWN" | "ARROWDOWN" => Ok(Key::DownArrow),
        "LEFT" | "ARROWLEFT" => Ok(Key::LeftArrow),
        "RIGHT" | "ARROWRIGHT" => Ok(Key::RightArrow),
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
        "F13" => Ok(Key::F13),
        "F14" => Ok(Key::F14),
        "F15" => Ok(Key::F15),
        "F16" => Ok(Key::F16),
        "F17" => Ok(Key::F17),
        "F18" => Ok(Key::F18),
        "F19" => Ok(Key::F19),
        "F20" => Ok(Key::F20),
        "HOME" => Ok(Key::Home),
        "END" => Ok(Key::End),
        "PAGEUP" => Ok(Key::PageUp),
        "PAGEDOWN" => Ok(Key::PageDown),
        "DELETE" => Ok(Key::Delete),
        "BACKSPACE" => Ok(Key::Backspace),
        _ => Err("按键功能仅在 Windows 平台可用".into()),
    }
}

fn send_key(enigo: &mut Enigo, key: Key) -> Result<(), String> {
    enigo.key_click(key);
    Ok(())
}

fn sleep_with_interrupt(flag: &Arc<AtomicBool>, total_ms: u64) {
    let mut remaining = if total_ms == 0 { 1 } else { total_ms };
    while remaining > 0 && !flag.load(Ordering::SeqCst) {
        let step = remaining.min(50);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}

#[tauri::command]
pub fn get_hotkey_config(state: tauri::State<'_, HotkeyManager>) -> HotkeyConfig {
    state.config()
}

#[tauri::command]
pub fn get_hotkey_status(state: tauri::State<'_, HotkeyManager>) -> HotkeyStatus {
    state.status()
}

#[tauri::command]
pub fn save_hotkey_config(
    app: AppHandle,
    state: tauri::State<'_, HotkeyManager>,
    config: HotkeyConfig,
) -> Result<HotkeyConfig, String> {
    state.save_config(&app, config)
}

#[tauri::command]
pub fn stop_hotkey_task(app: AppHandle, state: tauri::State<'_, HotkeyManager>) {
    state.stop_runner(&app);
}
