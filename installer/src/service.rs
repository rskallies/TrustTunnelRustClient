//! Windows service registration and removal.

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;
use windows_service::{
    service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use crate::error::InstallerError;

const SERVICE_NAME: &str = "TrustTunnelService";
const SERVICE_DISPLAY: &str = "TrustTunnel VPN Service";
const SERVICE_DESC: &str = "Manages TrustTunnel VPN connections.";

/// Install and start the service pointing at `install_dir\trusttunnel-service.exe`.
pub fn install_service(install_dir: &Path) -> Result<(), InstallerError> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CREATE_SERVICE | ServiceManagerAccess::CONNECT,
    )?;

    // If already registered, just ensure it is started.
    if let Ok(existing) = manager.open_service(SERVICE_NAME, ServiceAccess::START) {
        start_if_not_running(&existing)?;
        return Ok(());
    }

    let exe_path = install_dir.join("trusttunnel-service.exe");

    let info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path,
        launch_arguments: vec![],
        dependencies: vec![],
        // Run as LocalSystem — required for TUN device creation.
        account_name: None,
        account_password: None,
    };

    let service = manager.create_service(
        &info,
        ServiceAccess::CHANGE_CONFIG | ServiceAccess::START | ServiceAccess::QUERY_STATUS,
    )?;

    // Set description (best-effort — ignore errors).
    let _ = service.set_description(SERVICE_DESC);

    start_if_not_running(&service)?;
    Ok(())
}

/// Stop and delete the service.
pub fn remove_service() -> Result<(), InstallerError> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT,
    )?;

    let service = match manager.open_service(
        SERVICE_NAME,
        ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
    ) {
        Ok(s) => s,
        Err(windows_service::Error::Winapi(e))
            if e.raw_os_error() == Some(1060) =>
        {
            // ERROR_SERVICE_DOES_NOT_EXIST — nothing to do.
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // Stop if running.
    let status = service.query_status()?;
    if status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait up to 10 s for the service to stop.
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            let s = service.query_status()?;
            if s.current_state == ServiceState::Stopped {
                break;
            }
        }
    }

    service.delete()?;
    Ok(())
}

fn start_if_not_running(
    service: &windows_service::service::Service,
) -> Result<(), InstallerError> {
    let status = service.query_status()?;
    if status.current_state == ServiceState::Stopped
        || status.current_state == ServiceState::StopPending
    {
        service.start(&[] as &[&str])?;
    }
    Ok(())
}
