//! WebDAV 存储层。`CloudStorage` 是 sync 编排层依赖的抽象（单测用内存实现替代网络），
//! `WebDavStorage` 是 reqwest blocking 实现——只用到 GET / PUT / MKCOL / PROPFIND
//! 四个动词，云端列表不靠 PROPFIND 遍历而是读 manifest.json，因此无需解析 XML。

use std::time::Duration;

use url::Url;

use crate::error::{AppError, AppResult};

/// 云存储抽象：get 在文件不存在时返回 None；put 自动确保父目录存在
pub trait CloudStorage {
    fn get(&self, path: &str) -> AppResult<Option<Vec<u8>>>;
    fn put(&self, path: &str, bytes: &[u8]) -> AppResult<()>;
    /// 连通性 + 凭证检查
    fn check(&self) -> AppResult<()>;
}

/// 规范化服务器地址：去空白、要求 http(s)、补尾部 /
pub fn normalize_base_url(input: &str) -> AppResult<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Cloud("服务器地址不能为空".into()));
    }
    let url = Url::parse(trimmed)
        .map_err(|e| AppError::Cloud(format!("服务器地址无效: {e}")))?;
    if url.scheme() != "https" && url.scheme() != "http" {
        return Err(AppError::Cloud("服务器地址必须以 http:// 或 https:// 开头".into()));
    }
    if url.path().ends_with('/') {
        Ok(url)
    } else {
        let mut url = url;
        url.set_path(&format!("{}/", url.path()));
        Ok(url)
    }
}

/// 在 base 上拼接相对路径（按段 percent-encode，中文安全）；拒绝 `..`
pub fn join_path(base: &Url, rel: &str) -> AppResult<Url> {
    if rel.split('/').any(|seg| seg == "..") {
        return Err(AppError::Cloud(format!("路径不能包含 '..': {rel}")));
    }
    let mut url = base.clone();
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| AppError::Cloud("服务器地址无效（cannot-be-a-base）".into()))?;
        segments.pop_if_empty();
        for seg in rel.split('/').filter(|seg| !seg.is_empty()) {
            segments.push(seg);
        }
    }
    Ok(url)
}

/// 一个文件路径的所有祖先目录（由浅到深），用于逐级 MKCOL
pub fn ancestor_dirs(path: &str) -> Vec<String> {
    let segments: Vec<&str> = path.split('/').filter(|seg| !seg.is_empty()).collect();
    (1..segments.len())
        .map(|depth| segments[..depth].join("/"))
        .collect()
}

pub struct WebDavStorage {
    base: Url,
    username: String,
    password: String,
    client: reqwest::blocking::Client,
    /// 本实例已确保存在的目录，批量上传时避免对同一目录反复 MKCOL（坚果云免费版限频）
    created_dirs: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl WebDavStorage {
    pub fn new(server_url: &str, username: &str, password: &str) -> AppResult<Self> {
        let base = normalize_base_url(server_url)?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Cloud(format!("初始化 HTTP 客户端失败: {e}")))?;
        Ok(Self {
            base,
            username: username.to_string(),
            password: password.to_string(),
            client,
            created_dirs: std::sync::Mutex::new(std::collections::HashSet::new()),
        })
    }

    fn url_for(&self, path: &str) -> AppResult<Url> {
        join_path(&self.base, path)
    }

    fn send_err(e: reqwest::Error) -> AppError {
        AppError::Cloud(format!("网络请求失败（请检查服务器地址和网络）: {e}"))
    }

    fn status_err(status: u16) -> AppError {
        match status {
            401 => AppError::Cloud("账号或应用密码错误（401）".into()),
            403 => AppError::Cloud("没有权限访问该路径（403）".into()),
            507 => AppError::Cloud("网盘空间不足（507）".into()),
            s => AppError::Cloud(format!("服务器返回异常状态码: {s}")),
        }
    }

    /// 逐级创建文件路径的祖先目录；201 = 新建成功，405 = 已存在，均视为成功
    fn mkcol_ancestors(&self, path: &str) -> AppResult<()> {
        for dir in ancestor_dirs(path) {
            if self
                .created_dirs
                .lock()
                .is_ok_and(|dirs| dirs.contains(&dir))
            {
                continue;
            }
            let mut url = self.url_for(&dir)?;
            // MKCOL 集合地址按惯例带尾部 /，避免部分服务端 301 重定向
            if !url.path().ends_with('/') {
                url.set_path(&format!("{}/", url.path()));
            }
            let method = reqwest::Method::from_bytes(b"MKCOL")
                .map_err(|e| AppError::Cloud(format!("构造 MKCOL 请求失败: {e}")))?;
            let resp = self
                .client
                .request(method, url)
                .basic_auth(&self.username, Some(&self.password))
                .send()
                .map_err(Self::send_err)?;
            match resp.status().as_u16() {
                201 | 405 => {
                    if let Ok(mut dirs) = self.created_dirs.lock() {
                        dirs.insert(dir);
                    }
                }
                s => return Err(Self::status_err(s)),
            }
        }
        Ok(())
    }
}

impl CloudStorage for WebDavStorage {
    fn get(&self, path: &str) -> AppResult<Option<Vec<u8>>> {
        let resp = self
            .client
            .get(self.url_for(path)?)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .map_err(Self::send_err)?;
        match resp.status().as_u16() {
            200 => {
                let bytes = resp
                    .bytes()
                    .map_err(|e| AppError::Cloud(format!("读取响应内容失败: {e}")))?;
                Ok(Some(bytes.to_vec()))
            }
            404 => Ok(None),
            s => Err(Self::status_err(s)),
        }
    }

    fn put(&self, path: &str, bytes: &[u8]) -> AppResult<()> {
        self.mkcol_ancestors(path)?;
        let resp = self
            .client
            .put(self.url_for(path)?)
            .basic_auth(&self.username, Some(&self.password))
            .body(bytes.to_vec())
            .send()
            .map_err(Self::send_err)?;
        match resp.status().as_u16() {
            200 | 201 | 204 => Ok(()),
            s => Err(Self::status_err(s)),
        }
    }

    fn check(&self) -> AppResult<()> {
        let method = reqwest::Method::from_bytes(b"PROPFIND")
            .map_err(|e| AppError::Cloud(format!("构造 PROPFIND 请求失败: {e}")))?;
        let resp = self
            .client
            .request(method, self.base.clone())
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "0")
            .send()
            .map_err(Self::send_err)?;
        match resp.status().as_u16() {
            200 | 207 => Ok(()),
            404 => Err(AppError::Cloud("服务器地址路径不存在（404），请检查 WebDAV 地址".into())),
            s => Err(Self::status_err(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_trailing_slash_and_trims() {
        let url = normalize_base_url("  https://dav.jianguoyun.com/dav  ").unwrap();
        assert_eq!(url.as_str(), "https://dav.jianguoyun.com/dav/");
        // 已带 / 的不重复加
        let url = normalize_base_url("https://dav.jianguoyun.com/dav/").unwrap();
        assert_eq!(url.as_str(), "https://dav.jianguoyun.com/dav/");
    }

    #[test]
    fn normalize_rejects_non_http_and_garbage() {
        assert!(normalize_base_url("ftp://example.com/dav").is_err());
        assert!(normalize_base_url("not a url").is_err());
        assert!(normalize_base_url("").is_err());
    }

    #[test]
    fn join_percent_encodes_chinese_segments() {
        let base = normalize_base_url("https://dav.jianguoyun.com/dav").unwrap();
        let url = join_path(&base, "jx3-tools/roles/梦江南_落梅听风雪/keybinding.zip").unwrap();
        assert_eq!(
            url.as_str(),
            "https://dav.jianguoyun.com/dav/jx3-tools/roles/\
             %E6%A2%A6%E6%B1%9F%E5%8D%97_%E8%90%BD%E6%A2%85%E5%90%AC%E9%A3%8E%E9%9B%AA/keybinding.zip"
        );
    }

    #[test]
    fn join_rejects_parent_traversal() {
        let base = normalize_base_url("https://example.com/dav").unwrap();
        assert!(join_path(&base, "a/../b").is_err());
        assert!(join_path(&base, "../escape").is_err());
    }

    #[test]
    fn ancestor_dirs_lists_shallow_to_deep() {
        assert_eq!(
            ancestor_dirs("jx3-tools/roles/梦江南_角色/keybinding.zip"),
            vec![
                "jx3-tools".to_string(),
                "jx3-tools/roles".to_string(),
                "jx3-tools/roles/梦江南_角色".to_string(),
            ]
        );
        assert!(ancestor_dirs("top-level.json").is_empty());
    }
}
