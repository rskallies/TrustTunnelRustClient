//! Named pipe IPC server.
//!
//! Accepts GUI client connections and multiplexes inbound Commands with
//! outbound unsolicited state-change Event pushes on a single pipe connection.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use shared::{Command, Event, VpnState, PIPE_NAME, decode_frame, encode_frame};
use crate::error::ServiceError;
use crate::vpn_manager::VpnManager;

pub async fn run_ipc_server(manager: Arc<VpnManager>) -> Result<(), ServiceError> {
    info!("IPC server listening on {}", PIPE_NAME);

    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(false)
            .pipe_mode(PipeMode::Byte)
            .create(PIPE_NAME)
            .map_err(ServiceError::Io)?;

        server.connect().await.map_err(ServiceError::Io)?;
        info!("IPC client connected");

        let manager = Arc::clone(&manager);
        let state_rx = manager.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(server, manager, state_rx).await {
                warn!("IPC client handler exited: {}", e);
            }
            info!("IPC client disconnected");
        });
    }
}

async fn handle_client<S>(
    mut stream: S,
    manager: Arc<VpnManager>,
    mut state_rx: broadcast::Receiver<VpnState>,
) -> Result<(), ServiceError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let mut len_buf = [0u8; 4];

    loop {
        tokio::select! {
            // ── Inbound Command from GUI ────────────────────────────────────
            result = stream.read_exact(&mut len_buf) => {
                result.map_err(ServiceError::Io)?;
                let len = u32::from_le_bytes(len_buf) as usize;

                if len > 1024 * 1024 {
                    error!("IPC: oversized message ({} bytes)", len);
                    return Err(ServiceError::Protocol("message too large"));
                }

                let mut body = vec![0u8; len];
                stream.read_exact(&mut body).await.map_err(ServiceError::Io)?;

                let cmd: Command = decode_frame(&body)
                    .map_err(|_| ServiceError::Protocol("invalid JSON"))?;

                let reply = dispatch_command(cmd, &manager).await;
                let frame = encode_frame(&reply)
                    .map_err(|_| ServiceError::Protocol("encode error"))?;
                stream.write_all(&frame).await.map_err(ServiceError::Io)?;
            }

            // ── Outbound unsolicited state-change Event push ────────────────
            result = state_rx.recv() => {
                match result {
                    Ok(state) => {
                        let event = Event::StateChanged { state };
                        let frame = encode_frame(&event)
                            .map_err(|_| ServiceError::Protocol("encode error"))?;
                        stream.write_all(&frame).await.map_err(ServiceError::Io)?;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("IPC: dropped {} state events (lagged receiver)", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn dispatch_command(cmd: Command, manager: &VpnManager) -> Event {
    match cmd {
        Command::Connect { config_toml } => match manager.start(&config_toml) {
            Ok(()) => Event::StateChanged { state: VpnState::Connecting },
            Err(e) => Event::Error { message: e.to_string() },
        },
        Command::Disconnect => {
            manager.stop();
            Event::StateChanged { state: VpnState::Disconnecting }
        }
        Command::GetStatus => Event::StatusResponse { state: manager.current_state() },
    }
}
