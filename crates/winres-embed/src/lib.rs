//! Build-time embedding of a Windows icon (`ICON`) and `VERSIONINFO` resource,
//! for a crate's `build.rs`.
//!
//! The project targets the GNU toolchain, whose `windres` compiles the resource
//! and links it into the executable so Explorer, the taskbar and the file
//! properties dialog see an application icon and version. On other targets — or
//! under MSVC, which would need `rc.exe` instead — it is a no-op, so a
//! cross-toolchain `cargo check` or CI run stays green.
//!
//! ```no_run
//! // build.rs
//! winres_embed::embed(winres_embed::Resources {
//!     icon: std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icon.ico"),
//!     version: winres_embed::Version::Fixed(env!("CARGO_PKG_VERSION").to_string()),
//!     ..winres_embed::Resources::for_package(env!("CARGO_PKG_NAME"), "My App")
//! });
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the version string comes from.
pub enum Version {
    /// A literal `major.minor.patch.build`; missing parts are zero.
    Fixed(String),
    /// An environment variable, falling back to the given value when it is
    /// unset.
    Env { var: String, fallback: String },
}

/// Everything the generated resource needs. Build it with
/// [`Resources::for_package`] and override the fields you care about.
pub struct Resources {
    /// Path to the `.ico` to embed as the application icon.
    pub icon: PathBuf,
    /// Where the version comes from.
    pub version: Version,
    pub company: String,
    pub file_description: String,
    pub internal_name: String,
    pub original_filename: String,
    pub product_name: String,
    pub copyright: String,
    pub comments: String,
}

impl Resources {
    /// Defaults for `package` (used as the internal name and file name prefix)
    /// and `product` (the display name).
    pub fn for_package(package: &str, product: &str) -> Resources {
        Resources {
            icon: PathBuf::new(),
            version: Version::Fixed(String::new()),
            company: String::new(),
            file_description: product.to_string(),
            internal_name: package.to_string(),
            original_filename: format!("{package}.exe"),
            product_name: product.to_string(),
            copyright: String::new(),
            comments: String::new(),
        }
    }
}

/// Compiles and links the resource described by `res`. No-op off Windows/GNU.
pub fn embed(res: Resources) {
    println!("cargo:rerun-if-changed={}", res.icon.display());
    println!("cargo:rerun-if-changed=build.rs");
    if let Version::Env { var, .. } = &res.version {
        println!("cargo:rerun-if-env-changed={var}");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // The resource is compiled with the GNU `windres`; MSVC would need `rc.exe`.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu") {
        return;
    }

    let version = match &res.version {
        Version::Fixed(v) => v.clone(),
        Version::Env { var, fallback } => std::env::var(var).unwrap_or_else(|_| fallback.clone()),
    };
    let parts: Vec<u32> = version.split('.').map(|x| x.parse().unwrap_or(0)).collect();
    let get = |i: usize| parts.get(i).copied().unwrap_or(0);
    let file_ver = format!("{},{},{},{}", get(0), get(1), get(2), get(3));
    let str_ver = format!("{}.{}.{}.{}", get(0), get(1), get(2), get(3));
    let icon = res.icon.display().to_string().replace('\\', "\\\\");

    let rc = format!(
        r#"1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {fv}
PRODUCTVERSION {fv}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "080404b0"
    BEGIN
      VALUE "CompanyName", "{company}"
      VALUE "FileDescription", "{desc}"
      VALUE "FileVersion", "{sv}"
      VALUE "InternalName", "{internal}"
      VALUE "LegalCopyright", "{copyright}"
      VALUE "OriginalFilename", "{filename}"
      VALUE "ProductName", "{product}"
      VALUE "ProductVersion", "{sv}"
      VALUE "Comments", "{comments}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x804, 1200
  END
END
"#,
        icon = icon,
        fv = file_ver,
        sv = str_ver,
        company = res.company,
        desc = res.file_description,
        internal = res.internal_name,
        copyright = res.copyright,
        filename = res.original_filename,
        product = res.product_name,
        comments = res.comments,
    );

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let rc_path = out.join("resource.rc");
    if std::fs::write(&rc_path, rc).is_err() {
        return;
    }

    let Some(windres) = find_windres() else {
        println!("cargo:warning=windres not found; skipping the icon/version resource");
        return;
    };
    // windres shells out to gcc for preprocessing; put its sibling on PATH.
    let path = match windres.parent() {
        Some(dir) => {
            let old = std::env::var("PATH").unwrap_or_default();
            format!("{};{}", dir.display(), old)
        }
        None => std::env::var("PATH").unwrap_or_default(),
    };
    let obj = out.join("resource.res.o");
    let status = Command::new(&windres)
        .env("PATH", path)
        .args(["--codepage=65001", "-O", "coff", "-i"])
        .arg(&rc_path)
        .arg("-o")
        .arg(&obj)
        .status();
    match status {
        Ok(s) if s.success() => println!("cargo:rustc-link-arg={}", obj.display()),
        _ => println!("cargo:warning=windres failed; skipping the icon/version resource"),
    }
}

fn find_windres() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("WINDRES") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Some(p);
        }
    }
    for cand in [
        r"C:\msys64\ucrt64\bin\windres.exe",
        r"C:\msys64\clang64\bin\windres.exe",
        r"C:\msys64\mingw64\bin\windres.exe",
    ] {
        let p = Path::new(cand);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(';') {
            let p = Path::new(dir).join("windres.exe");
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}
