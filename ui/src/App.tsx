import { useEffect, useState } from "react";
import { useAppStore } from "./store";
import { encodeConfig } from "./configEncoder";
import { connectVpn, disconnectVpn, getStatus, onServiceEvent } from "./tauriApi";
import { ServerList } from "./components/ServerList";
import { ServerForm } from "./components/ServerForm";
import { VpnStatusBadge } from "./components/VpnStatusBadge";
import { Server } from "./types";

type View = "servers" | "add" | "edit";

export default function App() {
  const {
    vpnState, errorMessage, activeServerId,
    servers, selectedServerId,
    setVpnState, setError, setActiveServerId,
    selectServer, addServer, updateServer, deleteServer,
  } = useAppStore();

  const [view, setView] = useState<View>("servers");
  const [editTarget, setEditTarget] = useState<Server | null>(null);

  // Connect to service and subscribe to events on mount.
  useEffect(() => {
    getStatus().then(setVpnState).catch(() => {
      // Service not running — UI still functional, commands will fail gracefully.
    });

    const unlisten = onServiceEvent((event) => {
      if (event.type === "state_changed" || event.type === "status_response") {
        setVpnState(event.state);
        if (event.state === "disconnected" || event.state === "idle") {
          setActiveServerId(null);
        }
      } else if (event.type === "error") {
        setError(event.message);
        setVpnState("disconnected");
        setActiveServerId(null);
      }
    });

    return () => { unlisten.then((fn) => fn()); };
  }, []);

  async function handleConnect(server: Server) {
    setError(null);
    try {
      const toml = encodeConfig(server);
      const state = await connectVpn(toml);
      setVpnState(state);
      setActiveServerId(server.id);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDisconnect() {
    setError(null);
    try {
      const state = await disconnectVpn();
      setVpnState(state);
    } catch (e) {
      setError(String(e));
    }
  }

  function handleEdit(server: Server) {
    setEditTarget(server);
    setView("edit");
  }

  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-gray-800">
        <div className="flex items-center gap-3">
          <span className="font-bold text-white">TrustTunnel</span>
          <VpnStatusBadge state={vpnState} />
        </div>
        {view === "servers" && (
          <button
            onClick={() => setView("add")}
            className="text-sm px-3 py-1 rounded-md bg-blue-600 hover:bg-blue-700"
          >
            + Add Server
          </button>
        )}
      </header>

      {/* Error banner */}
      {errorMessage && (
        <div className="mx-4 mt-3 px-3 py-2 rounded-md bg-red-900/50 border border-red-700 text-sm text-red-300 flex items-center justify-between">
          <span>{errorMessage}</span>
          <button onClick={() => setError(null)} className="ml-2 text-red-400 hover:text-red-200">✕</button>
        </div>
      )}

      {/* Content */}
      <main className="flex-1 overflow-y-auto p-4">
        {view === "servers" && (
          <ServerList
            servers={servers}
            selectedId={selectedServerId}
            activeId={activeServerId}
            vpnState={vpnState}
            onSelect={selectServer}
            onEdit={handleEdit}
            onDelete={deleteServer}
            onConnect={handleConnect}
            onDisconnect={handleDisconnect}
          />
        )}

        {view === "add" && (
          <ServerForm
            onSave={(values) => {
              addServer(values);
              setView("servers");
            }}
            onCancel={() => setView("servers")}
          />
        )}

        {view === "edit" && editTarget && (
          <ServerForm
            initial={editTarget}
            onSave={(values) => {
              updateServer(editTarget.id, values);
              setView("servers");
              setEditTarget(null);
            }}
            onCancel={() => { setView("servers"); setEditTarget(null); }}
          />
        )}
      </main>
    </div>
  );
}
