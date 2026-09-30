//! Copies the WebView2 loader next to the demo executable, and embeds the app
//! icon + version resource into it.
//!
//! With the GNU toolchain `webview2-com-sys` links `WebView2Loader.dll`
//! dynamically (the static loader is MSVC-only), so the DLL must sit beside the
//! exe at runtime. This makes `cargo run -p orbit-demo` work out of the box.
//! The icon/version resource (via `winres-embed`) makes the process look like an app
//! in Explorer, the taskbar and the file properties dialog.

fn main() {
    copy_webview2_loader();
    winres_embed::embed(winres_embed::Resources {
        icon: std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icon.ico"),
        version: winres_embed::Version::Fixed(env!("CARGO_PKG_VERSION").to_string()),
        company: "zz6zz666".to_string(),
        copyright: "MIT License".to_string(),
        comments: "https://github.com/zz6zz666/rust-webui-kit".to_string(),
        ..winres_embed::Resources::for_package(env!("CARGO_PKG_NAME"), "Orbit")
    });
}

fn copy_webview2_loader() {
    let dll = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join("WebView2Loader.dll");
    println!("cargo:rerun-if-changed={}", dll.display());
    if !dll.exists() {
        return;
    }
    // OUT_DIR is <target>/<profile>/build/<pkg>-<hash>/out; the exe lives three
    // levels up.
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    if let Some(profile_dir) = out_dir.ancestors().nth(3) {
        let _ = std::fs::copy(&dll, profile_dir.join("WebView2Loader.dll"));
    }
}
