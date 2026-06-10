//! PowerShell script assembly for MAC address operations
//! These are only used on Windows. Each script is the shared prelude
//! (helper functions) plus a feature-specific body.

#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

const COMMON: &str = include_str!("scripts/common.ps1");
const GET_MAC_INFO: &str = include_str!("scripts/get_mac_info.ps1");
const CHANGE_MAC: &str = include_str!("scripts/change_mac.ps1");
const RESTORE_MAC: &str = include_str!("scripts/restore_mac.ps1");

/// Script to query the primary adapter's MAC info (read-only)
pub fn get_mac_info_script() -> String {
    format!("{COMMON}\n{GET_MAC_INFO}")
}

/// Script to apply `mac` (12 uppercase hex chars) and verify the driver accepted it
pub fn change_mac_script(mac: &str) -> String {
    format!("{COMMON}\n{}", CHANGE_MAC.replace("{MAC}", mac))
}

/// Script to remove all MAC overrides and fall back to permanent addresses
pub fn restore_mac_script() -> String {
    format!("{COMMON}\n{RESTORE_MAC}")
}
