import { AgentWithCount } from "../lib/types";

interface SidebarProps {
  agents: AgentWithCount[];
  selectedAgent: string | null;
  onSelectAgent: (agent: string | null) => void;
}

const agentIcons: Record<string, string> = {
  claude_code: "CC",
  cursor: "Cu",
  aider: "Ai",
  windsurf: "Ws",
  copilot: "Cp",
  continue_dev: "Cn",
};

const agentColors: Record<string, string> = {
  claude_code: "claude",
  cursor: "cursor",
  aider: "aider",
  windsurf: "windsurf",
  copilot: "copilot",
  continue_dev: "unknown",
};

export function Sidebar({ agents, selectedAgent, onSelectAgent }: SidebarProps) {
  const activeAgents = agents.filter((a) => a.count > 0);
  const inactiveAgents = agents.filter((a) => a.count === 0);

  return (
    <aside className="sidebar">
      <div className="sidebar-section">
        <div className="sidebar-title">Active Agents</div>
        <div className="agent-list">
          {activeAgents.length === 0 ? (
            <div style={{ color: "var(--text-muted)", fontSize: 12, padding: 8 }}>
              No active agents detected
            </div>
          ) : (
            activeAgents.map((agent) => (
              <div
                key={agent.signature.name}
                className={`agent-item ${
                  selectedAgent === agent.signature.name ? "active" : ""
                }`}
                onClick={() =>
                  onSelectAgent(
                    selectedAgent === agent.signature.name
                      ? null
                      : agent.signature.name
                  )
                }
              >
                <div
                  className={`agent-icon ${
                    agentColors[agent.signature.name] || "unknown"
                  }`}
                >
                  {agentIcons[agent.signature.name] || "??"}
                </div>
                <span className="agent-name">{agent.signature.displayName}</span>
                <span className="agent-count">{agent.count}</span>
              </div>
            ))
          )}
        </div>
      </div>

      <div className="sidebar-section">
        <div className="sidebar-title">All Agents</div>
        <div className="agent-list">
          {inactiveAgents.map((agent) => (
            <div
              key={agent.signature.name}
              className="agent-item"
              style={{ opacity: 0.5 }}
            >
              <div
                className={`agent-icon ${
                  agentColors[agent.signature.name] || "unknown"
                }`}
              >
                {agentIcons[agent.signature.name] || "??"}
              </div>
              <span className="agent-name">{agent.signature.displayName}</span>
              <span className="agent-count">0</span>
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
}
