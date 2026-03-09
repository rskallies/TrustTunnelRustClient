//! TrustTunnel installer/uninstaller.
//!
//! Usage:
//!   trusttunnel-installer.exe           -- install (default)
//!   trusttunnel-installer.exe install   -- install files + register + start service
//!   trusttunnel-installer.exe uninstall -- stop + delete service + remove files

// Always use the console subsystem so the user sees output and errors.

mod error;
mod files;
mod service;
mod shortcut;

use std::env;
use error::InstallerError;

fn main() {
    let action = env::args().nth(1).unwrap_or_default();

    // Re-launch with elevation if not already running as admin.
    if !is_elevated() {
        relaunch_as_admin(&action);
        return;
    }

    let result = match action.as_str() {
        "install" | "" => install(),
        "uninstall"    => uninstall(),
        _ => {
            eprintln!("Usage: trusttunnel-installer.exe [install|uninstall]");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("\nError: {}", e);
        pause();
        std::process::exit(1);
    }
    pause();
}

/// Returns true if the current process has administrator privileges.
fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Re-launch this binary with ShellExecute "runas" to trigger a UAC prompt.
fn relaunch_as_admin(action: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("Cannot determine exe path: {}", e); return; }
    };

    let exe_w: Vec<u16> = exe.to_string_lossy().encode_utf16().chain(Some(0)).collect();
    let params_w: Vec<u16> = action.encode_utf16().chain(Some(0)).collect();
    let verb_w: Vec<u16> = "runas\0".encode_utf16().collect();

    unsafe {
        ShellExecuteW(
            None,
            PCWSTR::from_raw(verb_w.as_ptr()),
            PCWSTR::from_raw(exe_w.as_ptr()),
            PCWSTR::from_raw(params_w.as_ptr()),
            PCWSTR::null(),
            SW_SHOW,
        );
    }
}

fn pause() {
    use std::io::{self, Write};
    print!("\nPress Enter to close...");
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}

fn install() -> Result<(), InstallerError> {
    println!("Installing TrustTunnel...");

    let install_dir = files::install_dir()?;
    println!("  Install directory: {}", install_dir.display());

    files::copy_files(&install_dir)?;
    println!("  Files copied.");

    service::install_service(&install_dir)?;
    println!("  Service registered and started.");

    shortcut::create_start_menu_shortcut(&install_dir)?;
    println!("  Start Menu shortcut created.");

    println!("Installation complete.");
    Ok(())
}

fn uninstall() -> Result<(), InstallerError> {
    println!("Uninstalling TrustTunnel...");

    service::remove_service()?;
    println!("  Service stopped and removed.");

    shortcut::remove_start_menu_shortcut()?;
    println!("  Start Menu shortcut removed.");

    let install_dir = files::install_dir()?;
    files::remove_files(&install_dir)?;
    println!("  Files removed.");

    println!("Uninstallation complete.");
    Ok(())
}
