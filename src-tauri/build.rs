fn main() {
    // Embed Windows manifest for UAC elevation (requireAdministrator)
    // Only compile manifest when building the app, not during tests
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TEST").is_err() {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        // Don't set version info - Tauri handles this to avoid duplicate resources
        res.set("FileVersion", "");
        res.set("ProductVersion", "");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {}", e);
        }
    }

    tauri_build::build()
}
