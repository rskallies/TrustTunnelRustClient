//! TrustTunnel installer/uninstaller.
//!
//! Must be run as Administrator.
//!
//! Usage:
//!   trusttunnel-installer.exe install    -- install files + register + start service
//!   trusttunnel-installer.exe uninstall  -- stop + delete service + remove files

// Always use the console subsystem so the user sees output and errors.

mod error;
mod files;
mod service;
mod shortcut;

use std::env;
use error::InstallerError;

fn main() {
    let action = env::args().nth(1).unwrap_or_default();

    let result = match action.as_str() {
        "install" | "" => install(),
        "uninstall"    => uninstall(),
        _ => {
            eprintln!("Usage: trusttunnel-installer.exe [install|uninstall]");
            eprintln!("       install is the default when run without arguments.");
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
