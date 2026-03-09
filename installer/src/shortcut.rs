//! Start Menu shortcut creation and removal using the Windows Shell COM API.

use std::path::Path;
use windows::{
    core::{Interface, PCWSTR, BSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize,
            IPersistFile,
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        },
        UI::Shell::{IShellLinkW, ShellLink},
    },
};
use crate::error::InstallerError;

const SHORTCUT_NAME: &str = "TrustTunnel.lnk";
const APP_NAME: &str = "TrustTunnel";

/// Create a Start Menu shortcut pointing to `install_dir\trusttunnel.exe`.
pub fn create_start_menu_shortcut(install_dir: &Path) -> Result<(), InstallerError> {
    let shortcut_dir = start_menu_dir()?;
    std::fs::create_dir_all(&shortcut_dir)?;

    let target = install_dir.join("trusttunnel.exe");
    let shortcut_path = shortcut_dir.join(SHORTCUT_NAME);

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        shell_link.SetPath(PCWSTR::from_raw(
            to_wide_null(target.to_string_lossy().as_ref()).as_ptr(),
        ))?;

        shell_link.SetWorkingDirectory(PCWSTR::from_raw(
            to_wide_null(install_dir.to_string_lossy().as_ref()).as_ptr(),
        ))?;

        shell_link.SetDescription(PCWSTR::from_raw(
            to_wide_null("TrustTunnel VPN Client").as_ptr(),
        ))?;

        let persist: IPersistFile = shell_link.cast()?;
        let path_bstr = BSTR::from(shortcut_path.to_string_lossy().as_ref());
        persist.Save(PCWSTR::from_raw(path_bstr.as_ptr()), true)?;

        CoUninitialize();
    }

    Ok(())
}

/// Remove the Start Menu shortcut.
pub fn remove_start_menu_shortcut() -> Result<(), InstallerError> {
    let path = start_menu_dir()?.join(SHORTCUT_NAME);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Returns `%ProgramData%\Microsoft\Windows\Start Menu\Programs\TrustTunnel`.
fn start_menu_dir() -> Result<std::path::PathBuf, InstallerError> {
    let program_data = std::env::var("ProgramData")
        .map_err(|_| InstallerError::Other("ProgramData env var not set"))?;
    Ok(std::path::PathBuf::from(program_data)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join(APP_NAME))
}

/// Convert a Rust `&str` to a null-terminated UTF-16 `Vec<u16>`.
fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
