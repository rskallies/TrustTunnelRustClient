import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Server, VpnState, defaultServer } from "./types";

interface AppState {
  // VPN runtime state (not persisted)
  vpnState: VpnState;
  activeServerId: string | null;
  errorMessage: string | null;

  // Server profiles (persisted via localStorage until we wire Tauri fs)
  servers: Server[];
  selectedServerId: string | null;

  // Actions
  setVpnState: (state: VpnState) => void;
  setError: (message: string | null) => void;
  setActiveServerId: (id: string | null) => void;
  selectServer: (id: string) => void;
  addServer: (server: Omit<Server, "id">) => void;
  updateServer: (id: string, patch: Partial<Omit<Server, "id">>) => void;
  deleteServer: (id: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      vpnState: "idle",
      activeServerId: null,
      errorMessage: null,
      servers: [],
      selectedServerId: null,

      setVpnState: (vpnState) => set({ vpnState }),
      setError: (errorMessage) => set({ errorMessage }),
      setActiveServerId: (activeServerId) => set({ activeServerId }),
      selectServer: (selectedServerId) => set({ selectedServerId }),

      addServer: (server) =>
        set((s) => ({
          servers: [...s.servers, { ...server, id: crypto.randomUUID() }],
        })),

      updateServer: (id, patch) =>
        set((s) => ({
          servers: s.servers.map((sv) =>
            sv.id === id ? { ...sv, ...patch } : sv
          ),
        })),

      deleteServer: (id) =>
        set((s) => ({
          servers: s.servers.filter((sv) => sv.id !== id),
          selectedServerId: s.selectedServerId === id ? null : s.selectedServerId,
        })),
    }),
    {
      name: "trusttunnel-storage",
      // Only persist server profiles and selection, not runtime VPN state.
      partialize: (s) => ({
        servers: s.servers,
        selectedServerId: s.selectedServerId,
      }),
    }
  )
);
