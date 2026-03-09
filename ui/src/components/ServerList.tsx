import { Server, VpnState } from "../types";

interface Props {
  servers: Server[];
  selectedId: string | null;
  activeId: string | null;
  vpnState: VpnState;
  onSelect: (id: string) => void;
  onEdit: (server: Server) => void;
  onDelete: (id: string) => void;
  onConnect: (server: Server) => void;
  onDisconnect: () => void;
}

export function ServerList({
  servers,
  selectedId,
  activeId,
  vpnState,
  onSelect,
  onEdit,
  onDelete,
  onConnect,
  onDisconnect,
}: Props) {
  const isBusy =
    vpnState === "connecting" ||
    vpnState === "disconnecting" ||
    vpnState === "reconnecting";

  if (servers.length === 0) {
    return (
      <p className="text-center text-gray-500 py-8 text-sm">
        No servers yet. Add one to get started.
      </p>
    );
  }

  return (
    <ul className="space-y-2">
      {servers.map((server) => {
        const isActive = server.id === activeId;
        const isSelected = server.id === selectedId;
        const isConnected = isActive && vpnState === "connected";

        return (
          <li
            key={server.id}
            onClick={() => onSelect(server.id)}
            className={`rounded-lg p-3 cursor-pointer border transition-colors ${
              isSelected
                ? "border-blue-500 bg-gray-800"
                : "border-gray-700 bg-gray-900 hover:bg-gray-800"
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="min-w-0">
                <p className="font-medium truncate">{server.name}</p>
                <p className="text-xs text-gray-400 truncate">
                  {server.domain || server.ipAddress}
                </p>
              </div>

              <div className="flex items-center gap-2 ml-2 shrink-0">
                {isConnected ? (
                  <button
                    onClick={(e) => { e.stopPropagation(); onDisconnect(); }}
                    disabled={isBusy}
                    className="px-3 py-1 text-xs rounded-md bg-red-600 hover:bg-red-700 disabled:opacity-50"
                  >
                    Disconnect
                  </button>
                ) : (
                  <button
                    onClick={(e) => { e.stopPropagation(); onConnect(server); }}
                    disabled={isBusy || vpnState === "connected"}
                    className="px-3 py-1 text-xs rounded-md bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
                  >
                    Connect
                  </button>
                )}
                <button
                  onClick={(e) => { e.stopPropagation(); onEdit(server); }}
                  className="p-1 text-gray-400 hover:text-white"
                  title="Edit"
                >
                  ✎
                </button>
                <button
                  onClick={(e) => { e.stopPropagation(); onDelete(server.id); }}
                  className="p-1 text-gray-400 hover:text-red-400"
                  title="Delete"
                >
                  ✕
                </button>
              </div>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
