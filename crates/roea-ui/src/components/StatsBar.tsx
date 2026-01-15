import { Activity, Cpu, Database, FileText, Globe } from "lucide-react";

interface StatsBarProps {
  processCount: number;
  agentCount: number;
  eventsCollected: number;
  fileOpsCount: number;
  connectionsCount: number;
}

export function StatsBar({
  processCount,
  agentCount,
  eventsCollected,
  fileOpsCount,
  connectionsCount,
}: StatsBarProps) {
  return (
    <footer className="stats-bar">
      <div className="stat-item">
        <Cpu size={14} />
        <span>Processes:</span>
        <span className="stat-value">{processCount}</span>
      </div>
      <div className="stat-item">
        <Activity size={14} />
        <span>Agents:</span>
        <span className="stat-value">{agentCount}</span>
      </div>
      <div className="stat-item">
        <Database size={14} />
        <span>Events:</span>
        <span className="stat-value">{eventsCollected.toLocaleString()}</span>
      </div>
      <div className="stat-item">
        <FileText size={14} />
        <span>Files:</span>
        <span className="stat-value">{fileOpsCount}</span>
      </div>
      <div className="stat-item">
        <Globe size={14} />
        <span>Connections:</span>
        <span className="stat-value">{connectionsCount}</span>
      </div>
    </footer>
  );
}
