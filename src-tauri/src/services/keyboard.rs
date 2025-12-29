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

        if !source.exists() {
            return Err(AppError::Keyboard(format!(
                "源路径不存在: {}",
                source.display()
            )));
        }

        if !source.is_dir() {
            return Err(AppError::Keyboard(format!(
                "源路径不是目录: {}",
                source.display()
            )));
        }

        // Check if source is a symlink (security risk)
        if source.is_symlink() {
            return Err(AppError::Keyboard("不支持复制符号链接目录".into()));
        }

        // Always remove existing target directory first to ensure clean copy
        if target.exists() {
            log::debug!("清空目标目录: {}", target.display());
            fs::remove_dir_all(&target).map_err(|e| {
                AppError::Keyboard(format!(
                    "无法清空目标目录，请手动删除后重试: {}\n错误: {}",
                    target.display(),
                    e
                ))
            })?;
        }

        // Create target directory
        fs::create_dir_all(&target).map_err(|e| {
            AppError::Keyboard(format!("无法创建目标目录 {}: {}", target.display(), e))
        })?;

        // Copy all contents
        Self::copy_dir_all(&source, &target)?;

        log::info!(
            "键位复制完成: {} -> {}",
            source.display(),
            target.display()
        );

        Ok(true)
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
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                let dir_name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| AppError::Keyboard("无效的目录名称".into()))?;

                // Skip userpreferences directory
                if dir_name == "userpreferences" {
                    continue;
                }

                let subdir = Self::read_directory(&entry.path(), depth + 1)?;

                // Skip empty directories at depths 1-3
                if (depth <= 3) && subdir.is_empty() {
                    continue;
                }

                // At depth 4, mark as non-role (is_dir = false)
                let is_dir = depth != 4;

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
