use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    id: u64,
    name: String,
    is_dir: bool,
    selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<FileEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopyParams {
    pub source_path: String,
    pub target_path: String,
}

pub struct KeyboardService;

/// 角色目录所在的树深度（userdata/<账号>/<区服>/<服务器>/<角色>）
const ROLE_DEPTH: usize = 4;

impl KeyboardService {
    /// List directory contents recursively for keyboard config selection
    pub fn list_directory_contents(path: &str) -> AppResult<Vec<FileEntry>> {
        let path = Path::new(path);
        if !path.is_dir() {
            return Err(AppError::Keyboard("提供的路径不是一个目录".into()));
        }
        Self::read_directory(path, 1)
    }

    /// Copy keyboard config from source to target directory
    pub fn copy_source_to_target(params: &CopyParams) -> AppResult<bool> {
        let source = Self::canonicalize_path(&params.source_path)?;
        let target = PathBuf::from(&params.target_path);

        // Validate paths are not the same
        if let Ok(canonical_target) = target.canonicalize() {
            if source == canonical_target {
                return Err(AppError::Keyboard("源路径和目标路径不能相同".into()));
            }
        }

        // Validate source path doesn't contain path traversal
        if params.source_path.contains("..") || params.target_path.contains("..") {
            return Err(AppError::Keyboard("路径不能包含 '..'".into()));
        }

        if !source.is_dir() {
            return Err(AppError::Keyboard(format!(
                "源路径不是目录: {}",
                source.display()
            )));
        }

        Self::swap_replace_dir(&source, &target)?;

        log::info!(
            "键位复制完成: {} -> {}",
            source.display(),
            target.display()
        );

        Ok(true)
    }

    /// 安全交换式复制：先把源完整复制到同级临时目录，成功后再与旧目标交换。
    /// 任何一步失败，目标原有内容都保持完好（不先删后拷）。
    /// 键位复制与插件配置同步共用此语义。
    pub(crate) fn swap_replace_dir(source: &Path, target: &Path) -> AppResult<()> {
        let parent = target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| AppError::Keyboard(format!("目标路径无效: {}", target.display())))?;
        let target_name = target
            .file_name()
            .ok_or_else(|| AppError::Keyboard(format!("目标路径无效: {}", target.display())))?
            .to_string_lossy()
            .to_string();
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Keyboard(format!("无法创建目标目录 {}: {}", parent.display(), e))
        })?;

        // 以 . 开头：read_directory 会忽略隐藏目录，残留也不会污染树
        let tmp = parent.join(format!(".{target_name}.tmp-copy"));
        let bak = parent.join(format!(".{target_name}.bak-copy"));
        let _ = fs::remove_dir_all(&tmp);
        let _ = fs::remove_dir_all(&bak);

        if let Err(e) = Self::copy_dir_all(source, &tmp) {
            let _ = fs::remove_dir_all(&tmp);
            return Err(AppError::Keyboard(format!("复制失败（目标未受影响）: {e}")));
        }

        // 交换：旧目标先挪到备份位，再把新内容就位
        if target.exists() {
            fs::rename(target, &bak).map_err(|e| {
                let _ = fs::remove_dir_all(&tmp);
                AppError::Keyboard(format!(
                    "无法移开旧的目标目录（可能被游戏占用，请关闭游戏后重试）: {e}"
                ))
            })?;
        }
        if let Err(e) = fs::rename(&tmp, target) {
            // 就位失败：恢复旧目标
            if bak.exists() {
                let _ = fs::rename(&bak, target);
            }
            let _ = fs::remove_dir_all(&tmp);
            return Err(AppError::Keyboard(format!("写入目标目录失败（已恢复原内容）: {e}")));
        }
        let _ = fs::remove_dir_all(&bak);
        Ok(())
    }

    /// Canonicalize path and handle errors
    fn canonicalize_path(path: &str) -> AppResult<PathBuf> {
        Path::new(path)
            .canonicalize()
            .map_err(|e| AppError::Keyboard(format!("无法解析路径 {}: {}", path, e)))
    }

    fn read_directory(path: &Path, depth: usize) -> AppResult<Vec<FileEntry>> {
        let mut entries = vec![];

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            // file_type() 复用目录读取的结果，避免每个条目一次额外 stat
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                let dir_name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| AppError::Keyboard("无效的目录名称".into()))?;

                // Skip userpreferences and hidden directories (incl. our tmp/bak dirs)
                if dir_name == "userpreferences" || dir_name.starts_with('.') {
                    continue;
                }

                // 角色层（ROLE_DEPTH）即叶子：不再向下递归。
                // 否则角色目录内部的子目录会被当成 children，前端会把该角色渲染成
                // 不可选中的"文件夹"节点，同时白白读取整棵无用子树。
                let subdir = if depth < ROLE_DEPTH {
                    Self::read_directory(&entry.path(), depth + 1)?
                } else {
                    Vec::new()
                };

                // 账号/区服/服务器层的空目录（下面没有任何角色）直接跳过
                if depth < ROLE_DEPTH && subdir.is_empty() {
                    continue;
                }

                // 角色层标记为可选中的叶子（is_dir = false）
                let is_dir = depth != ROLE_DEPTH;

                let children = if subdir.is_empty() { None } else { Some(subdir) };

                entries.push(FileEntry {
                    id: Self::generate_id(&dir_name, &entry.path()),
                    name: dir_name,
                    is_dir,
                    selected: false,
                    children,
                });
            }
        }

        Ok(entries)
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> AppResult<()> {
        if !src.is_dir() {
            return Err(AppError::Keyboard(format!(
                "源路径不是目录: {}",
                src.display()
            )));
        }

        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let new_dst = dst.join(entry.file_name());

            // Skip symlinks for security (prevent symlink-based path traversal)
            if file_type.is_symlink() {
                log::warn!("跳过符号链接: {}", entry.path().display());
                continue;
            }

            if file_type.is_dir() {
                Self::copy_dir_all(&entry.path(), &new_dst)?;
            } else {
                fs::copy(entry.path(), &new_dst)?;
            }
        }

        Ok(())
    }

    fn generate_id(name: &str, path: &Path) -> u64 {
        let mut hasher = DefaultHasher::new();
        (name, path).hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_DIR_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "jx3-kb-test-{}-{}-{}",
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

    #[test]
    fn tree_stops_at_role_depth_and_marks_roles_selectable() {
        let root = temp_dir("tree");
        // <账号>/<区服>/<服务器>/<角色>/<角色内部子目录>
        fs::create_dir_all(root.join("acc1/zone/server/roleA/inner")).unwrap();
        write_file(&root.join("acc1/zone/server/roleA/config.ini"), "x");
        // 干扰项：散落文件、userpreferences、隐藏目录都应被忽略
        write_file(&root.join("acc1/somefile.txt"), "x");
        fs::create_dir_all(root.join("acc1/zone/server/userpreferences")).unwrap();
        fs::create_dir_all(root.join(".hidden/zone/server/role")).unwrap();
        // 没有任何角色的空账号应被跳过
        fs::create_dir_all(root.join("acc-empty/zone/server")).unwrap();

        let tree = KeyboardService::list_directory_contents(root.to_str().unwrap()).unwrap();

        assert_eq!(tree.len(), 1, "只应保留有角色的账号");
        let acc = &tree[0];
        assert_eq!(acc.name, "acc1");
        assert!(acc.is_dir);
        let role = &acc.children.as_ref().unwrap()[0].children.as_ref().unwrap()[0]
            .children
            .as_ref()
            .unwrap()[0];
        assert_eq!(role.name, "roleA");
        assert!(!role.is_dir, "角色层应标记为可选中叶子");
        assert!(role.children.is_none(), "角色层不应继续递归出子目录");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn copy_replaces_target_and_leaves_no_temp_dirs() {
        let root = temp_dir("copy");
        let source = root.join("source");
        let target = root.join("target");
        write_file(&source.join("keys.ini"), "new-keys");
        write_file(&source.join("sub/extra.ini"), "extra");
        write_file(&target.join("old.ini"), "old-keys");

        let ok = KeyboardService::copy_source_to_target(&CopyParams {
            source_path: source.to_string_lossy().to_string(),
            target_path: target.to_string_lossy().to_string(),
        })
        .unwrap();

        assert!(ok);
        assert_eq!(
            fs::read_to_string(target.join("keys.ini")).unwrap(),
            "new-keys"
        );
        assert_eq!(
            fs::read_to_string(target.join("sub/extra.ini")).unwrap(),
            "extra"
        );
        assert!(!target.join("old.ini").exists(), "旧内容应被整体替换");
        assert!(!root.join(".target.tmp-copy").exists(), "临时目录不应残留");
        assert!(!root.join(".target.bak-copy").exists(), "备份目录不应残留");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn copy_rejects_same_path_and_traversal() {
        let root = temp_dir("guard");
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();

        let same = KeyboardService::copy_source_to_target(&CopyParams {
            source_path: source.to_string_lossy().to_string(),
            target_path: source.to_string_lossy().to_string(),
        });
        assert!(same.is_err(), "源 == 目标应被拒绝");

        let traversal = KeyboardService::copy_source_to_target(&CopyParams {
            source_path: source.to_string_lossy().to_string(),
            target_path: root.join("a/../b").to_string_lossy().to_string(),
        });
        assert!(traversal.is_err(), "包含 .. 的路径应被拒绝");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn copy_failure_preserves_existing_target() {
        let root = temp_dir("preserve");
        let target = root.join("target");
        write_file(&target.join("old.ini"), "old-keys");

        let missing = root.join("missing-source");
        let result = KeyboardService::copy_source_to_target(&CopyParams {
            source_path: missing.to_string_lossy().to_string(),
            target_path: target.to_string_lossy().to_string(),
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(target.join("old.ini")).unwrap(),
            "old-keys",
            "复制失败时目标原有键位必须保持完好"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
