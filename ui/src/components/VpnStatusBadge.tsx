import { VpnState } from "../types";

const STATE_LABELS: Record<VpnState, string> = {
  idle: "Idle",
  connecting: "Connecting…",
  connected: "Connected",
  disconnecting: "Disconnecting…",
  disconnected: "Disconnected",
  reconnecting: "Reconnecting…",
};

const STATE_COLORS: Record<VpnState, string> = {
  idle: "bg-gray-500",
  connecting: "bg-yellow-500",
  connected: "bg-green-500",
  disconnecting: "bg-yellow-500",
  disconnected: "bg-red-500",
  reconnecting: "bg-yellow-500",
};

interface Props {
  state: VpnState;
}

export function VpnStatusBadge({ state }: Props) {
  return (
    <div className="flex items-center gap-2">
      <span className={`w-2.5 h-2.5 rounded-full ${STATE_COLORS[state]}`} />
      <span className="text-sm font-medium text-gray-300">{STATE_LABELS[state]}</span>
    </div>
  );
}
