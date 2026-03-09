// Typed wrappers around Tauri invoke commands and events.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { VpnState, ServiceEvent } from "./types";

export async function connectVpn(configToml: string): Promise<VpnState> {
  return invoke<VpnState>("connect", { configToml });
}

export async function disconnectVpn(): Promise<VpnState> {
  return invoke<VpnState>("disconnect");
}

export async function getStatus(): Promise<VpnState> {
  return invoke<VpnState>("get_status");
}

/** Subscribe to unsolicited service events (state changes, errors). */
export function onServiceEvent(handler: (event: ServiceEvent) => void) {
  return listen<ServiceEvent>("vpn-event", (e) => handler(e.payload));
}
