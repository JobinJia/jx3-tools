//! 按键模拟模块 - 使用 Interception 内核驱动注入按键
//!
//! 剑网三 TP 反作弊会过滤用户态合成输入（SendInput/keybd_event 带 INJECTED
//! 标志），必须用 Interception 内核驱动注入——其按键不带 INJECTED 标志，与真实
//! 键盘无法区分。驱动需用户安装（随安装包集成）并重启电脑后才可用。

#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use interception::{Interception, KeyState, ScanCode, Stroke};
use windows::core::w;
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::LoadLibraryW;

use super::keymap::KeyDef;
use crate::error::{AppError, AppResult};

/// 按键驱动状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Ready,
    NotInstalled,
}

/// 全局 Interception 发送上下文（延迟初始化，进程内仅创建一次）
static SENDER: OnceLock<Option<Sender>> = OnceLock::new();

struct Sender {
    ctx: Interception,
    /// 探测到的可用 keyboard device（0 = 尚未探测）
    device: AtomicI32,
}

// Interception 上下文内部是裸指针，手动声明跨线程安全：
// 实际只在 runner 线程串行调用，且发送本身是线程安全的内核调用。
unsafe impl Send for Sender {}
unsafe impl Sync for Sender {}

/// 检测 interception.dll 是否可加载。
/// 必须在触碰任何 delay-load 符号前调用：DLL 缺失时 LoadLibrary 会干净地返回错误，
/// 而直接调用 delay-load 函数会抛 SEH 异常，无法被 Rust 的 catch_unwind 捕获、会崩溃。
fn dll_loadable() -> bool {
    unsafe {
        match LoadLibraryW(w!("interception.dll")) {
            Ok(module) => {
                let _ = FreeLibrary(module);
                true
            }
            Err(_) => false,
        }
    }
}

fn init_sender() -> Option<Sender> {
    if !dll_loadable() {
        log::warn!("interception.dll 不可加载，按键驱动不可用");
        return None;
    }
    // DLL 在，但内核驱动（.sys）未安装/未重启时 create_context 返回 NULL → None
    let ctx = Interception::new()?;
    log::info!("Interception 发送上下文已创建");
    Some(Sender {
        ctx,
        device: AtomicI32::new(0),
    })
}

fn get_sender() -> Option<&'static Sender> {
    SENDER.get_or_init(init_sender).as_ref()
}

/// 查询按键驱动是否就绪（DLL 可加载且内核驱动已安装）
pub fn driver_status() -> DriverStatus {
    if get_sender().is_some() {
        DriverStatus::Ready
    } else {
        DriverStatus::NotInstalled
    }
}

impl Sender {
    /// 向指定 device 发送一次按下+释放，返回 keydown 是否注入成功
    fn try_send(&self, device: i32, code: ScanCode, extended: bool) -> bool {
        let base = if extended { KeyState::E0 } else { KeyState::empty() };
        let down = Stroke::Keyboard { code, state: base, information: 0 };
        let up = Stroke::Keyboard { code, state: base | KeyState::UP, information: 0 };

        if self.ctx.send(device, &[down]) <= 0 {
            return false;
        }
        thread::sleep(Duration::from_millis(10));
        self.ctx.send(device, &[up]);
        true
    }

    fn send_key(&self, key: KeyDef) -> AppResult<()> {
        let code = ScanCode::try_from(key.scancode)
            .map_err(|_| AppError::Hotkey(format!("无效的扫描码: {:#04x}", key.scancode)))?;

        // 优先复用已探测到的 device
        let cached = self.device.load(Ordering::Relaxed);
        if cached != 0 && self.try_send(cached, code, key.extended) {
            return Ok(());
        }

        // 探测可用 keyboard device（Interception 键盘 device 为 1..=10）
        for device in 1..=10 {
            if self.try_send(device, code, key.extended) {
                self.device.store(device, Ordering::Relaxed);
                return Ok(());
            }
        }
        Err(AppError::Hotkey(
            "Interception 注入按键失败，请确认驱动已安装并重启电脑".into(),
        ))
    }
}

/// 模拟按键点击（按下 + 释放），经 Interception 内核注入
pub fn simulate_key_press(key: KeyDef) -> AppResult<()> {
    let sender = get_sender().ok_or_else(|| {
        AppError::Hotkey("按键驱动未就绪，请安装 Interception 驱动并重启电脑".into())
    })?;
    sender.send_key(key)
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
