import { useState, useMemo } from "react";
import { ChevronRight, ChevronDown } from "lucide-react";
import { Process } from "../lib/types";

interface ProcessTreeProps {
  processes: Process[];
  selectedProcess: Process | null;
  onSelectProcess: (process: Process | null) => void;
}

interface TreeNode {
  process: Process;
  children: TreeNode[];
}

interface AgentGroup {
  agentType: string;
  displayName: string;
  rootProcesses: TreeNode[];
  processCount: number;
  activeCount: number;
}

const agentIcons: Record<string, string> = {
  claude_code: "CC",
  cursor: "Cu",
  aider: "Ai",
  windsurf: "Ws",
  copilot: "Cp",
  continue_dev: "Cn",
};

const agentDisplayNames: Record<string, string> = {
  claude_code: "Claude Code",
  cursor: "Cursor",
  aider: "Aider",
  windsurf: "Windsurf",
  copilot: "GitHub Copilot",
  continue_dev: "Continue",
};

const agentColors: Record<string, string> = {
  claude_code: "var(--agent-claude)",
  cursor: "var(--agent-cursor)",
  aider: "var(--agent-aider)",
  windsurf: "var(--agent-windsurf)",
  copilot: "var(--agent-copilot)",
  continue_dev: "var(--agent-unknown)",
};

function buildAgentGroups(processes: Process[]): AgentGroup[] {
  // Group processes by agent type
  const agentProcessMap = new Map<string, Process[]>();
  const processMap = new Map<number, Process>();
  const childrenMap = new Map<number, Process[]>();

  processes.forEach((p) => {
    processMap.set(p.pid, p);

    // Build children map
    if (!childrenMap.has(p.ppid)) {
      childrenMap.set(p.ppid, []);
    }
    childrenMap.get(p.ppid)!.push(p);
  });

  // Find the agent type for each process (inherit from parent if needed)
  const resolvedAgentTypes = new Map<number, string>();

  function resolveAgentType(p: Process): string {
    if (resolvedAgentTypes.has(p.pid)) {
      return resolvedAgentTypes.get(p.pid)!;
    }

    if (p.agentType && p.agentType !== "" && p.agentType !== "unknown") {
      resolvedAgentTypes.set(p.pid, p.agentType);
      return p.agentType;
    }

    // Check parent
    const parent = processMap.get(p.ppid);
    if (parent) {
      const parentAgent = resolveAgentType(parent);
      resolvedAgentTypes.set(p.pid, parentAgent);
      return parentAgent;
    }

    resolvedAgentTypes.set(p.pid, "");
    return "";
  }

  // Resolve all agent types and group
  processes.forEach((p) => {
    const agentType = resolveAgentType(p);
    if (agentType) {
      if (!agentProcessMap.has(agentType)) {
        agentProcessMap.set(agentType, []);
      }
      agentProcessMap.get(agentType)!.push(p);
    }
  });

  // Build tree for each agent group
  function buildNode(process: Process, agentProcesses: Set<number>): TreeNode {
    const children = (childrenMap.get(process.pid) || [])
      .filter((child) => agentProcesses.has(child.pid))
      .sort((a, b) => a.pid - b.pid)
      .map((child) => buildNode(child, agentProcesses));
    return { process, children };
  }

  const groups: AgentGroup[] = [];

  agentProcessMap.forEach((procs, agentType) => {
    const agentProcessPids = new Set(procs.map((p) => p.pid));

    // Find root processes for this agent (those whose parent is not in this agent's processes)
    const rootProcesses = procs
      .filter((p) => !agentProcessPids.has(p.ppid))
      .sort((a, b) => a.pid - b.pid)
      .map((p) => buildNode(p, agentProcessPids));

    const activeCount = procs.filter((p) => p.endTime === 0).length;

    groups.push({
      agentType,
      displayName: agentDisplayNames[agentType] || agentType,
      rootProcesses,
      processCount: procs.length,
      activeCount,
    });
  });

  // Sort by active count descending
  return groups.sort((a, b) => b.activeCount - a.activeCount);
}

function ProcessTreeNode({
  node,
  depth,
  selectedProcess,
  onSelectProcess,
  expandedPids,
  toggleExpand,
}: {
  node: TreeNode;
  depth: number;
  selectedProcess: Process | null;
  onSelectProcess: (process: Process | null) => void;
  expandedPids: Set<number>;
  toggleExpand: (pid: number) => void;
}) {
  const { process, children } = node;
  const hasChildren = children.length > 0;
  const isExpanded = expandedPids.has(process.pid);
  const isSelected = selectedProcess?.pid === process.pid;
  const isActive = process.endTime === 0;

  return (
    <div className="tree-node-container">
      <div
        className={`tree-node ${isSelected ? "selected" : ""}`}
        style={{ paddingLeft: depth * 16 + 8 }}
        onClick={() => onSelectProcess(isSelected ? null : process)}
      >
        {hasChildren ? (
          <button
            className="tree-expand-btn"
            onClick={(e) => {
              e.stopPropagation();
              toggleExpand(process.pid);
            }}
          >
            {isExpanded ? (
              <ChevronDown size={14} />
            ) : (
              <ChevronRight size={14} />
            )}
          </button>
        ) : (
          <span className="tree-expand-spacer" />
        )}

        <span
          className="tree-status-dot"
          style={{
            background: isActive ? "var(--accent-success)" : "var(--text-muted)",
          }}
        />

        <span className="tree-process-name">{process.name}</span>
        <span className="tree-process-pid">(PID {process.pid})</span>
      </div>

      {hasChildren && isExpanded && (
        <div className="tree-children">
          {children.map((child) => (
            <ProcessTreeNode
              key={child.process.pid}
              node={child}
              depth={depth + 1}
              selectedProcess={selectedProcess}
              onSelectProcess={onSelectProcess}
              expandedPids={expandedPids}
              toggleExpand={toggleExpand}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function AgentGroupNode({
  group,
  selectedProcess,
  onSelectProcess,
  expandedPids,
  toggleExpand,
  isExpanded,
  onToggleGroup,
}: {
  group: AgentGroup;
  selectedProcess: Process | null;
  onSelectProcess: (process: Process | null) => void;
  expandedPids: Set<number>;
  toggleExpand: (pid: number) => void;
  isExpanded: boolean;
  onToggleGroup: () => void;
}) {
  const agentColor = agentColors[group.agentType] || "var(--agent-unknown)";
  const icon = agentIcons[group.agentType] || "??";

  return (
    <div className="agent-group">
      <div className="agent-group-header" onClick={onToggleGroup}>
        <button className="tree-expand-btn">
          {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        </button>
        <span className="agent-group-icon" style={{ background: agentColor }}>
          {icon}
        </span>
        <span className="agent-group-name">{group.displayName}</span>
        <span className="agent-group-stats">
          <span className="agent-group-active">{group.activeCount} active</span>
          <span className="agent-group-total">/ {group.processCount}</span>
        </span>
      </div>
      {isExpanded && (
        <div className="agent-group-content">
          {group.rootProcesses.map((node) => (
            <ProcessTreeNode
              key={node.process.pid}
              node={node}
              depth={1}
              selectedProcess={selectedProcess}
              onSelectProcess={onSelectProcess}
              expandedPids={expandedPids}
              toggleExpand={toggleExpand}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function ProcessTree({
  processes,
  selectedProcess,
  onSelectProcess,
}: ProcessTreeProps) {
  const [expandedPids, setExpandedPids] = useState<Set<number>>(new Set());
  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());

  const agentGroups = useMemo(() => buildAgentGroups(processes), [processes]);

  // Auto-expand first agent group and its root processes
  useMemo(() => {
    if (expandedAgents.size === 0 && agentGroups.length > 0) {
      const initialAgents = new Set<string>();
      const initialPids = new Set<number>();

      agentGroups.forEach((group) => {
        initialAgents.add(group.agentType);
        // Expand root processes of each agent
        group.rootProcesses.forEach((node) => {
          initialPids.add(node.process.pid);
        });
      });

      setExpandedAgents(initialAgents);
      setExpandedPids(initialPids);
    }
  }, [agentGroups]);

  const toggleExpand = (pid: number) => {
    setExpandedPids((prev) => {
      const next = new Set(prev);
      if (next.has(pid)) {
        next.delete(pid);
      } else {
        next.add(pid);
      }
      return next;
    });
  };

  const toggleAgentGroup = (agentType: string) => {
    setExpandedAgents((prev) => {
      const next = new Set(prev);
      if (next.has(agentType)) {
        next.delete(agentType);
      } else {
        next.add(agentType);
      }
      return next;
    });
  };

  const expandAll = () => {
    const allPids = new Set<number>();
    const allAgents = new Set<string>();

    function collectPids(nodes: TreeNode[]) {
      nodes.forEach((node) => {
        if (node.children.length > 0) {
          allPids.add(node.process.pid);
          collectPids(node.children);
        }
      });
    }

    agentGroups.forEach((group) => {
      allAgents.add(group.agentType);
      collectPids(group.rootProcesses);
    });

    setExpandedAgents(allAgents);
    setExpandedPids(allPids);
  };

  const collapseAll = () => {
    setExpandedAgents(new Set());
    setExpandedPids(new Set());
  };

  if (processes.length === 0) {
    return (
      <div className="process-tree">
        <div className="tree-header">
          <span className="tree-title">Agents</span>
        </div>
        <div className="empty-state">
          <div className="empty-state-icon">📊</div>
          <div className="empty-state-title">No agents</div>
          <div className="empty-state-description">
            Connect to the agent to see tracked processes
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="process-tree">
      <div className="tree-header">
        <span className="tree-title">Agents</span>
        <div className="tree-actions">
          <button className="tree-action-btn" onClick={expandAll} title="Expand all">
            <ChevronDown size={12} />
          </button>
          <button className="tree-action-btn" onClick={collapseAll} title="Collapse all">
            <ChevronRight size={12} />
          </button>
        </div>
      </div>
      <div className="tree-content">
        {agentGroups.map((group) => (
          <AgentGroupNode
            key={group.agentType}
            group={group}
            selectedProcess={selectedProcess}
            onSelectProcess={onSelectProcess}
            expandedPids={expandedPids}
            toggleExpand={toggleExpand}
            isExpanded={expandedAgents.has(group.agentType)}
            onToggleGroup={() => toggleAgentGroup(group.agentType)}
          />
        ))}
      </div>
    </div>
  );
}
