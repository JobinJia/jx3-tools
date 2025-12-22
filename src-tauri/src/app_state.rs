use std::sync::Arc;

use tauri::AppHandle;

use crate::error::AppResult;
use crate::services::{hotkey::HotkeyService, mac::MacService};

pub struct AppState {
    hotkey: Arc<HotkeyService>,
    mac: Arc<MacService>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> AppResult<Self> {
        let mac = Arc::new(MacService::new()?);
        let hotkey = Arc::new(HotkeyService::new()?);
        hotkey.initialize(app)?;
        Ok(Self { hotkey, mac })
    }

    pub fn hotkey(&self) -> Arc<HotkeyService> {
        self.hotkey.clone()
    }

    pub fn mac(&self) -> Arc<MacService> {
        self.mac.clone()
    }
}
