//! Copies the WebView2 loader next to the demo executable.
//!
//! With the GNU toolchain `webview2-com-sys` links `WebView2Loader.dll`
//! dynamically (the static loader is MSVC-only), so the DLL must sit beside the
//! exe at runtime. This makes `cargo run -p orbit-demo` work out of the box.

fn main() {
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
