//! Named pipe client talking to the TrustTunnel service.
//!
//! A single persistent connection is held in Tauri managed state.
//! Commands (GUI → service) and synchronous replies share the pipe with
//! unsolicited Event pushes (service → GUI). The event listener task
//! forwards pushes to the Tauri app via window events.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use shared::{Command, Event, PIPE_NAME, decode_frame, encode_frame};
use crate::error::GuiError;

pub struct IpcClient {
    inner: Arc<Mutex<tokio::net::windows::named_pipe::NamedPipeClient>>,
}

impl IpcClient {
    /// Connect to the service pipe. Returns an error if the service is not running.
    pub async fn connect() -> Result<Self, GuiError> {
        let pipe = ClientOptions::new()
            .open(PIPE_NAME)
            .map_err(|e| GuiError::ServiceUnavailable(e.to_string()))?;
        info!("Connected to service IPC pipe");
        Ok(Self {
            inner: Arc::new(Mutex::new(pipe)),
        })
    }

    /// Send a command and receive the synchronous reply Event.
    pub async fn send_command(&self, cmd: &Command) -> Result<Event, GuiError> {
        let frame = encode_frame(cmd)?;
        let mut pipe = self.inner.lock().await;

        pipe.write_all(&frame).await.map_err(GuiError::Io)?;

        let mut len_buf = [0u8; 4];
        pipe.read_exact(&mut len_buf).await.map_err(GuiError::Io)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len > 1024 * 1024 {
            return Err(GuiError::Protocol("reply too large"));
        }

        let mut body = vec![0u8; len];
        pipe.read_exact(&mut body).await.map_err(GuiError::Io)?;

        decode_frame::<Event>(&body).map_err(GuiError::Json)
    }

    /// Spawn a background task that reads unsolicited Events and calls `on_event`.
    pub fn spawn_event_listener<F>(
        inner: Arc<Mutex<tokio::net::windows::named_pipe::NamedPipeClient>>,
        on_event: F,
    ) where
        F: Fn(Event) + Send + 'static,
    {
        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            loop {
                let mut pipe = inner.lock().await;
                match pipe.read_exact(&mut len_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Event listener disconnected: {}", e);
                        break;
                    }
                }
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                if pipe.read_exact(&mut body).await.is_err() {
                    break;
                }
                drop(pipe); // release lock before callback
                match serde_json::from_slice::<Event>(&body) {
                    Ok(event) => on_event(event),
                    Err(e) => error!("Failed to decode service event: {}", e),
                }
            }
        });
    }

    pub fn inner(&self) -> Arc<Mutex<tokio::net::windows::named_pipe::NamedPipeClient>> {
        Arc::clone(&self.inner)
    }
}
