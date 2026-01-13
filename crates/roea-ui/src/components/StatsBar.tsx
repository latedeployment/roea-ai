import { Activity, Cpu, Database } from "lucide-react";

interface StatsBarProps {
  processCount: number;
  agentCount: number;
  eventsCollected: number;
}

export function StatsBar({
  processCount,
  agentCount,
  eventsCollected,
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
        <span>Active Agents:</span>
        <span className="stat-value">{agentCount}</span>
      </div>
      <div className="stat-item">
        <Database size={14} />
        <span>Events:</span>
        <span className="stat-value">{eventsCollected.toLocaleString()}</span>
      </div>
    </footer>
  );
}
