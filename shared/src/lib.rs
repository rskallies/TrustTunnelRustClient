//! Shared IPC types between the TrustTunnel service and GUI.
//!
//! Wire format: each message is a length-prefixed JSON frame.
//!
//! Frame layout (little-endian):
//!   [u32 len][len bytes of UTF-8 JSON]

use serde::{Deserialize, Serialize};

/// The canonical VPN state mirroring VPN_SS_* from vpn_easy.h.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VpnState {
    /// VPN_SS_IDLE = 0
    Idle = 0,
    /// VPN_SS_CONNECTING = 1
    Connecting = 1,
    /// VPN_SS_CONNECTED = 2
    Connected = 2,
    /// VPN_SS_DISCONNECTING = 3
    Disconnecting = 3,
    /// VPN_SS_DISCONNECTED = 4
    Disconnected = 4,
    /// VPN_SS_RECONNECTING = 5
    Reconnecting = 5,
}

impl TryFrom<i32> for VpnState {
    type Error = i32;
    fn try_from(v: i32) -> Result<Self, i32> {
        match v {
            0 => Ok(Self::Idle),
            1 => Ok(Self::Connecting),
            2 => Ok(Self::Connected),
            3 => Ok(Self::Disconnecting),
            4 => Ok(Self::Disconnected),
            5 => Ok(Self::Reconnecting),
            _ => Err(v),
        }
    }
}

/// Commands sent from the GUI to the service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Start the VPN with the given TOML configuration string.
    Connect { config_toml: String },
    /// Tear down an active VPN session.
    Disconnect,
    /// Request the current VPN state; service replies with StatusResponse.
    GetStatus,
}

/// Events sent from the service to the GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Emitted every time vpn_easy fires its state-changed callback.
    StateChanged { state: VpnState },
    /// Emitted when a non-recoverable error occurs inside the service.
    Error { message: String },
    /// Reply to a GetStatus command.
    StatusResponse { state: VpnState },
}

/// Named pipe path used by both service and GUI.
pub const PIPE_NAME: &str = r"\\.\pipe\TrustTunnelService";

/// Encode a message as a length-prefixed JSON frame.
pub fn encode_frame<T: Serialize>(msg: &T) -> Result<Vec<u8>, serde_json::Error> {
    let json = serde_json::to_vec(msg)?;
    let len = json.len() as u32;
    let mut out = Vec::with_capacity(4 + json.len());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&json);
    Ok(out)
}

/// Decode a length-prefixed JSON payload (without the 4-byte length prefix).
pub fn decode_frame<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Result<T, serde_json::Error> {
    serde_json::from_slice(buf)
}
