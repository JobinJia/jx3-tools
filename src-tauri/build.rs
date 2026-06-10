fn main() {
    // 嵌入自定义 Windows manifest 以请求管理员权限（详见 jx3-tools.manifest）。
    // 非 Windows 目标会忽略 windows_attributes，因此 macOS 构建不受影响。
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("jx3-tools.manifest"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attributes).expect("failed to run tauri build script");
}
