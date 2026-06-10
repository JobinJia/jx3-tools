//! 插件配置同步：把 interface 下按角色 UID 存储的插件数据从源角色同步到目标角色。
//!
//! 数据目录有三种形态（均以 `#data` 结尾，不区分大小写）：
//! - 框架式（my#data、lm#data）：`<uid>@<edition>/` 目录，内含 info.jx3dat（GBK 编码的
//!   角色身份）与 config/（插件设置）。同步 = 交换式复制 config 目录，绝不触碰 info.jx3dat。
//! - 单文件式（SG#data）：`<uid>.jx3dat` 文件，整文件复制为目标 UID 文件名。
//! - 全局式（JX#DATA）：无按 UID 的条目，所有账号天然共享，直接忽略。
//!
//! 角色 UID 无法从角色名推导，只能反查：扫描框架式目录里每个 UID 目录的 info.jx3dat，
//! 按（角色名, 服务器）匹配；同名同服出现多个 UID（改名残留）时取 time 最大者。

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::services::keyboard::{CopyParams, KeyboardService};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedItem {
    pub dir: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSyncReport {
    pub synced: Vec<String>,
    pub skipped: Vec<SkippedItem>,
}

/// 从 info.jx3dat 解析出的角色身份
#[derive(Debug, Clone, PartialEq)]
struct RoleInfo {
    uid: String,
    name: String,
    server: String,
    time: u64,
}

/// 单个数据目录的同步结果
enum Outcome {
    Synced,
    Skipped(String),
    /// 全局式目录（无按 UID 的条目），天然共享，无需同步也不进报告
    Ignored,
}

pub struct PluginDataService;

impl PluginDataService {
    /// 同步插件配置：params 为 userdata 下源/目标角色目录的绝对路径
    /// （userdata/<账号>/<区服>/<服务器>/<角色>，与键位复制同一套参数）
    pub fn sync_plugin_config(params: &CopyParams) -> AppResult<PluginSyncReport> {
        if params.source_path.contains("..") || params.target_path.contains("..") {
            return Err(AppError::Plugin("路径不能包含 '..'".into()));
        }
        let source = Path::new(&params.source_path);
        let target = Path::new(&params.target_path);
        if source == target {
            return Err(AppError::Plugin("源角色和目标角色不能相同".into()));
        }

        let (src_name, src_server) = Self::role_identity(source)?;
        let (tgt_name, tgt_server) = Self::role_identity(target)?;
        let interface = Self::locate_interface_dir(source)?;
        let data_dirs = Self::collect_data_dirs(&interface)?;
        if data_dirs.is_empty() {
            return Err(AppError::Plugin(format!(
                "interface 下未发现插件数据目录（*#data）: {}",
                interface.display()
            )));
        }

        let src_uid = Self::resolve_uid(&data_dirs, &src_name, &src_server).ok_or_else(|| {
            AppError::Plugin(format!(
                "在插件数据中未找到源角色 {src_name}（{src_server}），请确认该角色已配置过插件"
            ))
        })?;
        let tgt_uid = Self::resolve_uid(&data_dirs, &tgt_name, &tgt_server).ok_or_else(|| {
            AppError::Plugin(format!(
                "在插件数据中未找到目标角色 {tgt_name}（{tgt_server}），请先用该角色登录一次游戏"
            ))
        })?;
        if src_uid == tgt_uid {
            return Err(AppError::Plugin("源角色和目标角色相同，无需同步".into()));
        }

        let mut report = PluginSyncReport {
            synced: vec![],
            skipped: vec![],
        };
        for dir in &data_dirs {
            let dir_name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            match Self::sync_one_dir(dir, &src_uid, &tgt_uid) {
                Outcome::Synced => report.synced.push(dir_name),
                Outcome::Skipped(reason) => report.skipped.push(SkippedItem {
                    dir: dir_name,
                    reason,
                }),
                Outcome::Ignored => {}
            }
        }

        log::info!(
            "插件配置同步完成: {src_name}({src_uid}) -> {tgt_name}({tgt_uid}), 已同步 {:?}, 跳过 {} 项",
            report.synced,
            report.skipped.len()
        );
        Ok(report)
    }

    /// 从角色路径取（角色名, 服务器名）：userdata/<账号>/<区服>/<服务器>/<角色>
    fn role_identity(role_path: &Path) -> AppResult<(String, String)> {
        let name = role_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| AppError::Plugin(format!("角色路径无效: {}", role_path.display())))?;
        let server = role_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| {
                AppError::Plugin(format!("无法从路径解析服务器名: {}", role_path.display()))
            })?;
        Ok((name, server))
    }

    /// 从角色路径向上找到 userdata 的父目录（游戏根），返回其中的 interface 目录
    fn locate_interface_dir(role_path: &Path) -> AppResult<PathBuf> {
        let mut current = role_path;
        while let Some(parent) = current.parent() {
            let is_userdata = current
                .file_name()
                .map(|n| n.to_string_lossy().eq_ignore_ascii_case("userdata"))
                .unwrap_or(false);
            if is_userdata {
                let interface = parent.join("interface");
                if interface.is_dir() {
                    return Ok(interface);
                }
                return Err(AppError::Plugin(format!(
                    "游戏目录下没有 interface 目录: {}",
                    parent.display()
                )));
            }
            current = parent;
        }
        Err(AppError::Plugin(
            "所选角色路径不在 userdata 目录下，无法定位插件数据".into(),
        ))
    }

    /// interface 下所有 *#data 数据目录（不区分大小写）
    fn collect_data_dirs(interface: &Path) -> AppResult<Vec<PathBuf>> {
        let mut dirs = vec![];
        for entry in fs::read_dir(interface)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.ends_with("#data") {
                    dirs.push(entry.path());
                }
            }
        }
        dirs.sort();
        Ok(dirs)
    }

    /// 按（角色名, 服务器）反查 UID；同名同服多个 UID（改名残留）取 time 最大者
    fn resolve_uid(data_dirs: &[PathBuf], name: &str, server: &str) -> Option<String> {
        let mut best: Option<(u64, String)> = None;
        for dir in data_dirs {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let entry_name = entry.file_name().to_string_lossy().to_string();
                if !Self::is_uid_dir_name(&entry_name)
                    || !entry.file_type().is_ok_and(|t| t.is_dir())
                {
                    continue;
                }
                let Ok(bytes) = fs::read(entry.path().join("info.jx3dat")) else {
                    continue;
                };
                let Some(info) = Self::parse_info(&bytes) else {
                    continue;
                };
                if info.name == name
                    && info.server == server
                    && best.as_ref().map_or(true, |(t, _)| info.time > *t)
                {
                    best = Some((info.time, info.uid));
                }
            }
        }
        best.map(|(_, uid)| uid)
    }

    /// 同步单个数据目录，按形态分派
    fn sync_one_dir(dir: &Path, src_uid: &str, tgt_uid: &str) -> Outcome {
        let (framework_style, single_file_style) = Self::dir_style(dir);
        if framework_style {
            let Some(src_entry) = Self::find_uid_entry(dir, src_uid) else {
                return Outcome::Skipped("源角色在该插件下无数据".into());
            };
            let Some(tgt_entry) = Self::find_uid_entry(dir, tgt_uid) else {
                return Outcome::Skipped(
                    "目标角色在该插件下无数据（请先用目标角色登录一次游戏）".into(),
                );
            };
            let src_config = src_entry.join("config");
            if !src_config.is_dir() {
                return Outcome::Skipped("源角色在该插件下没有 config 配置".into());
            }
            // 只同步 config（插件设置），不碰 info.jx3dat（角色身份）和 userdata（聊天记录等数据）
            match KeyboardService::swap_replace_dir(&src_config, &tgt_entry.join("config")) {
                Ok(()) => Outcome::Synced,
                Err(e) => Outcome::Skipped(format!("同步失败: {e}")),
            }
        } else if single_file_style {
            let src_file = dir.join(format!("{src_uid}.jx3dat"));
            if !src_file.is_file() {
                return Outcome::Skipped("源角色在该插件下无数据".into());
            }
            // 先写临时文件再 rename 就位，避免写一半的目标文件
            let tmp = dir.join(format!(".{tgt_uid}.jx3dat.tmp-copy"));
            let result = fs::copy(&src_file, &tmp)
                .and_then(|_| fs::rename(&tmp, dir.join(format!("{tgt_uid}.jx3dat"))));
            match result {
                Ok(()) => Outcome::Synced,
                Err(e) => {
                    let _ = fs::remove_file(&tmp);
                    Outcome::Skipped(format!("同步失败: {e}"))
                }
            }
        } else {
            Outcome::Ignored
        }
    }

    /// 判断数据目录形态：(有 `<uid>@<edition>` 子目录, 有 `<uid>.jx3dat` 文件)
    fn dir_style(dir: &Path) -> (bool, bool) {
        let mut has_uid_dirs = false;
        let mut has_uid_files = false;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if file_type.is_dir() && Self::is_uid_dir_name(&name) {
                    has_uid_dirs = true;
                } else if file_type.is_file() && Self::is_uid_file_name(&name) {
                    has_uid_files = true;
                }
            }
        }
        (has_uid_dirs, has_uid_files)
    }

    fn is_uid_dir_name(name: &str) -> bool {
        match name.split_once('@') {
            Some((uid, _)) => !uid.is_empty() && uid.chars().all(|c| c.is_ascii_digit()),
            None => false,
        }
    }

    fn is_uid_file_name(name: &str) -> bool {
        match name.strip_suffix(".jx3dat") {
            Some(uid) => !uid.is_empty() && uid.chars().all(|c| c.is_ascii_digit()),
            None => false,
        }
    }

    /// 在框架式目录里找 `<uid>@<edition>` 子目录
    fn find_uid_entry(dir: &Path, uid: &str) -> Option<PathBuf> {
        let prefix = format!("{uid}@");
        fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let matched = name.starts_with(&prefix)
                && entry.file_type().is_ok_and(|t| t.is_dir());
            matched.then(|| entry.path())
        })
    }

    /// 解析 GBK 编码的 info.jx3dat（单行 Lua table），提取 uid / name / server / time。
    /// 不做完整 Lua 解析，按 `key="value"` 子串提取；键名前必须是 `{` 或 `,`，
    /// 避免误匹配 server_origin / time_str 这类带相同前缀的键。
    fn parse_info(bytes: &[u8]) -> Option<RoleInfo> {
        let (text, _, _) = encoding_rs::GBK.decode(bytes);
        let s = text.as_ref();
        Some(RoleInfo {
            uid: extract_quoted(s, "uid")?,
            name: extract_quoted(s, "name")?,
            server: extract_quoted(s, "server")?,
            time: extract_number(s, "time").unwrap_or(0),
        })
    }
}

/// 提取 `key="value"` 形式的值；键名前必须是 `{` 或 `,`
fn extract_quoted(s: &str, key: &str) -> Option<String> {
    let pat = format!("{key}=\"");
    let mut from = 0;
    while let Some(idx) = s[from..].find(&pat) {
        let abs = from + idx;
        if abs == 0 || matches!(s.as_bytes()[abs - 1], b'{' | b',') {
            let value_start = abs + pat.len();
            let value_end = s[value_start..].find('"')? + value_start;
            return Some(s[value_start..value_end].to_string());
        }
        from = abs + pat.len();
    }
    None
}

/// 提取 `key=123` 形式的数字值；键名前必须是 `{` 或 `,`
fn extract_number(s: &str, key: &str) -> Option<u64> {
    let pat = format!("{key}=");
    let mut from = 0;
    while let Some(idx) = s[from..].find(&pat) {
        let abs = from + idx;
        if abs == 0 || matches!(s.as_bytes()[abs - 1], b'{' | b',') {
            let digits: String = s[abs + pat.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                return digits.parse().ok();
            }
        }
        from = abs + pat.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_DIR_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "jx3-plugin-test-{}-{}-{}",
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

    fn info_lua(uid: &str, name: &str, server: &str, time: u64) -> Vec<u8> {
        // 与真实文件同构：time_str 在 time 之前、server_origin/region_origin 干扰键齐全
        let lua = format!(
            r#"return {{region="电信区",id=1337077,time_str="20260424032824",time={time},server_origin="电信区",region_origin="{server}",uid="{uid}",name="{name}",server="{server}",version="1.5.0.9724",branch="remake",lang="zhcn",edition="zhcn_hd"}}"#
        );
        let (encoded, _, _) = encoding_rs::GBK.encode(&lua);
        encoded.into_owned()
    }

    fn write_info(uid_dir: &Path, uid: &str, name: &str, server: &str, time: u64) {
        fs::create_dir_all(uid_dir).unwrap();
        fs::write(uid_dir.join("info.jx3dat"), info_lua(uid, name, server, time)).unwrap();
    }

    /// 搭一个最小游戏目录：userdata 两个角色 + interface 三种形态的数据目录
    /// 返回 (root, 源角色路径, 目标角色路径)
    fn build_game_tree() -> (PathBuf, PathBuf, PathBuf) {
        let root = temp_dir("game");
        let source_role = root.join("userdata/account1/电信区/梦江南/源角色");
        let target_role = root.join("userdata/account1/电信区/梦江南/目标角色");
        fs::create_dir_all(&source_role).unwrap();
        fs::create_dir_all(&target_role).unwrap();

        let interface = root.join("interface");
        // 框架式：源 111 / 目标 222
        let src_uid = interface.join("my#data/111@zhcn_hd");
        write_info(&src_uid, "111", "源角色", "梦江南", 100);
        write_file(&src_uid.join("config/settings.db"), "new-settings");
        let tgt_uid = interface.join("my#data/222@zhcn_hd");
        write_info(&tgt_uid, "222", "目标角色", "梦江南", 100);
        write_file(&tgt_uid.join("config/old.db"), "old-settings");
        // 框架式目录里的非角色条目（全局/服务器/缓存）不应干扰
        write_file(
            &interface.join("my#data/!all-users@zhcn_hd/config/settings.db"),
            "global",
        );
        fs::create_dir_all(interface.join("my#data/#cache")).unwrap();
        // 单文件式
        write_file(&interface.join("SG#data/111.jx3dat"), "sg-data");
        // 全局式：不应被同步、不应出现在报告里
        write_file(&interface.join("JX#DATA/CustomData.jx3dat"), "jx-global");

        (root, source_role, target_role)
    }

    fn sync(source: &Path, target: &Path) -> AppResult<PluginSyncReport> {
        PluginDataService::sync_plugin_config(&CopyParams {
            source_path: source.to_string_lossy().to_string(),
            target_path: target.to_string_lossy().to_string(),
        })
    }

    #[test]
    fn parse_info_decodes_gbk_identity() {
        let bytes = info_lua("432345564228904693", "落梅听风雪", "梦江南", 1776972504);
        let info = PluginDataService::parse_info(&bytes).unwrap();
        assert_eq!(info.uid, "432345564228904693");
        assert_eq!(info.name, "落梅听风雪");
        assert_eq!(info.server, "梦江南");
        assert_eq!(info.time, 1776972504);
    }

    #[test]
    fn parse_info_rejects_garbage() {
        assert!(PluginDataService::parse_info(b"not a lua table").is_none());
    }

    #[test]
    fn locate_interface_dir_finds_sibling_of_userdata() {
        let root = temp_dir("locate");
        let role = root.join("game/userdata/acc/电信区/梦江南/角色A");
        fs::create_dir_all(&role).unwrap();
        fs::create_dir_all(root.join("game/interface")).unwrap();

        let found = PluginDataService::locate_interface_dir(&role).unwrap();
        assert_eq!(found, root.join("game/interface"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn locate_interface_dir_errors_without_userdata_ancestor() {
        let root = temp_dir("locate-bad");
        let role = root.join("somewhere/else/角色A");
        fs::create_dir_all(&role).unwrap();

        let err = PluginDataService::locate_interface_dir(&role)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("userdata"),
            "应提示路径不在 userdata 下, got: {err}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_copies_config_and_single_file_data() {
        let (root, source_role, target_role) = build_game_tree();
        let interface = root.join("interface");
        let target_info_before =
            fs::read(interface.join("my#data/222@zhcn_hd/info.jx3dat")).unwrap();

        let report = sync(&source_role, &target_role).unwrap();

        let mut synced = report.synced.clone();
        synced.sort();
        assert_eq!(synced, vec!["SG#data", "my#data"]);
        assert!(report.skipped.is_empty(), "skipped: {:?}", report.skipped);

        // config 被整体替换
        let tgt_cfg = interface.join("my#data/222@zhcn_hd/config");
        assert_eq!(
            fs::read_to_string(tgt_cfg.join("settings.db")).unwrap(),
            "new-settings"
        );
        assert!(!tgt_cfg.join("old.db").exists(), "旧配置应被整体替换");
        // info.jx3dat 绝不能被覆盖
        assert_eq!(
            fs::read(interface.join("my#data/222@zhcn_hd/info.jx3dat")).unwrap(),
            target_info_before,
            "目标角色身份文件不能被改动"
        );
        // 单文件式按目标 UID 落盘
        assert_eq!(
            fs::read_to_string(interface.join("SG#data/222.jx3dat")).unwrap(),
            "sg-data"
        );
        // 全局目录不受影响
        assert_eq!(
            fs::read_to_string(interface.join("JX#DATA/CustomData.jx3dat")).unwrap(),
            "jx-global"
        );
        // 全局条目不受影响
        assert_eq!(
            fs::read_to_string(interface.join("my#data/!all-users@zhcn_hd/config/settings.db"))
                .unwrap(),
            "global"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_skips_dir_where_target_has_no_data() {
        let (root, source_role, target_role) = build_game_tree();
        // lm#data 只有源角色，没有目标角色
        let lm_src = root.join("interface/lm#data/111@zhcn_hd");
        write_info(&lm_src, "111", "源角色", "梦江南", 100);
        write_file(&lm_src.join("config/settings.db"), "lm-settings");

        let report = sync(&source_role, &target_role).unwrap();

        assert!(report.synced.contains(&"my#data".to_string()));
        assert!(
            report.skipped.iter().any(|s| s.dir == "lm#data"),
            "目标角色无 lm#data 数据时应跳过并说明原因, got: {:?}",
            report.skipped
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_skips_single_file_dir_without_source_file() {
        let (root, source_role, target_role) = build_game_tree();
        let _ = fs::remove_file(root.join("interface/SG#data/111.jx3dat"));
        // 留一个其他角色的文件，目录仍是单文件式
        write_file(&root.join("interface/SG#data/999.jx3dat"), "other");

        let report = sync(&source_role, &target_role).unwrap();

        assert!(report.synced.contains(&"my#data".to_string()));
        assert!(!report.synced.contains(&"SG#data".to_string()));
        assert!(report.skipped.iter().any(|s| s.dir == "SG#data"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_errors_when_source_role_unknown() {
        let (root, _, target_role) = build_game_tree();
        let unknown = root.join("userdata/account1/电信区/梦江南/无名氏");
        fs::create_dir_all(&unknown).unwrap();

        let err = sync(&unknown, &target_role).unwrap_err().to_string();
        assert!(
            err.contains("源角色") && err.contains("无名氏"),
            "源角色查不到 UID 应整体报错并指明角色, got: {err}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_errors_when_target_role_unknown() {
        let (root, source_role, _) = build_game_tree();
        let unknown = root.join("userdata/account1/电信区/梦江南/新角色");
        fs::create_dir_all(&unknown).unwrap();

        let err = sync(&source_role, &unknown).unwrap_err().to_string();
        assert!(
            err.contains("登录") && err.contains("新角色"),
            "目标角色查不到 UID 应提示先登录一次, got: {err}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_rejects_same_role() {
        let (root, source_role, _) = build_game_tree();
        let err = sync(&source_role, &source_role).unwrap_err().to_string();
        assert!(err.contains("相同"), "源 == 目标应被明确拒绝, got: {err}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn duplicate_identity_resolves_to_latest_time() {
        let (root, source_role, target_role) = build_game_tree();
        // 同名同服的旧 UID 残留（time 更小），不应被选中
        let stale = root.join("interface/my#data/333@zhcn_hd");
        write_info(&stale, "333", "源角色", "梦江南", 50);
        write_file(&stale.join("config/settings.db"), "stale-settings");

        let report = sync(&source_role, &target_role).unwrap();

        assert!(report.synced.contains(&"my#data".to_string()));
        assert_eq!(
            fs::read_to_string(root.join("interface/my#data/222@zhcn_hd/config/settings.db"))
                .unwrap(),
            "new-settings",
            "应使用 time 最大的 UID（111）作为源"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
