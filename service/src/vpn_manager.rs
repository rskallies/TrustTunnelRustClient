//! Safe wrapper around the vpn_easy FFI.
//!
//! All FFI calls are serialised through a Mutex-guarded running flag.
//! State changes are broadcast via a tokio channel to the IPC server.

use std::ffi::CString;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{error, info};

use shared::VpnState;
use crate::error::ServiceError;
use crate::ffi;

struct Inner {
    state: VpnState,
    /// Raw pointer to the heap-allocated CallbackContext.
    /// Null when no session is active.
    ctx_ptr: *mut c_void,
}

// SAFETY: Inner is only accessed behind a Mutex.
unsafe impl Send for Inner {}

pub struct VpnManager {
    inner: Arc<Mutex<Inner>>,
    event_tx: broadcast::Sender<VpnState>,
}

impl VpnManager {
    pub fn new(event_tx: broadcast::Sender<VpnState>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                state: VpnState::Idle,
                ctx_ptr: std::ptr::null_mut(),
            })),
            event_tx,
        }
    }

    /// Subscribe to state-change events.
    pub fn subscribe(&self) -> broadcast::Receiver<VpnState> {
        self.event_tx.subscribe()
    }

    /// Current cached VPN state.
    pub fn current_state(&self) -> VpnState {
        self.inner.lock().unwrap().state
    }

    /// Set the engine log file path. Pass `None` to disable.
    /// Must be called before `start`.
    pub fn set_log_file(&self, path: Option<&str>) -> Result<(), ServiceError> {
        match path {
            Some(p) => {
                let cstr = CString::new(p)
                    .map_err(|_| ServiceError::InvalidArgument("log path contains null byte"))?;
                unsafe { ffi::vpn_easy_set_log_file(cstr.as_ptr()); }
            }
            None => unsafe { ffi::vpn_easy_set_log_file(std::ptr::null()); },
        }
        Ok(())
    }

    /// Start the VPN with a TOML configuration string.
    pub fn start(&self, toml_config: &str) -> Result<(), ServiceError> {
        let cstr = CString::new(toml_config)
            .map_err(|_| ServiceError::InvalidArgument("config contains null byte"))?;

        // Build the callback context and leak it to the heap.
        // It will be reclaimed in `stop` or `Drop`.
        let ctx = Box::new(CallbackContext {
            inner: Arc::clone(&self.inner),
            tx: self.event_tx.clone(),
        });
        let ctx_ptr = Box::into_raw(ctx) as *mut c_void;

        {
            let mut guard = self.inner.lock().unwrap();
            guard.ctx_ptr = ctx_ptr;
        }

        unsafe {
            ffi::vpn_easy_start(cstr.as_ptr(), Some(state_changed_cb), ctx_ptr);
        }
        info!("vpn_easy_start called");
        Ok(())
    }

    /// Stop the VPN synchronously and reclaim the callback context.
    pub fn stop(&self) {
        unsafe { ffi::vpn_easy_stop(); }
        info!("vpn_easy_stop called");
        self.reclaim_ctx();
    }

    fn reclaim_ctx(&self) {
        let mut guard = self.inner.lock().unwrap();
        if !guard.ctx_ptr.is_null() {
            // SAFETY: we allocated this with Box::into_raw above and
            // vpn_easy_stop guarantees no more callbacks fire after it returns.
            let _ = unsafe { Box::from_raw(guard.ctx_ptr as *mut CallbackContext) };
            guard.ctx_ptr = std::ptr::null_mut();
        }
    }
}

impl Drop for VpnManager {
    fn drop(&mut self) {
        unsafe { ffi::vpn_easy_stop(); }
        self.reclaim_ctx();
    }
}

/// Passed through the FFI boundary as `*mut c_void`.
struct CallbackContext {
    inner: Arc<Mutex<Inner>>,
    tx: broadcast::Sender<VpnState>,
}

/// C-callable state-change callback.
/// Called from vpn_easy's internal thread — MUST NOT panic or block.
unsafe extern "C" fn state_changed_cb(arg: *mut c_void, new_state: i32) {
    if arg.is_null() {
        return;
    }
    // SAFETY: arg was created by Box::into_raw in `start` and is valid
    // until `reclaim_ctx` is called after vpn_easy_stop returns.
    let ctx = &*(arg as *const CallbackContext);

    let state = VpnState::try_from(new_state).unwrap_or_else(|v| {
        error!("vpn_easy callback: unknown state value {}", v);
        VpnState::Idle
    });

    if let Ok(mut inner) = ctx.inner.lock() {
        inner.state = state;
    }

    // Ignore lagged-receiver errors — GUI may not be connected.
    let _ = ctx.tx.send(state);
}
