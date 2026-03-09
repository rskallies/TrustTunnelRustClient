//! Raw FFI bindings to vpn_easy.dll.
//!
//! vpn_easy.dll must be present alongside the service binary at runtime.
//! The import library (vpn_easy.lib) is required at compile time and must be
//! on the linker search path (set RUSTFLAGS or a build.rs if needed).
//!
//! # Safety contract
//! - `vpn_easy_start` fires `state_changed_cb(arg, new_state)` from an
//!   arbitrary background thread. The callback MUST NOT block or panic.
//! - `vpn_easy_stop` is synchronous; the engine is fully stopped on return.
//! - `vpn_easy_set_log_file` must be called BEFORE `vpn_easy_start`.
//! - All `*const c_char` arguments must be null-terminated and remain valid
//!   for the entire duration of the VPN session.

use std::ffi::c_void;
use std::os::raw::c_char;

/// Callback signature: `fn(arg: *mut c_void, new_state: i32)`
pub type OnStateChanged = unsafe extern "C" fn(arg: *mut c_void, new_state: i32);

#[link(name = "vpn_easy")]
extern "C" {
    /// Start (connect) the VPN engine.
    ///
    /// - `toml_config`: null-terminated TOML configuration string.
    /// - `state_changed_cb`: called on every VPN state transition.
    /// - `state_changed_cb_arg`: opaque pointer forwarded to every callback.
    pub fn vpn_easy_start(
        toml_config: *const c_char,
        state_changed_cb: Option<OnStateChanged>,
        state_changed_cb_arg: *mut c_void,
    );

    /// Stop (disconnect) the VPN engine and free all engine resources.
    pub fn vpn_easy_stop();

    /// Redirect engine debug logs to `path` (append mode).
    /// Pass a null pointer to disable logging.
    /// Must be called before `vpn_easy_start`.
    pub fn vpn_easy_set_log_file(path: *const c_char);
}
