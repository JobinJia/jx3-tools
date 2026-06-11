//! 云同步编排：上传 = 角色键位目录 + 插件 config 打 zip 存入网盘并登记 manifest；
//! 下载 = 按 manifest 拉包、解到临时目录、swap_replace_dir 交换就位（失败不伤本地）。
//!
//! 云端结构（用户自己的网盘）：
//! ```text
//! jx3-tools/
//!   manifest.json                      角色清单（云端列表的唯一数据源，不做 PROPFIND 遍历）
//!   roles/<服务器>_<角色名>/
//!     keybinding.zip                   userdata 角色目录内容
//!     plugins.zip                      my#data/config/**、SG#data/data.jx3dat（按数据目录归档，
//!                                      UID 不进包——下载时按目标角色重新反查，天然跨账号）
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::pack;
use super::webdav::CloudStorage;
use crate::error::{AppError, AppResult};
use crate::services::keyboard::KeyboardService;
use crate::services::plugin_data::{PluginDataService, SkippedItem};

const MANIFEST_PATH: &str = "jx3-tools/manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudRoleEntry {
    pub key: String,
    pub name: String,
    pub server: String,
    pub uploaded_at: u64,
    pub device: String,
    pub keybinding_file: String,
    pub plugins_file: Option<String>,
    pub keybinding_size: u64,
    pub plugins_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudManifest {
    pub version: u32,
    pub roles: Vec<CloudRoleEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudUploadReport {
    pub key: String,
    pub keybinding_size: u64,
    pub plugins_size: u64,
    pub plugin_dirs: Vec<String>,
    pub skipped: Vec<SkippedItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudDownloadReport {
    pub keybinding_applied: bool,
    pub plugin_dirs: Vec<String>,
    pub skipped: Vec<SkippedItem>,
}

static TEMP_SEQ: AtomicU32 = AtomicU32::new(0);

/// 进程内唯一的临时目录路径（不创建）
fn unique_temp(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "jx3-cloud-{}-{}-{}",
        std::process::id(),
        label,
        TEMP_SEQ.fetch_add(1, Ordering::SeqCst)
    ))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "未知设备".to_string())
}

/// 插件归集产物：打包字节流 + 收录的数据目录 + 跳过原因
struct StagedPlugins {
    bytes: Option<Vec<u8>>,
    included: Vec<String>,
    skipped: Vec<SkippedItem>,
}

pub struct CloudSyncService;

impl CloudSyncService {
    /// 上传角色：键位 zip + 插件 config zip + 更新 manifest
    pub fn upload_role(
        storage: &dyn CloudStorage,
        role_path: &Path,
    ) -> AppResult<CloudUploadReport> {
        if !role_path.is_dir() {
            return Err(AppError::Cloud(format!(
                "角色目录不存在: {}",
                role_path.display()
            )));
        }
        let (name, server) = PluginDataService::role_identity(role_path)?;
        let key = format!("{server}/{name}");
        let cloud_dir = format!("jx3-tools/roles/{server}_{name}");

        let keybinding_bytes = pack::pack_dir(role_path)?;
        let StagedPlugins {
            bytes: plugins_bytes,
            included: plugin_dirs,
            skipped,
        } = Self::stage_plugins(role_path, &name, &server)?;

        let keybinding_file = format!("{cloud_dir}/keybinding.zip");
        storage.put(&keybinding_file, &keybinding_bytes)?;
        let plugins_file = match &plugins_bytes {
            Some(bytes) => {
                let path = format!("{cloud_dir}/plugins.zip");
                storage.put(&path, bytes)?;
                Some(path)
            }
            None => None,
        };

        // 上传文件就位后再登记 manifest，列表里永远不会出现拉不到包的条目
        let mut manifest = Self::load_manifest(storage)?;
        manifest.roles.retain(|r| r.key != key);
        manifest.roles.push(CloudRoleEntry {
            key: key.clone(),
            name,
            server,
            uploaded_at: now_secs(),
            device: device_name(),
            keybinding_file,
            plugins_file,
            keybinding_size: keybinding_bytes.len() as u64,
            plugins_size: plugins_bytes.as_ref().map_or(0, |b| b.len() as u64),
        });
        manifest.roles.sort_by(|a, b| a.key.cmp(&b.key));
        storage.put(
            MANIFEST_PATH,
            &serde_json::to_vec_pretty(&manifest)
                .map_err(|e| AppError::Cloud(format!("生成 manifest 失败: {e}")))?,
        )?;

        log::info!(
            "云端上传完成: {key}，键位 {} 字节，插件 {:?}",
            keybinding_bytes.len(),
            plugin_dirs
        );
        Ok(CloudUploadReport {
            key,
            keybinding_size: keybinding_bytes.len() as u64,
            plugins_size: plugins_bytes.map_or(0, |b| b.len() as u64),
            plugin_dirs,
            skipped,
        })
    }

    /// 云端角色列表（读 manifest）
    pub fn list_roles(storage: &dyn CloudStorage) -> AppResult<Vec<CloudRoleEntry>> {
        Ok(Self::load_manifest(storage)?.roles)
    }

    /// 下载云端角色到本地目标角色：键位交换就位，插件按目标角色 UID 反查落位
    pub fn download_role(
        storage: &dyn CloudStorage,
        key: &str,
        target_role_path: &Path,
    ) -> AppResult<CloudDownloadReport> {
        let manifest = Self::load_manifest(storage)?;
        let entry = manifest
            .roles
            .iter()
            .find(|r| r.key == key)
            .ok_or_else(|| AppError::Cloud(format!("云端没有该角色的存档: {key}")))?;

        // 键位：拉包 → 解到临时目录 → 交换就位
        let keybinding_bytes = storage.get(&entry.keybinding_file)?.ok_or_else(|| {
            AppError::Cloud(format!("云端键位文件丢失: {}", entry.keybinding_file))
        })?;
        let kb_tmp = unique_temp("kb");
        pack::unpack_to_dir(&keybinding_bytes, &kb_tmp)?;
        let swap_result = KeyboardService::swap_replace_dir(&kb_tmp, target_role_path);
        let _ = fs::remove_dir_all(&kb_tmp);
        swap_result?;

        let mut report = CloudDownloadReport {
            keybinding_applied: true,
            plugin_dirs: vec![],
            skipped: vec![],
        };

        // 插件：拉包 → 解包 → 按目标角色 UID 反查落位
        if let Some(plugins_file) = &entry.plugins_file {
            match storage.get(plugins_file)? {
                Some(bytes) => {
                    let plugin_tmp = unique_temp("plugins");
                    pack::unpack_to_dir(&bytes, &plugin_tmp)?;
                    Self::apply_plugins(&plugin_tmp, target_role_path, &mut report);
                    let _ = fs::remove_dir_all(&plugin_tmp);
                }
                None => report.skipped.push(SkippedItem {
                    dir: "插件配置".into(),
                    reason: format!("云端插件包丢失: {plugins_file}"),
                }),
            }
        }

        log::info!(
            "云端下载完成: {key} -> {}，插件 {:?}，跳过 {} 项",
            target_role_path.display(),
            report.plugin_dirs,
            report.skipped.len()
        );
        Ok(report)
    }

    fn load_manifest(storage: &dyn CloudStorage) -> AppResult<CloudManifest> {
        match storage.get(MANIFEST_PATH)? {
            None => Ok(CloudManifest {
                version: 1,
                roles: vec![],
            }),
            Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                AppError::Cloud(format!(
                    "云端 manifest.json 解析失败（文件可能损坏，可在网盘中删除 jx3-tools/manifest.json 后重新上传）: {e}"
                ))
            }),
        }
    }

    /// 把源角色的插件数据按数据目录归集到临时树并打包：
    /// `<数据目录名>/config/**`（框架式）、`<数据目录名>/data.jx3dat`（单文件式）。
    /// UID 不进包，下载端按目标角色重新反查。
    fn stage_plugins(role_path: &Path, name: &str, server: &str) -> AppResult<StagedPlugins> {
        let mut skipped = vec![];
        let interface = match PluginDataService::locate_interface_dir(role_path) {
            Ok(dir) => dir,
            Err(e) => {
                skipped.push(SkippedItem {
                    dir: "插件配置".into(),
                    reason: e.to_string(),
                });
                return Ok(StagedPlugins {
                    bytes: None,
                    included: vec![],
                    skipped,
                });
            }
        };
        let data_dirs = PluginDataService::collect_data_dirs(&interface)?;
        let Some(uid) = PluginDataService::resolve_uid(&data_dirs, name, server) else {
            skipped.push(SkippedItem {
                dir: "插件配置".into(),
                reason: "未找到该角色的插件数据（角色未用插件登录过），本次只上传键位".into(),
            });
            return Ok(StagedPlugins {
                bytes: None,
                included: vec![],
                skipped,
            });
        };

        let staging = unique_temp("stage");
        fs::create_dir_all(&staging)?;
        let mut included = vec![];
        for dir in &data_dirs {
            let dir_name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let (framework_style, single_file_style) = PluginDataService::dir_style(dir);
            if framework_style {
                let Some(uid_entry) = PluginDataService::find_uid_entry(dir, &uid) else {
                    skipped.push(SkippedItem {
                        dir: dir_name,
                        reason: "该插件下无此角色数据".into(),
                    });
                    continue;
                };
                let config = uid_entry.join("config");
                if !config.is_dir() {
                    skipped.push(SkippedItem {
                        dir: dir_name,
                        reason: "该插件下没有 config 配置".into(),
                    });
                    continue;
                }
                KeyboardService::copy_dir_all(&config, &staging.join(&dir_name).join("config"))?;
                included.push(dir_name);
            } else if single_file_style {
                let file = dir.join(format!("{uid}.jx3dat"));
                if !file.is_file() {
                    skipped.push(SkippedItem {
                        dir: dir_name,
                        reason: "该插件下无此角色数据".into(),
                    });
                    continue;
                }
                fs::create_dir_all(staging.join(&dir_name))?;
                fs::copy(&file, staging.join(&dir_name).join("data.jx3dat"))?;
                included.push(dir_name);
            }
            // 全局式：天然共享，不进包
        }

        let bytes = if included.is_empty() {
            None
        } else {
            Some(pack::pack_dir(&staging)?)
        };
        let _ = fs::remove_dir_all(&staging);
        Ok(StagedPlugins {
            bytes,
            included,
            skipped,
        })
    }

    /// 把解包后的插件归档落到目标角色：按目录名匹配本机数据目录，按目标 UID 落位
    fn apply_plugins(staging: &Path, target_role_path: &Path, report: &mut CloudDownloadReport) {
        let (tgt_name, tgt_server) = match PluginDataService::role_identity(target_role_path) {
            Ok(identity) => identity,
            Err(e) => {
                report.skipped.push(SkippedItem {
                    dir: "插件配置".into(),
                    reason: e.to_string(),
                });
                return;
            }
        };
        let interface = match PluginDataService::locate_interface_dir(target_role_path) {
            Ok(dir) => dir,
            Err(e) => {
                report.skipped.push(SkippedItem {
                    dir: "插件配置".into(),
                    reason: e.to_string(),
                });
                return;
            }
        };
        let data_dirs = match PluginDataService::collect_data_dirs(&interface) {
            Ok(dirs) => dirs,
            Err(e) => {
                report.skipped.push(SkippedItem {
                    dir: "插件配置".into(),
                    reason: e.to_string(),
                });
                return;
            }
        };
        let Some(uid) = PluginDataService::resolve_uid(&data_dirs, &tgt_name, &tgt_server) else {
            report.skipped.push(SkippedItem {
                dir: "插件配置".into(),
                reason: format!(
                    "未找到目标角色 {tgt_name}（{tgt_server}）的插件数据，请先用该角色登录一次游戏后重试"
                ),
            });
            return;
        };

        let Ok(entries) = fs::read_dir(staging) else {
            return;
        };
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            let Some(local_dir) = data_dirs.iter().find(|d| {
                d.file_name()
                    .is_some_and(|n| n.to_string_lossy() == dir_name)
            }) else {
                report.skipped.push(SkippedItem {
                    dir: dir_name,
                    reason: "本机未安装该插件（无对应数据目录）".into(),
                });
                continue;
            };

            let staged_config = entry.path().join("config");
            let staged_single = entry.path().join("data.jx3dat");
            if staged_config.is_dir() {
                let Some(uid_entry) = PluginDataService::find_uid_entry(local_dir, &uid) else {
                    report.skipped.push(SkippedItem {
                        dir: dir_name,
                        reason: "目标角色在该插件下无数据（请先用目标角色登录一次游戏）".into(),
                    });
                    continue;
                };
                match KeyboardService::swap_replace_dir(&staged_config, &uid_entry.join("config")) {
                    Ok(()) => report.plugin_dirs.push(dir_name),
                    Err(e) => report.skipped.push(SkippedItem {
                        dir: dir_name,
                        reason: format!("落位失败: {e}"),
                    }),
                }
            } else if staged_single.is_file() {
                let tmp = local_dir.join(format!(".{uid}.jx3dat.tmp-cloud"));
                let result = fs::copy(&staged_single, &tmp)
                    .and_then(|_| fs::rename(&tmp, local_dir.join(format!("{uid}.jx3dat"))));
                match result {
                    Ok(()) => report.plugin_dirs.push(dir_name),
                    Err(e) => {
                        let _ = fs::remove_file(&tmp);
                        report.skipped.push(SkippedItem {
                            dir: dir_name,
                            reason: format!("落位失败: {e}"),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MemStorage(RefCell<HashMap<String, Vec<u8>>>);

    impl MemStorage {
        fn new() -> Self {
            Self(RefCell::new(HashMap::new()))
        }
    }

    impl CloudStorage for MemStorage {
        fn get(&self, path: &str) -> AppResult<Option<Vec<u8>>> {
            Ok(self.0.borrow().get(path).cloned())
        }

        fn put(&self, path: &str, bytes: &[u8]) -> AppResult<()> {
            self.0.borrow_mut().insert(path.to_string(), bytes.to_vec());
            Ok(())
        }

        fn check(&self) -> AppResult<()> {
            Ok(())
        }
    }

    static TEST_DIR_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "jx3-cloudsync-test-{}-{}-{}",
            std::process::id(),
            label,
            TEST_DIR_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn write_info(uid_dir: &Path, uid: &str, name: &str, server: &str, time: u64) {
        let lua = format!(
            r#"return {{region="电信区",id=1,time_str="x",time={time},server_origin="电信区",region_origin="{server}",uid="{uid}",name="{name}",server="{server}",version="1.0",branch="remake",lang="zhcn",edition="zhcn_hd"}}"#
        );
        let (encoded, _, _) = encoding_rs::GBK.encode(&lua);
        fs::create_dir_all(uid_dir).unwrap();
        fs::write(uid_dir.join("info.jx3dat"), encoded.into_owned()).unwrap();
    }

    /// 源设备游戏树：角色「源角色」(uid 111) 带键位 + my#data/lm#data config + SG#data
    fn build_source_tree() -> (PathBuf, PathBuf) {
        let root = temp_dir("src-game");
        let role = root.join("userdata/acc/电信区/梦江南/源角色");
        write_file(&role.join("keybind.ini"), "src-keys");
        write_file(&role.join("sub/extra.dat"), "extra");

        let interface = root.join("interface");
        let my = interface.join("my#data/111@zhcn_hd");
        write_info(&my, "111", "源角色", "梦江南", 100);
        write_file(&my.join("config/settings.db"), "src-my-config");
        let lm = interface.join("lm#data/111@zhcn_hd");
        write_info(&lm, "111", "源角色", "梦江南", 100);
        write_file(&lm.join("config/settings.db"), "src-lm-config");
        write_file(&interface.join("SG#data/111.jx3dat"), "src-sg");
        write_file(&interface.join("JX#DATA/CustomData.jx3dat"), "global");

        (root, role)
    }

    /// 目标设备游戏树：角色「新角色」(uid 999) 已用 my#data 登录过；
    /// SG#data 存在；lm#data 故意缺失（模拟没装枫影）
    fn build_target_tree() -> (PathBuf, PathBuf) {
        let root = temp_dir("tgt-game");
        let role = root.join("userdata/acc2/电信区/天鹅坪/新角色");
        write_file(&role.join("old-keybind.ini"), "old-keys");

        let interface = root.join("interface");
        let my = interface.join("my#data/999@zhcn_hd");
        write_info(&my, "999", "新角色", "天鹅坪", 100);
        write_file(&my.join("config/old.db"), "tgt-old-config");
        write_file(&interface.join("SG#data/888.jx3dat"), "other-role");

        (root, role)
    }

    #[test]
    fn upload_then_list_returns_entry_with_plugins() {
        let storage = MemStorage::new();
        let (src_root, src_role) = build_source_tree();

        let report = CloudSyncService::upload_role(&storage, &src_role).unwrap();
        assert_eq!(report.key, "梦江南/源角色");
        assert!(report.keybinding_size > 0);
        assert!(report.plugins_size > 0);
        let mut dirs = report.plugin_dirs.clone();
        dirs.sort();
        assert_eq!(dirs, vec!["SG#data", "lm#data", "my#data"]);
        assert!(report.skipped.is_empty(), "skipped: {:?}", report.skipped);

        let roles = CloudSyncService::list_roles(&storage).unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "源角色");
        assert_eq!(roles[0].server, "梦江南");
        assert!(roles[0].plugins_file.is_some());
        assert!(roles[0].uploaded_at > 0);

        let _ = fs::remove_dir_all(&src_root);
    }

    #[test]
    fn list_is_empty_before_any_upload() {
        let storage = MemStorage::new();
        assert!(CloudSyncService::list_roles(&storage).unwrap().is_empty());
    }

    #[test]
    fn reupload_same_role_keeps_single_manifest_entry() {
        let storage = MemStorage::new();
        let (src_root, src_role) = build_source_tree();

        CloudSyncService::upload_role(&storage, &src_role).unwrap();
        CloudSyncService::upload_role(&storage, &src_role).unwrap();

        assert_eq!(CloudSyncService::list_roles(&storage).unwrap().len(), 1);

        let _ = fs::remove_dir_all(&src_root);
    }

    #[test]
    fn download_applies_keybinding_and_plugins_to_target_role() {
        let storage = MemStorage::new();
        let (src_root, src_role) = build_source_tree();
        let (tgt_root, tgt_role) = build_target_tree();
        CloudSyncService::upload_role(&storage, &src_role).unwrap();

        let report =
            CloudSyncService::download_role(&storage, "梦江南/源角色", &tgt_role).unwrap();

        assert!(report.keybinding_applied);
        // 键位整体替换
        assert_eq!(
            fs::read_to_string(tgt_role.join("keybind.ini")).unwrap(),
            "src-keys"
        );
        assert_eq!(
            fs::read_to_string(tgt_role.join("sub/extra.dat")).unwrap(),
            "extra"
        );
        assert!(!tgt_role.join("old-keybind.ini").exists(), "旧键位应被整体替换");

        // my#data: 目标 UID(999) 的 config 被替换，info.jx3dat 不受影响
        let tgt_my = tgt_root.join("interface/my#data/999@zhcn_hd");
        assert_eq!(
            fs::read_to_string(tgt_my.join("config/settings.db")).unwrap(),
            "src-my-config"
        );
        assert!(!tgt_my.join("config/old.db").exists());
        assert!(tgt_my.join("info.jx3dat").is_file(), "目标身份文件必须保留");

        // SG#data: 按目标 UID 落盘
        assert_eq!(
            fs::read_to_string(tgt_root.join("interface/SG#data/999.jx3dat")).unwrap(),
            "src-sg"
        );

        // lm#data 本机未安装 → 跳过并说明
        assert!(report.plugin_dirs.contains(&"my#data".to_string()));
        assert!(report.plugin_dirs.contains(&"SG#data".to_string()));
        assert!(
            report.skipped.iter().any(|s| s.dir == "lm#data"),
            "skipped: {:?}",
            report.skipped
        );

        let _ = fs::remove_dir_all(&src_root);
        let _ = fs::remove_dir_all(&tgt_root);
    }

    #[test]
    fn download_skips_all_plugins_when_target_never_logged_in() {
        let storage = MemStorage::new();
        let (src_root, src_role) = build_source_tree();
        CloudSyncService::upload_role(&storage, &src_role).unwrap();

        // 目标树有 interface 但没有目标角色的任何 UID 数据
        let tgt_root = temp_dir("tgt-fresh");
        let tgt_role = tgt_root.join("userdata/acc/电信区/天鹅坪/纯新角色");
        write_file(&tgt_role.join("old.ini"), "old");
        fs::create_dir_all(tgt_root.join("interface/my#data")).unwrap();

        let report =
            CloudSyncService::download_role(&storage, "梦江南/源角色", &tgt_role).unwrap();

        assert!(report.keybinding_applied, "键位不依赖插件数据，应照常应用");
        assert!(report.plugin_dirs.is_empty());
        assert!(
            report.skipped.iter().any(|s| s.reason.contains("登录")),
            "应提示先用目标角色登录一次, got: {:?}",
            report.skipped
        );

        let _ = fs::remove_dir_all(&src_root);
        let _ = fs::remove_dir_all(&tgt_root);
    }

    #[test]
    fn download_errors_on_unknown_key() {
        let storage = MemStorage::new();
        let (src_root, src_role) = build_source_tree();
        CloudSyncService::upload_role(&storage, &src_role).unwrap();

        let tgt = temp_dir("tgt-any").join("userdata/a/b/c/角色");
        fs::create_dir_all(&tgt).unwrap();
        let err = CloudSyncService::download_role(&storage, "不存在/角色", &tgt)
            .unwrap_err()
            .to_string();
        assert!(err.contains("云端"), "应提示云端不存在该角色, got: {err}");

        let _ = fs::remove_dir_all(&src_root);
    }

    #[test]
    fn upload_without_plugin_data_still_uploads_keybinding() {
        let storage = MemStorage::new();
        // 只有 userdata 键位、interface 里查不到该角色 UID
        let root = temp_dir("no-plugin");
        let role = root.join("userdata/acc/电信区/梦江南/无插件角色");
        write_file(&role.join("keybind.ini"), "keys");
        fs::create_dir_all(root.join("interface/my#data")).unwrap();

        let report = CloudSyncService::upload_role(&storage, &role).unwrap();
        assert!(report.keybinding_size > 0);
        assert_eq!(report.plugins_size, 0);
        assert!(report.plugin_dirs.is_empty());
        assert!(!report.skipped.is_empty(), "应说明插件部分为何缺席");

        let roles = CloudSyncService::list_roles(&storage).unwrap();
        assert!(roles[0].plugins_file.is_none());

        let _ = fs::remove_dir_all(&root);
    }
}
