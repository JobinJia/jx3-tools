//! PowerShell script templates for MAC address operations
//! These are only used on Windows

#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

/// Script to get primary network adapter information
pub const GET_ADAPTER: &str = include_str!("scripts/get_adapter.ps1");

/// Template for restarting a network adapter
/// Replace {NAME} with the adapter name
pub const RESTART_ADAPTER_TEMPLATE: &str = include_str!("scripts/restart_adapter.ps1");

/// Template for setting/removing NetworkAddress registry value
/// Replace {GUID} with the interface GUID
/// Replace {ACTION} with the appropriate PowerShell command
pub const SET_NETWORK_ADDRESS_TEMPLATE: &str = include_str!("scripts/set_network_address.ps1");

/// Generate the action for setting a MAC address value
pub fn set_mac_action(mac_value: &str) -> String {
    format!(
        "Set-ItemProperty -Path $target.PSPath -Name 'NetworkAddress' -Value '{}' -Force",
        mac_value
    )
}

/// Generate the action for removing the MAC address value (restore original)
pub fn remove_mac_action() -> &'static str {
    "Remove-ItemProperty -Path $target.PSPath -Name 'NetworkAddress' -ErrorAction SilentlyContinue"
}
