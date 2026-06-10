//! Key label resolution
//!
//! Single source of truth mapping UI key labels (what the frontend records,
//! e.g. "F5", "A", ";", "Up", "Ctrl+Alt+X") to:
//! - Windows scancode + virtual-key code for key simulation (`resolve_key`)
//! - global-shortcut strings for hotkey registration (`label_to_shortcut`)

use tauri_plugin_global_shortcut::Shortcut;

use crate::error::{AppError, AppResult};

/// A resolved key for simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyDef {
    /// PS/2 set-1 scancode (for SendInput KEYEVENTF_SCANCODE)
    pub scancode: u16,
    /// Windows virtual-key code (for PostMessage WM_KEYDOWN)
    pub vk: u16,
    /// 0xE0-prefixed extended key (arrows/nav keys need KEYEVENTF_EXTENDEDKEY,
    /// otherwise SendInput emits the numpad variant)
    pub extended: bool,
}

const fn key(scancode: u16, vk: u16) -> KeyDef {
    KeyDef { scancode, vk, extended: false }
}

const fn ext_key(scancode: u16, vk: u16) -> KeyDef {
    KeyDef { scancode, vk, extended: true }
}

/// Resolve a single key label (no modifier combos) for simulation
pub fn resolve_key(label: &str) -> AppResult<KeyDef> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("按键不能为空".into()));
    }

    let upper = trimmed.to_uppercase();

    // Letters A-Z / digits 0-9
    if upper.len() == 1 {
        let ch = upper.chars().next().unwrap_or_default();
        if ch.is_ascii_uppercase() {
            return Ok(key(letter_scancode(ch), ch as u16));
        }
        if ch.is_ascii_digit() {
            return Ok(key(digit_scancode(ch), ch as u16));
        }
    }

    let def = match upper.as_str() {
        "ESC" | "ESCAPE" => key(0x01, 0x1B),
        "F1" => key(0x3B, 0x70),
        "F2" => key(0x3C, 0x71),
        "F3" => key(0x3D, 0x72),
        "F4" => key(0x3E, 0x73),
        "F5" => key(0x3F, 0x74),
        "F6" => key(0x40, 0x75),
        "F7" => key(0x41, 0x76),
        "F8" => key(0x42, 0x77),
        "F9" => key(0x43, 0x78),
        "F10" => key(0x44, 0x79),
        "F11" => key(0x57, 0x7A),
        "F12" => key(0x58, 0x7B),
        "BACKSPACE" => key(0x0E, 0x08),
        "TAB" => key(0x0F, 0x09),
        "ENTER" | "RETURN" => key(0x1C, 0x0D),
        "CTRL" | "CONTROL" | "LCTRL" => key(0x1D, 0x11),
        "SHIFT" | "LSHIFT" => key(0x2A, 0x10),
        "RSHIFT" => key(0x36, 0xA1),
        "ALT" | "LALT" => key(0x38, 0x12),
        "SPACE" => key(0x39, 0x20),
        "CAPSLOCK" | "CAPS" => key(0x3A, 0x14),
        "NUMLOCK" => key(0x45, 0x90),
        "SCROLLLOCK" => key(0x46, 0x91),
        "HOME" => ext_key(0x47, 0x24),
        "UP" | "ARROWUP" => ext_key(0x48, 0x26),
        "PAGEUP" => ext_key(0x49, 0x21),
        "LEFT" | "ARROWLEFT" => ext_key(0x4B, 0x25),
        "RIGHT" | "ARROWRIGHT" => ext_key(0x4D, 0x27),
        "END" => ext_key(0x4F, 0x23),
        "DOWN" | "ARROWDOWN" => ext_key(0x50, 0x28),
        "PAGEDOWN" => ext_key(0x51, 0x22),
        "INSERT" => ext_key(0x52, 0x2D),
        "DELETE" | "DEL" => ext_key(0x53, 0x2E),
        // Numpad
        "NUM0" | "NUMPAD0" => key(0x52, 0x60),
        "NUM1" | "NUMPAD1" => key(0x4F, 0x61),
        "NUM2" | "NUMPAD2" => key(0x50, 0x62),
        "NUM3" | "NUMPAD3" => key(0x51, 0x63),
        "NUM4" | "NUMPAD4" => key(0x4B, 0x64),
        "NUM5" | "NUMPAD5" => key(0x4C, 0x65),
        "NUM6" | "NUMPAD6" => key(0x4D, 0x66),
        "NUM7" | "NUMPAD7" => key(0x47, 0x67),
        "NUM8" | "NUMPAD8" => key(0x48, 0x68),
        "NUM9" | "NUMPAD9" => key(0x49, 0x69),
        "NUMMUL" | "NUMSTAR" | "NUMMULTIPLY" => key(0x37, 0x6A),
        "NUMSUB" | "NUMMINUS" => key(0x4A, 0x6D),
        "NUMADD" | "NUMPLUS" => key(0x4E, 0x6B),
        "NUMDOT" | "NUMDECIMAL" => key(0x53, 0x6E),
        "NUMDIV" | "NUMSLASH" | "NUMDIVIDE" => ext_key(0x35, 0x6F),
        // OEM keys
        ";" | "SEMICOLON" | "OEM1" => key(0x27, 0xBA),
        "=" | "EQUALS" | "OEMPLUS" => key(0x0D, 0xBB),
        "," | "COMMA" | "OEMCOMMA" => key(0x33, 0xBC),
        "-" | "MINUS" | "OEMMINUS" => key(0x0C, 0xBD),
        "." | "PERIOD" | "OEMPERIOD" => key(0x34, 0xBE),
        "/" | "SLASH" | "OEM2" => key(0x35, 0xBF),
        "`" | "GRAVE" | "BACKQUOTE" | "OEM3" => key(0x29, 0xC0),
        "[" | "BRACKETLEFT" | "OEM4" => key(0x1A, 0xDB),
        "\\" | "BACKSLASH" | "OEM5" => key(0x2B, 0xDC),
        "]" | "BRACKETRIGHT" | "OEM6" => key(0x1B, 0xDD),
        "'" | "QUOTE" | "OEM7" => key(0x28, 0xDE),
        _ => return Err(AppError::Hotkey(format!("不支持的按键: {trimmed}"))),
    };

    Ok(def)
}

/// Convert a hotkey label (single key or "Ctrl+Alt+X" combo) to a
/// global-shortcut string. Key token validity is checked when the caller
/// parses the result into a `Shortcut`.
pub fn label_to_shortcut(label: &str) -> AppResult<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("热键不能为空".into()));
    }

    let parts: Vec<&str> = trimmed.split('+').map(str::trim).collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(AppError::Hotkey(format!("热键格式无效: {trimmed}")));
    }

    let (key_part, modifiers) = parts.split_last().unwrap_or((&trimmed, &[]));

    let mut tokens: Vec<String> = Vec::with_capacity(parts.len());
    for modifier in modifiers {
        let normalized = match modifier.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => "ctrl",
            "ALT" | "OPTION" => "alt",
            "SHIFT" => "shift",
            // global-hotkey 只认 CMD/COMMAND/SUPER，不认 WIN
            "WIN" | "WINDOWS" | "META" | "CMD" | "COMMAND" | "SUPER" => "super",
            other => return Err(AppError::Hotkey(format!("无效的修饰键: {other}"))),
        };
        tokens.push(normalized.to_string());
    }
    tokens.push((*key_part).to_string());

    Ok(tokens.join("+"))
}

/// Parse a hotkey label into a registrable `Shortcut`
pub fn parse_shortcut(label: &str) -> AppResult<Shortcut> {
    let normalized = label_to_shortcut(label)?;
    normalized
        .parse::<Shortcut>()
        .map_err(|e| AppError::Hotkey(format!("无效的热键 {}: {e}", label.trim())))
}

fn letter_scancode(ch: char) -> u16 {
    match ch {
        'A' => 0x1E,
        'B' => 0x30,
        'C' => 0x2E,
        'D' => 0x20,
        'E' => 0x12,
        'F' => 0x21,
        'G' => 0x22,
        'H' => 0x23,
        'I' => 0x17,
        'J' => 0x24,
        'K' => 0x25,
        'L' => 0x26,
        'M' => 0x32,
        'N' => 0x31,
        'O' => 0x18,
        'P' => 0x19,
        'Q' => 0x10,
        'R' => 0x13,
        'S' => 0x1F,
        'T' => 0x14,
        'U' => 0x16,
        'V' => 0x2F,
        'W' => 0x11,
        'X' => 0x2D,
        'Y' => 0x15,
        'Z' => 0x2C,
        _ => 0,
    }
}

fn digit_scancode(ch: char) -> u16 {
    match ch {
        '0' => 0x0B,
        '1' => 0x02,
        '2' => 0x03,
        '3' => 0x04,
        '4' => 0x05,
        '5' => 0x06,
        '6' => 0x07,
        '7' => 0x08,
        '8' => 0x09,
        '9' => 0x0A,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_letters_digits_and_function_keys() {
        assert_eq!(resolve_key("a").unwrap(), key(0x1E, 0x41));
        assert_eq!(resolve_key("5").unwrap(), key(0x06, 0x35));
        assert_eq!(resolve_key("F5").unwrap(), key(0x3F, 0x74));
        assert_eq!(resolve_key(" f11 ").unwrap(), key(0x57, 0x7A));
    }

    #[test]
    fn resolve_arrows_and_nav_keys_are_extended() {
        let up = resolve_key("Up").unwrap();
        assert_eq!(up, ext_key(0x48, 0x26));
        assert!(resolve_key("Delete").unwrap().extended);
        assert!(resolve_key("PageDown").unwrap().extended);
        assert!(!resolve_key("Num8").unwrap().extended);
    }

    #[test]
    fn resolve_rejects_unknown_and_empty_labels() {
        assert!(resolve_key("").is_err());
        assert!(resolve_key("F13").is_err());
        assert!(resolve_key("Ctrl+A").is_err());
    }

    #[test]
    fn shortcut_passes_single_keys_through() {
        assert_eq!(label_to_shortcut("F11").unwrap(), "F11");
        assert_eq!(label_to_shortcut("a").unwrap(), "a");
        assert_eq!(label_to_shortcut(";").unwrap(), ";");
    }

    #[test]
    fn shortcut_normalizes_modifier_combos() {
        assert_eq!(label_to_shortcut("Ctrl+Alt+F5").unwrap(), "ctrl+alt+F5");
        assert_eq!(label_to_shortcut("Win+X").unwrap(), "super+X");
        assert_eq!(label_to_shortcut("Shift + Space").unwrap(), "shift+Space");
    }

    #[test]
    fn shortcut_rejects_malformed_combos() {
        assert!(label_to_shortcut("").is_err());
        assert!(label_to_shortcut("Ctrl+").is_err());
        assert!(label_to_shortcut("Foo+X").is_err());
    }

    #[test]
    fn parse_shortcut_accepts_frontend_labels() {
        assert!(parse_shortcut("F11").is_ok());
        assert!(parse_shortcut("Ctrl+Alt+F5").is_ok());
        assert!(parse_shortcut("Win+P").is_ok());
        assert!(parse_shortcut("Up").is_ok());
        assert!(parse_shortcut(";").is_ok());
    }

    #[test]
    fn parse_shortcut_rejects_modifier_only_and_unknown_keys() {
        assert!(parse_shortcut("Ctrl").is_err());
        assert!(parse_shortcut("不存在").is_err());
    }

    #[test]
    fn parse_shortcut_equality_ignores_label_spelling() {
        assert_eq!(
            parse_shortcut("Control+A").unwrap(),
            parse_shortcut("ctrl+a").unwrap()
        );
    }
}
