import { useState } from "react";
import { X, Circle, Globe, FileText, Info } from "lucide-react";
import { Process, Connection, FileOp } from "../lib/types";

interface DetailsPanelProps {
  process: Process;
  connections: Connection[];
  fileOps: FileOp[];
  onClose: () => void;
}

type TabType = "info" | "network" | "files";

export function DetailsPanel({ process, connections, fileOps, onClose }: DetailsPanelProps) {
  const [activeTab, setActiveTab] = useState<TabType>("info");

  const formatTimestamp = (ts: number) => {
    if (!ts) return "—";
    return new Date(ts * 1000).toLocaleString();
  };

  const isActive = process.endTime === 0;

  // Filter connections and file ops for this process
  const processConnections = connections.filter((c) => c.pid === process.pid);
  const processFileOps = fileOps.filter((f) => f.pid === process.pid);

  return (
    <aside className="details-panel">
      <div className="details-header">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
          <div>
            <div className="details-title">{process.name}</div>
            <div className="details-subtitle">
              <span
                className="status-indicator"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 4,
                }}
              >
                <Circle
                  size={8}
                  fill={isActive ? "var(--accent-success)" : "var(--text-muted)"}
                  stroke="none"
                />
                {isActive ? "Running" : "Exited"}
              </span>
              {process.agentType && process.agentType !== "unknown" && (
                <span style={{ marginLeft: 8, color: "var(--text-muted)" }}>
                  • {process.agentType}
                </span>
              )}
            </div>
          </div>
          <button
            onClick={onClose}
            style={{
              background: "none",
              border: "none",
              color: "var(--text-secondary)",
              cursor: "pointer",
              padding: 4,
            }}
          >
            <X size={18} />
          </button>
        </div>
      </div>

      <div className="details-tabs">
        <button
          className={`details-tab ${activeTab === "info" ? "active" : ""}`}
          onClick={() => setActiveTab("info")}
        >
          <Info size={14} />
          Process
        </button>
        <button
          className={`details-tab ${activeTab === "network" ? "active" : ""}`}
          onClick={() => setActiveTab("network")}
        >
          <Globe size={14} />
          Network
          <span className="tab-count">{processConnections.length}</span>
        </button>
        <button
          className={`details-tab ${activeTab === "files" ? "active" : ""}`}
          onClick={() => setActiveTab("files")}
        >
          <FileText size={14} />
          Files
          <span className="tab-count">{processFileOps.length}</span>
        </button>
      </div>

      <div className="details-content">
        {activeTab === "info" && (
          <>
            <div className="details-section">
              <div className="details-section-title">Process Info</div>
              <div className="details-row">
                <span className="details-label">PID</span>
                <span className="details-value monospace">{process.pid}</span>
              </div>
              <div className="details-row">
                <span className="details-label">Parent PID</span>
                <span className="details-value monospace">{process.ppid || "—"}</span>
              </div>
              <div className="details-row">
                <span className="details-label">User</span>
                <span className="details-value">{process.user || "—"}</span>
              </div>
              <div className="details-row">
                <span className="details-label">Started</span>
                <span className="details-value">{formatTimestamp(process.startTime)}</span>
              </div>
              {process.endTime > 0 && (
                <div className="details-row">
                  <span className="details-label">Ended</span>
                  <span className="details-value">{formatTimestamp(process.endTime)}</span>
                </div>
              )}
            </div>

            {process.cwd && (
              <div className="details-section">
                <div className="details-section-title">Working Directory</div>
                <div className="details-path">{process.cwd}</div>
              </div>
            )}

            <div className="details-section">
              <div className="details-section-title">Executable</div>
              <div className="details-path">{process.exePath || "—"}</div>
            </div>

            {process.cmdline && (
              <div className="details-section">
                <div className="details-section-title">Command Line</div>
                <div className="details-code">{process.cmdline}</div>
              </div>
            )}
          </>
        )}

        {activeTab === "network" && (
          <div className="details-section">
            <div className="details-section-title">
              Network Connections ({processConnections.length})
            </div>
            {processConnections.length === 0 ? (
              <div className="details-empty">No network connections</div>
            ) : (
              <div className="details-list">
                {processConnections.map((conn) => (
                  <div key={conn.id} className="details-list-item">
                    <div className="details-list-main">
                      <span className="connection-endpoint">
                        {conn.remoteAddr}:{conn.remotePort}
                      </span>
                      <span className={`connection-state ${conn.state.toLowerCase()}`}>
                        {conn.state}
                      </span>
                    </div>
                    <div className="details-list-meta">
                      {conn.protocol} • {conn.localAddr}:{conn.localPort}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === "files" && (
          <div className="details-section">
            <div className="details-section-title">
              File Operations ({processFileOps.length})
            </div>
            {processFileOps.length === 0 ? (
              <div className="details-empty">No file operations</div>
            ) : (
              <div className="details-list">
                {processFileOps.slice(0, 50).map((fo) => (
                  <div key={fo.id} className="details-list-item">
                    <div className="details-list-main">
                      <span className={`file-op-badge ${fo.operation.toLowerCase()}`}>
                        {fo.operation.toUpperCase()}
                      </span>
                      <span className="file-path">{fo.path}</span>
                    </div>
                    {fo.newPath && (
                      <div className="details-list-meta">→ {fo.newPath}</div>
                    )}
                  </div>
                ))}
                {processFileOps.length > 50 && (
                  <div className="details-more">
                    +{processFileOps.length - 50} more operations
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </aside>
  );
}
