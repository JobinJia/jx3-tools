//! 按键模拟模块 - 直连 Interception 键盘设备注入按键
//!
//! 剑网三 TP 反作弊会过滤用户态合成输入（SendInput/keybd_event 带 INJECTED
//! 标志），必须经 Interception 内核驱动注入——其按键不带 INJECTED 标志，与真实
//! 键盘无法区分。
//!
//! 不使用 interception.dll：它的 create_context 要求键盘+鼠标全部 20 个设备
//! 都能打开，而本工具刻意只安装键盘过滤器（鼠标过滤器有搞瘫鼠标的前科，见
//! driver.rs），dll 在这种部署下必然失败。注入协议非常简单（对
//! `\\.\interception0N` 设备发 IOCTL_WRITE + KEYBOARD_INPUT_DATA，已从官方
//! 库源码确认），这里直接实现，顺带去掉了 DLL 分发与 /DELAYLOAD 防崩 hack。

#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;

use super::keymap::KeyDef;
use crate::error::{AppError, AppResult};

/// 按键驱动状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Ready,
    NotInstalled,
}

/// Interception 键盘设备数（设备文件 \\.\interception00 ~ 09；10~19 是鼠标，不碰）
const KEYBOARD_DEVICE_COUNT: usize = 10;

/// IOCTL_WRITE = CTL_CODE(FILE_DEVICE_UNKNOWN, 0x820, METHOD_BUFFERED, FILE_ANY_ACCESS)
const IOCTL_WRITE: u32 = 0x0022_2080;

/// 驱动侧的 KEYBOARD_INPUT_DATA（12 字节，布局见 wdm.h / interception 库源码）
#[repr(C)]
#[derive(Clone, Copy)]
struct KeyboardInputData {
    unit_id: u16,
    make_code: u16,
    /// KEY_MAKE=0 / KEY_BREAK=1 / KEY_E0=2，与 UI 层语义一致
    flags: u16,
    reserved: u16,
    extra_information: u32,
}

const KEY_BREAK: u16 = 0x01;
const KEY_E0: u16 = 0x02;

/// 已打开的键盘设备句柄。HANDLE 是裸指针不自动 Send/Sync，但内核对象句柄
/// 本身可跨线程使用，DeviceIoControl 也是线程安全的内核调用。
struct Device {
    handle: HANDLE,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// 全局发送器（延迟初始化，进程内仅创建一次）
static SENDER: OnceLock<Option<Sender>> = OnceLock::new();

struct Sender {
    devices: Vec<Device>,
    /// 上次注入成功的设备下标（加速后续发送）
    cached: AtomicUsize,
}

fn open_keyboard_device(index: usize) -> Option<Device> {
    let path: Vec<u16> = format!(r"\\.\interception{index:02}")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            PCWSTR(path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    };
    match handle {
        Ok(handle) => Some(Device { handle }),
        Err(_) => None,
    }
}

fn init_sender() -> Option<Sender> {
    // 驱动加载时会一次性创建全部 10 个键盘设备：一个都打不开 = 驱动未加载
    let devices: Vec<Device> = (0..KEYBOARD_DEVICE_COUNT)
        .filter_map(open_keyboard_device)
        .collect();
    if devices.is_empty() {
        log::warn!("无法打开任何 interception 键盘设备，按键驱动不可用");
        return None;
    }
    log::info!("Interception 键盘设备已打开（{} 个）", devices.len());
    Some(Sender {
        devices,
        cached: AtomicUsize::new(usize::MAX),
    })
}

fn get_sender() -> Option<&'static Sender> {
    SENDER.get_or_init(init_sender).as_ref()
}

/// 查询按键驱动是否就绪（键盘设备可打开 = 内核驱动已加载）
pub fn driver_status() -> DriverStatus {
    if get_sender().is_some() {
        DriverStatus::Ready
    } else {
        DriverStatus::NotInstalled
    }
}

impl Sender {
    /// 向单个设备写入一组键击，返回是否全部写入
    fn write_strokes(&self, device: &Device, strokes: &[KeyboardInputData]) -> bool {
        let size = std::mem::size_of_val(strokes) as u32;
        let mut written: u32 = 0;
        let ok = unsafe {
            DeviceIoControl(
                device.handle,
                IOCTL_WRITE,
                Some(strokes.as_ptr().cast()),
                size,
                None,
                0,
                Some(&mut written),
                None,
            )
        };
        ok.is_ok() && written == size
    }

    /// 向指定设备发送一次按下+释放，返回 keydown 是否注入成功
    fn try_send(&self, device: &Device, scancode: u16, extended: bool) -> bool {
        let base = if extended { KEY_E0 } else { 0 };
        let stroke = |flags: u16| KeyboardInputData {
            unit_id: 0,
            make_code: scancode,
            flags,
            reserved: 0,
            extra_information: 0,
        };

        if !self.write_strokes(device, &[stroke(base)]) {
            return false;
        }
        thread::sleep(Duration::from_millis(10));
        self.write_strokes(device, &[stroke(base | KEY_BREAK)]);
        true
    }

    fn send_key(&self, key: KeyDef) -> AppResult<()> {
        // 优先复用已探测到的设备
        let cached = self.cached.load(Ordering::Relaxed);
        if let Some(device) = self.devices.get(cached) {
            if self.try_send(device, key.scancode, key.extended) {
                return Ok(());
            }
        }

        // 探测可用键盘设备（只有挂着真实键盘的设备注入才会被接收）
        for (index, device) in self.devices.iter().enumerate() {
            if index == cached {
                continue;
            }
            if self.try_send(device, key.scancode, key.extended) {
                self.cached.store(index, Ordering::Relaxed);
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
        AppError::Hotkey("按键驱动未就绪，请先在本页面安装驱动并重启电脑".into())
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
