//! TrustTunnel GUI — Tauri v2 entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod ipc_client;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

use commands::ClientState;
use ipc_client::IpcClient;
use shared::VpnState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let rt = tokio::runtime::Handle::current();

            let client = rt.block_on(async {
                IpcClient::connect()
                    .await
                    .expect("TrustTunnel service is not running. Start it before launching the GUI.")
            });

            // Forward service state-change pushes as Tauri window events
            // and update the tray icon on connect/disconnect.
            let pipe_inner = client.inner();
            let app_handle = app.handle().clone();
            IpcClient::spawn_event_listener(pipe_inner, move |event| {
                // Swap tray icon based on VPN state.
                if let shared::Event::StateChanged { state } = &event {
                    if let Some(tray) = app_handle.tray_by_id("main") {
                        let icon_bytes: &[u8] = match state {
                            VpnState::Connected => {
                                include_bytes!("../icons/tray_icon_connected.png")
                            }
                            _ => include_bytes!("../icons/tray_icon.png"),
                        };
                        if let Ok(img) = Image::from_bytes(icon_bytes) {
                            let _ = tray.set_icon(Some(img));
                        }
                    }
                }
                let _ = app_handle.emit("vpn-event", &event);
            });

            app.manage(ClientState(tokio::sync::Mutex::new(client)));

            // System tray menu.
            let connect_item    = MenuItem::with_id(app, "connect",    "Connect",    true, None::<&str>)?;
            let disconnect_item = MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)?;
            let quit_item       = MenuItem::with_id(app, "quit",       "Quit",       true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&connect_item, &disconnect_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "connect" => {
                        // TODO: open main window to select server and connect.
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "disconnect" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app.state::<ClientState>();
                            if let Err(e) = commands::disconnect(state).await {
                                tracing::error!("Disconnect failed: {}", e);
                            }
                        });
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Toggle main window on left click.
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_status,
        ])
        .on_window_event(|window, event| {
            // Hide to tray instead of closing.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
