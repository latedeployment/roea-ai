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

const agentIcons: Record<string, string> = {
  claude_code: "CC",
  cursor: "Cu",
  aider: "Ai",
  windsurf: "Ws",
  copilot: "Cp",
  continue_dev: "Cn",
};

const agentColors: Record<string, string> = {
  claude_code: "var(--agent-claude)",
  cursor: "var(--agent-cursor)",
  aider: "var(--agent-aider)",
  windsurf: "var(--agent-windsurf)",
  copilot: "var(--agent-copilot)",
  continue_dev: "var(--agent-unknown)",
};

function buildTree(processes: Process[]): TreeNode[] {
  const processMap = new Map<number, Process>();
  const childrenMap = new Map<number, Process[]>();

  // Build lookup maps
  processes.forEach((p) => {
    processMap.set(p.pid, p);
    if (!childrenMap.has(p.ppid)) {
      childrenMap.set(p.ppid, []);
    }
    childrenMap.get(p.ppid)!.push(p);
  });

  // Find root processes (those whose parent is not in our list)
  const roots = processes.filter((p) => !processMap.has(p.ppid));

  // Recursively build tree
  function buildNode(process: Process): TreeNode {
    const children = (childrenMap.get(process.pid) || [])
      .sort((a, b) => a.pid - b.pid)
      .map(buildNode);
    return { process, children };
  }

  return roots.sort((a, b) => a.pid - b.pid).map(buildNode);
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
  const isAgent = process.agentType !== "" && process.agentType !== "unknown";
  const agentColor = agentColors[process.agentType] || "var(--agent-unknown)";

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

        {isAgent && (
          <span
            className="tree-agent-badge"
            style={{ background: agentColor }}
          >
            {agentIcons[process.agentType] || "??"}
          </span>
        )}

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

export function ProcessTree({
  processes,
  selectedProcess,
  onSelectProcess,
}: ProcessTreeProps) {
  const [expandedPids, setExpandedPids] = useState<Set<number>>(new Set());

  const tree = useMemo(() => buildTree(processes), [processes]);

  // Auto-expand all root nodes initially
  useMemo(() => {
    if (expandedPids.size === 0 && tree.length > 0) {
      const initialExpanded = new Set<number>();
      tree.forEach((node) => initialExpanded.add(node.process.pid));
      setExpandedPids(initialExpanded);
    }
  }, [tree]);

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

  const expandAll = () => {
    const allPids = new Set<number>();
    function collectPids(nodes: TreeNode[]) {
      nodes.forEach((node) => {
        if (node.children.length > 0) {
          allPids.add(node.process.pid);
          collectPids(node.children);
        }
      });
    }
    collectPids(tree);
    setExpandedPids(allPids);
  };

  const collapseAll = () => {
    setExpandedPids(new Set());
  };

  if (processes.length === 0) {
    return (
      <div className="process-tree">
        <div className="tree-header">
          <span className="tree-title">Process Tree</span>
        </div>
        <div className="empty-state">
          <div className="empty-state-icon">📊</div>
          <div className="empty-state-title">No processes</div>
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
        <span className="tree-title">Process Tree</span>
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
        {tree.map((node) => (
          <ProcessTreeNode
            key={node.process.pid}
            node={node}
            depth={0}
            selectedProcess={selectedProcess}
            onSelectProcess={onSelectProcess}
            expandedPids={expandedPids}
            toggleExpand={toggleExpand}
          />
        ))}
      </div>
    </div>
  );
}
