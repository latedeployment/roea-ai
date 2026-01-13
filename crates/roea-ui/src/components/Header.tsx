import { RefreshCw } from "lucide-react";
import { AgentStatus } from "../lib/types";

interface HeaderProps {
  connected: boolean;
  status: AgentStatus | null;
  onReconnect: () => void;
}

export function Header({ connected, status, onReconnect }: HeaderProps) {
  return (
    <header className="header">
      <div className="header-left">
        <span className="logo">roea-ai</span>
        <div className="connection-status">
          <span
            className={`status-dot ${connected ? "connected" : "disconnected"}`}
          />
          <span>{connected ? "Connected" : "Disconnected"}</span>
          {status && (
            <span style={{ marginLeft: 8, opacity: 0.7 }}>
              ({status.platform})
            </span>
          )}
        </div>
      </div>
      <div className="header-right">
        <button
          className="toolbar-button"
          onClick={onReconnect}
          title="Reconnect to agent"
        >
          <RefreshCw size={14} />
          Reconnect
        </button>
      </div>
    </header>
  );
}
