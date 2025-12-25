fn main() {
    // Embed Windows manifest for UAC elevation (requireAdministrator)
    // Only compile manifest when building the release binary, not during tests
    // We check TAURI_ENV which is set by Tauri during build, not during cargo test
    #[cfg(windows)]
    if std::env::var("TAURI_ENV").is_ok() || std::env::var("PROFILE").map(|p| p == "release").unwrap_or(false) {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {}", e);
        }
    }

    tauri_build::build()
}
