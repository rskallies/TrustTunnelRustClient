// Mirrors shared/src/lib.rs — must stay in sync with Rust types.

export type VpnState =
  | "idle"
  | "connecting"
  | "connected"
  | "disconnecting"
  | "disconnected"
  | "reconnecting";

export type VpnProtocol = "http2" | "http3";

export type ServiceEvent =
  | { type: "state_changed"; state: VpnState }
  | { type: "error"; message: string }
  | { type: "status_response"; state: VpnState };

// Server profile — stored in Tauri's app data directory.
export interface Server {
  id: string;
  name: string;
  ipAddress: string;
  domain: string;
  username: string;
  password: string;
  protocol: VpnProtocol;
  fallbackProtocol: VpnProtocol | null;
  dnsUpstreams: string[];
  excludedRoutes: string[];
  includedRoutes: string[];
  killSwitch: boolean;
  postQuantum: boolean;
  antiDpi: boolean;
  mtuSize: number;
  skipVerification: boolean;
  certificate: string;
}

export function defaultServer(): Omit<Server, "id" | "name"> {
  return {
    ipAddress: "",
    domain: "",
    username: "",
    password: "",
    protocol: "http2",
    fallbackProtocol: null,
    dnsUpstreams: [],
    excludedRoutes: [],
    includedRoutes: [],
    killSwitch: false,
    postQuantum: false,
    antiDpi: false,
    mtuSize: 1500,
    skipVerification: false,
    certificate: "",
  };
}
