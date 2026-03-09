//! Tauri invoke commands exposed to the frontend.

use tauri::State;
use shared::{Command, Event, VpnState};
use crate::error::GuiError;
use crate::ipc_client::IpcClient;

/// Managed state wrapper.
pub struct ClientState(pub tokio::sync::Mutex<IpcClient>);

/// Start the VPN. `config_toml` is the full TOML configuration string.
#[tauri::command]
pub async fn connect(
    config_toml: String,
    client: State<'_, ClientState>,
) -> Result<VpnState, GuiError> {
    let guard = client.0.lock().await;
    match guard.send_command(&Command::Connect { config_toml }).await? {
        Event::StateChanged { state } => Ok(state),
        Event::Error { message } => Err(GuiError::ServiceUnavailable(message)),
        _ => Err(GuiError::Protocol("unexpected reply to Connect")),
    }
}

/// Stop the VPN.
#[tauri::command]
pub async fn disconnect(
    client: State<'_, ClientState>,
) -> Result<VpnState, GuiError> {
    let guard = client.0.lock().await;
    match guard.send_command(&Command::Disconnect).await? {
        Event::StateChanged { state } => Ok(state),
        Event::Error { message } => Err(GuiError::ServiceUnavailable(message)),
        _ => Err(GuiError::Protocol("unexpected reply to Disconnect")),
    }
}

/// Query the current VPN state.
#[tauri::command]
pub async fn get_status(
    client: State<'_, ClientState>,
) -> Result<VpnState, GuiError> {
    let guard = client.0.lock().await;
    match guard.send_command(&Command::GetStatus).await? {
        Event::StatusResponse { state } => Ok(state),
        Event::Error { message } => Err(GuiError::ServiceUnavailable(message)),
        _ => Err(GuiError::Protocol("unexpected reply to GetStatus")),
    }
}
