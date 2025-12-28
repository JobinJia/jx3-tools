fn main() {
    // Windows: 设置清单文件以请求管理员权限
    // 通过 Tauri 的 windows 配置来嵌入清单
    #[cfg(target_os = "windows")]
    {
        // 设置清单文件路径，让链接器直接使用
        println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:app.manifest");
    }

    tauri_build::build()
}
