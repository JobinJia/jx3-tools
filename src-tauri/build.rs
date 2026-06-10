fn main() {
    // 仅 Windows 目标：延迟加载 interception.dll。
    // exe 默认在启动时就解析所有导入的 DLL，缺失 interception.dll 会导致
    // STATUS_DLL_NOT_FOUND、进程在 main 之前直接崩溃。延迟加载把符号解析推迟到
    // 首次调用，配合随包分发的 DLL（resources/interception）保证启动绝不崩。
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/DELAYLOAD:interception.dll");
        println!("cargo:rustc-link-lib=dylib=delayimp");
    }

    // 嵌入自定义 Windows manifest 以请求管理员权限（详见 jx3-tools.manifest）。
    // 非 Windows 目标会忽略 windows_attributes，因此 macOS 构建不受影响。
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("jx3-tools.manifest"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attributes).expect("failed to run tauri build script");
}
