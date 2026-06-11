//! 目录 ↔ zip 字节流。云同步把角色键位目录/插件 config 目录打成 zip 上传，
//! 下载后解包到临时目录再交换就位（复用 swap_replace_dir 语义，失败不伤本地）。

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::error::{AppError, AppResult};

/// 把目录内容（不含目录本身这一层）递归打包为 zip 字节流；符号链接跳过
pub fn pack_dir(dir: &Path) -> AppResult<Vec<u8>> {
    if !dir.is_dir() {
        return Err(AppError::Cloud(format!("不是目录，无法打包: {}", dir.display())));
    }
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    add_dir_recursive(&mut writer, dir, Path::new(""), options)?;
    let cursor = writer
        .finish()
        .map_err(|e| AppError::Cloud(format!("zip 打包失败: {e}")))?;
    Ok(cursor.into_inner())
}

fn add_dir_recursive(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    src: &Path,
    rel: &Path,
    options: SimpleFileOptions,
) -> AppResult<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        // 与 copy_dir_all 同口径：跳过符号链接，防 symlink 路径穿越
        if file_type.is_symlink() {
            log::warn!("打包时跳过符号链接: {}", entry.path().display());
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AppError::Cloud("文件名不是有效 UTF-8".into()))?;
        let rel_path = if rel.as_os_str().is_empty() {
            PathBuf::from(&name)
        } else {
            rel.join(&name)
        };
        // zip 条目统一用 / 分隔，保证 Windows 打包、任意平台解包一致
        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
        if file_type.is_dir() {
            zip.add_directory(format!("{rel_str}/"), options)
                .map_err(|e| AppError::Cloud(format!("zip 写入目录失败: {e}")))?;
            add_dir_recursive(zip, &entry.path(), &rel_path, options)?;
        } else {
            zip.start_file(&rel_str, options)
                .map_err(|e| AppError::Cloud(format!("zip 写入文件失败: {e}")))?;
            zip.write_all(&fs::read(entry.path())?)
                .map_err(|e| AppError::Cloud(format!("zip 写入内容失败: {e}")))?;
        }
    }
    Ok(())
}

/// 把 zip 字节流解包到目标目录（zip crate 自带 zip-slip 路径净化）
pub fn unpack_to_dir(bytes: &[u8], dst: &Path) -> AppResult<()> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Cloud(format!("zip 解析失败（文件可能损坏）: {e}")))?;
    fs::create_dir_all(dst)?;
    archive
        .extract(dst)
        .map_err(|e| AppError::Cloud(format!("zip 解包失败: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_DIR_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "jx3-pack-test-{}-{}-{}",
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
    fn roundtrip_preserves_tree_and_chinese_names() {
        let root = temp_dir("roundtrip");
        let src = root.join("源目录");
        write_file(&src.join("键位.ini"), "key-bindings");
        write_file(&src.join("子目录/嵌套/设置.jx3dat"), "nested-data");
        write_file(&src.join("empty.txt"), "");
        fs::create_dir_all(src.join("空目录")).unwrap();

        let bytes = pack_dir(&src).unwrap();
        assert!(!bytes.is_empty());

        let dst = root.join("解包");
        unpack_to_dir(&bytes, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("键位.ini")).unwrap(), "key-bindings");
        assert_eq!(
            fs::read_to_string(dst.join("子目录/嵌套/设置.jx3dat")).unwrap(),
            "nested-data"
        );
        assert_eq!(fs::read_to_string(dst.join("empty.txt")).unwrap(), "");
        assert!(dst.join("空目录").is_dir(), "空目录也应保留");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn pack_rejects_non_directory() {
        let root = temp_dir("nondir");
        let file = root.join("a.txt");
        write_file(&file, "x");
        assert!(pack_dir(&file).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unpack_rejects_garbage_bytes() {
        let root = temp_dir("garbage");
        assert!(unpack_to_dir(b"not a zip", &root.join("out")).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn pack_skips_symlinks() {
        let root = temp_dir("symlink");
        let src = root.join("src");
        write_file(&src.join("real.txt"), "real");
        std::os::unix::fs::symlink(src.join("real.txt"), src.join("link.txt")).unwrap();

        let bytes = pack_dir(&src).unwrap();
        let dst = root.join("dst");
        unpack_to_dir(&bytes, &dst).unwrap();

        assert!(dst.join("real.txt").is_file());
        assert!(!dst.join("link.txt").exists(), "符号链接应被跳过");

        let _ = fs::remove_dir_all(&root);
    }
}
