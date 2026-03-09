//! TrustTunnel Windows Service binary.
//!
//! Registered with NSSM as a SYSTEM service:
//!   nssm install TrustTunnelService "C:\path\to\trusttunnel-service.exe"
//!   nssm set TrustTunnelService ObjectName LocalSystem
//!   nssm start TrustTunnelService

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod ffi;
mod ipc_server;
mod vpn_manager;

use std::ffi::OsString;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::info;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "TrustTunnelService";

define_windows_service!(ffi_service_main, service_main);

fn main() -> Result<(), windows_service::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(arguments) {
        tracing::error!("service_main fatal: {}", e);
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<(), error::ServiceError> {
    let (state_tx, _) = broadcast::channel::<shared::VpnState>(64);
    let manager = Arc::new(vpn_manager::VpnManager::new(state_tx));

    let manager_for_handler = Arc::clone(&manager);
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                manager_for_handler.stop();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::ZERO,
        process_id: None,
    })?;

    info!("TrustTunnel service started");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(ipc_server::run_ipc_server(manager))
        .ok();

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::ZERO,
        process_id: None,
    })?;

    Ok(())
}
