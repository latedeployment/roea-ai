import { X } from "lucide-react";
import { Process } from "../lib/types";

interface DetailsPanelProps {
  process: Process;
  onClose: () => void;
}

export function DetailsPanel({ process, onClose }: DetailsPanelProps) {
  const formatTimestamp = (ts: number) => {
    if (!ts) return "—";
    return new Date(ts).toLocaleString();
  };

  return (
    <aside className="details-panel">
      <div className="details-header">
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <div>
            <div className="details-title">{process.name}</div>
            <div className="details-subtitle">
              PID: {process.pid}
              {process.agentType && ` • ${process.agentType}`}
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

      <div className="details-section">
        <div className="details-section-title">Process Info</div>
        <div className="details-row">
          <span className="details-label">PID</span>
          <span className="details-value">{process.pid}</span>
        </div>
        <div className="details-row">
          <span className="details-label">Parent PID</span>
          <span className="details-value">{process.ppid || "—"}</span>
        </div>
        <div className="details-row">
          <span className="details-label">User</span>
          <span className="details-value">{process.user || "—"}</span>
        </div>
        <div className="details-row">
          <span className="details-label">Started</span>
          <span className="details-value">
            {formatTimestamp(process.startTime)}
          </span>
        </div>
        {process.endTime > 0 && (
          <div className="details-row">
            <span className="details-label">Ended</span>
            <span className="details-value">
              {formatTimestamp(process.endTime)}
            </span>
          </div>
        )}
      </div>

      {process.agentType && (
        <div className="details-section">
          <div className="details-section-title">Agent Info</div>
          <div className="details-row">
            <span className="details-label">Agent Type</span>
            <span className="details-value">{process.agentType}</span>
          </div>
        </div>
      )}

      <div className="details-section">
        <div className="details-section-title">Executable</div>
        <div style={{ fontSize: 12, wordBreak: "break-all", color: "var(--text-secondary)" }}>
          {process.exePath || "—"}
        </div>
      </div>

      {process.cwd && (
        <div className="details-section">
          <div className="details-section-title">Working Directory</div>
          <div style={{ fontSize: 12, wordBreak: "break-all", color: "var(--text-secondary)" }}>
            {process.cwd}
          </div>
        </div>
      )}

      {process.cmdline && (
        <div className="details-section">
          <div className="details-section-title">Command Line</div>
          <div
            style={{
              fontSize: 11,
              fontFamily: "monospace",
              background: "var(--bg-primary)",
              padding: 8,
              borderRadius: 4,
              wordBreak: "break-all",
              color: "var(--text-secondary)",
            }}
          >
            {process.cmdline}
          </div>
        </div>
      )}
    </aside>
  );
}
