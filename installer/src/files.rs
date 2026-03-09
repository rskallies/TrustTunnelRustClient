//! File installation — copies bundled binaries to Program Files.
//!
//! The installer binary is expected to sit next to the files it installs:
//!   trusttunnel-installer.exe
//!   trusttunnel-service.exe
//!   trusttunnel.exe
//!   vpn_easy.dll
//!   wintun.dll

use std::path::{Path, PathBuf};
use crate::error::InstallerError;

const APP_DIR: &str = "TrustTunnel";

const BUNDLED_FILES: &[&str] = &[
    "trusttunnel-service.exe",
    "trusttunnel.exe",
    "vpn_easy.dll",
    "wintun.dll",
    "WebView2Loader.dll",
];

/// Returns `%ProgramFiles%\TrustTunnel`.
pub fn install_dir() -> Result<PathBuf, InstallerError> {
    let program_files = std::env::var("ProgramFiles")
        .map_err(|_| InstallerError::Other("ProgramFiles env var not set"))?;
    Ok(PathBuf::from(program_files).join(APP_DIR))
}

/// Copy all bundled files from the installer's own directory to `dest`.
pub fn copy_files(dest: &Path) -> Result<(), InstallerError> {
    std::fs::create_dir_all(dest)?;

    let src_dir = installer_dir()?;

    for name in BUNDLED_FILES {
        let src = src_dir.join(name);
        let dst = dest.join(name);
        std::fs::copy(&src, &dst).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("copying {} -> {}: {}", src.display(), dst.display(), e),
            )
        })?;
    }
    Ok(())
}

/// Remove installed files and the install directory (if empty after removal).
pub fn remove_files(dir: &Path) -> Result<(), InstallerError> {
    if !dir.exists() {
        return Ok(());
    }
    for name in BUNDLED_FILES {
        let path = dir.join(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }
    // Remove directory only if now empty.
    if dir.read_dir()?.next().is_none() {
        std::fs::remove_dir(dir)?;
    }
    Ok(())
}

fn installer_dir() -> Result<PathBuf, InstallerError> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or(InstallerError::Other("cannot determine installer directory"))
}
