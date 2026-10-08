//! For a Windows target, the executable's resources: the icon Windows shows
//! in Explorer, on the taskbar and on shortcuts, and the version block the
//! file's Properties show. Both are compiled with windres from a resource
//! script written here, and linked into the binary. Other targets have
//! nothing to do.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest.join("../../remote/windows/glass-evo.ico");
    let icon = icon.canonicalize().unwrap_or(icon);
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-changed=build.rs");

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut parts: Vec<u32> = version.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    parts.resize(4, 0);
    let numeric = format!("{},{},{},{}", parts[0], parts[1], parts[2], parts[3]);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let script = out.join("glass-evo.rc");
    fs::write(
        &script,
        format!(
            r#"1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "FileDescription", "Glass Remote with the Glass interface: a Volumio player's meters and face on this screen"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "glass-evo"
      VALUE "OriginalFilename", "glass-evo.exe"
      VALUE "ProductName", "glass-evo"
      VALUE "ProductVersion", "{version}"
      VALUE "LegalCopyright", "MIT licence, https://github.com/foonerd/glass-evo"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
            icon = icon.display().to_string().replace('\\', "\\\\"),
        ),
    )
    .expect("write the resource script");

    let object = out.join("glass_evo_resources.o");
    let windres = ["x86_64-w64-mingw32-windres", "windres"]
        .into_iter()
        .find(|name| Command::new(name).arg("--version").output().is_ok())
        .expect("windres from MinGW-w64 is needed to build the Windows resources");
    let status = Command::new(windres)
        .arg("-O")
        .arg("coff")
        .arg("-i")
        .arg(&script)
        .arg("-o")
        .arg(&object)
        .status()
        .expect("run windres");
    assert!(status.success(), "windres failed on {}", script.display());
    println!("cargo:rustc-link-arg-bins={}", object.display());
}
