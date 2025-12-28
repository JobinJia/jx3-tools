fn main() {
    // Windows: 嵌入清单文件以请求管理员权限
    #[cfg(target_os = "windows")]
    {
        let mut res = tauri_winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {}", e);
        }
    }

    tauri_build::build()
}
