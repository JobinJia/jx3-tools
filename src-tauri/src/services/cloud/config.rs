//! 云同步账号配置持久化：服务器地址 + 账号 + 应用密码，存
//! `dirs::config_dir()/jx3-tools/cloud_config.json`（与 hotkey_config.json 同目录）。
//! 应用密码是网盘侧可单独吊销的第三方授权密码，不是网盘登录密码；
//! 明文落盘是当前取舍（目录受用户档案保护），后续可换 DPAPI。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::webdav::normalize_base_url;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudConfig {
    pub server_url: String,
    pub username: String,
    pub app_password: String,
}

pub fn validate(config: &CloudConfig) -> AppResult<()> {
    normalize_base_url(&config.server_url)?;
    if config.username.trim().is_empty() {
        return Err(AppError::Cloud("账号不能为空".into()));
    }
    if config.app_password.trim().is_empty() {
        return Err(AppError::Cloud("应用密码不能为空".into()));
    }
    Ok(())
}

fn config_path() -> AppResult<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| AppError::Cloud("无法定位系统配置目录".into()))?
        .join("jx3-tools");
    Ok(dir.join("cloud_config.json"))
}

pub fn load_config() -> AppResult<Option<CloudConfig>> {
    load_from(&config_path()?)
}

pub fn save_config(config: &CloudConfig) -> AppResult<()> {
    save_to(config, &config_path()?)
}

fn load_from(path: &Path) -> AppResult<Option<CloudConfig>> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| AppError::Cloud(format!("云同步配置解析失败: {e}")))
}

fn save_to(config: &CloudConfig, path: &Path) -> AppResult<()> {
    validate(config)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(config)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> CloudConfig {
        CloudConfig {
            server_url: "https://dav.jianguoyun.com/dav/".into(),
            username: "user@example.com".into(),
            app_password: "app-pass".into(),
        }
    }

    #[test]
    fn validate_accepts_valid_and_rejects_blank_fields() {
        assert!(validate(&valid()).is_ok());
        assert!(validate(&CloudConfig {
            server_url: "not a url".into(),
            ..valid()
        })
        .is_err());
        assert!(validate(&CloudConfig {
            username: "  ".into(),
            ..valid()
        })
        .is_err());
        assert!(validate(&CloudConfig {
            app_password: "".into(),
            ..valid()
        })
        .is_err());
    }

    #[test]
    fn save_load_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "jx3-cloudcfg-test-{}/cloud_config.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        assert!(load_from(&path).unwrap().is_none());
        save_to(&valid(), &path).unwrap();
        let loaded = load_from(&path).unwrap().unwrap();
        assert_eq!(loaded.username, "user@example.com");
        assert_eq!(loaded.server_url, "https://dav.jianguoyun.com/dav/");

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
